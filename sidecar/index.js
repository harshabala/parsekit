import { LiteParse } from "@llamaindex/liteparse";
import fs from "fs/promises";
import path from "path";
import pLimit from "p-limit";
import { createInterface } from "readline";
import { stdin, stdout } from "process";

/** Dev-only Node sidecar. Production uses the Rust parsekit-sidecar binary. */
const SPREADSHEET_EXTENSIONS = new Set([".xls", ".xlsx", ".xlsm", ".ods", ".csv", ".tsv"]);

function isSpreadsheet(filePath) {
  return SPREADSHEET_EXTENSIONS.has(path.extname(filePath).toLowerCase());
}

function validateFormat(format) {
  if (!["md", "json", "txt"].includes(format)) {
    throw new Error(`Invalid format "${format}". Expected md, json, or txt.`);
  }
}

function getOutputPath(filePath, outDir, format) {
  const ext = path.extname(filePath).toLowerCase();
  const baseName = path.basename(filePath, ext);
  const isSpreadsheetFile = isSpreadsheet(filePath);
  let outExt;
  if (isSpreadsheetFile) {
    outExt = ".json";
  } else if (format === "json") {
    outExt = ".json";
  } else if (format === "txt") {
    outExt = ".txt";
  } else {
    outExt = ".md";
  }
  const outPath = path.join(outDir, baseName + outExt);
  return { outPath, baseName, isSpreadsheetFile };
}

function formatOutput(result, baseName, format, isSpreadsheetFile) {
  if (isSpreadsheetFile || format === "json") {
    return JSON.stringify(result, null, 2);
  }
  if (format === "txt") {
    if (result.pages?.length) {
      return result.pages.map((p) => p.text).join("\n\n---\n\n");
    }
    return result.text ?? JSON.stringify(result, null, 2);
  }
  if (result.pages?.length) {
    const pages = result.pages.map((p, i) => `## Page ${i + 1}\n\n${p.text}`);
    return `# ${baseName}\n\n${pages.join("\n\n---\n\n")}`;
  }
  return `# ${baseName}\n\n${JSON.stringify(result, null, 2)}`;
}

function createParser(ocrEnabled, ocrLanguage, fileConcurrency) {
  const numWorkers = ocrEnabled && fileConcurrency > 1 ? 1 : Math.max(1, fileConcurrency);
  return new LiteParse({
    ocrEnabled,
    ocrLanguage,
    quiet: true,
    numWorkers,
  });
}

async function processFile(filePath, outDir, format, ocrEnabled, ocrLanguage, fileConcurrency) {
  const fileName = path.basename(filePath);
  const { outPath, baseName, isSpreadsheetFile } = getOutputPath(filePath, outDir, format);

  try {
    const stat = await fs.stat(outPath);
    if (stat.size > 0) {
      stdout.write(JSON.stringify({ type: "progress", file: fileName, status: "skipped", path: outPath }) + "\n");
      return "skipped";
    }
  } catch {
    // File does not exist — proceed with parsing
  }

  stdout.write(JSON.stringify({ type: "progress", file: fileName, status: "parsing" }) + "\n");

  const parser = createParser(ocrEnabled, ocrLanguage, fileConcurrency);
  const result = await parser.parse(filePath);
  const outputContent = formatOutput(result, baseName, format, isSpreadsheetFile);

  await fs.writeFile(outPath, outputContent);

  stdout.write(JSON.stringify({ type: "progress", file: fileName, status: "completed", path: outPath }) + "\n");

  return "completed";
}

async function run(config) {
  const { files, outputDir, format, ocrEnabled = true, ocrLanguage = "eng", workers = 4 } = config;

  validateFormat(format);

  try {
    const fileConcurrency = Math.max(1, Number(workers) || 4);

    await fs.mkdir(outputDir, { recursive: true });
    const limit = pLimit(fileConcurrency);

    stdout.write(JSON.stringify({ type: "start", total: files.length }) + "\n");

    let parsed = 0;
    let skipped = 0;
    let errors = 0;

    const tasks = files.map((filePath) =>
      limit(async () => {
        try {
          const result = await processFile(
            filePath,
            outputDir,
            format,
            ocrEnabled,
            ocrLanguage,
            fileConcurrency
          );
          if (result === "skipped") {
            skipped++;
          } else {
            parsed++;
          }
        } catch (error) {
          errors++;
          stdout.write(
            JSON.stringify({
              type: "progress",
              file: path.basename(filePath),
              status: "error",
              error: error.message,
            }) + "\n"
          );
        }
      })
    );

    await Promise.all(tasks);
    stdout.write(JSON.stringify({ type: "done", parsed, skipped, errors }) + "\n");
  } catch (error) {
    stdout.write(JSON.stringify({ type: "error", message: error.message }) + "\n");
  }
}

const rl = createInterface({ input: stdin });
rl.on("line", (line) => {
  try {
    const config = JSON.parse(line);
    run(config).then(() => process.exit(0));
  } catch (error) {
    stdout.write(JSON.stringify({ type: "error", message: "Invalid JSON input" }) + "\n");
    process.exit(1);
  }
});