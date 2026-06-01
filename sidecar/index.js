import { LiteParse } from "@llamaindex/liteparse";
import fs from "fs/promises";
import path from "path";
import pLimit from "p-limit";
import { createInterface } from "readline";
import { stdin, stdout } from "process";

// NOTE: the format->extension mapping here must stay in sync with the format->content branches in processFile.
function getOutputPath(filePath, outDir, format) {
  const ext = path.extname(filePath).toLowerCase();
  const baseName = path.basename(filePath, ext);
  const isExcel = [".xlsx", ".xls"].includes(ext);
  let outExt;
  if (isExcel) {
    outExt = ".json";
  } else if (format === "json") {
    outExt = ".json";
  } else if (format === "txt") {
    outExt = ".txt";
  } else {
    outExt = ".md";
  }
  const outPath = path.join(outDir, baseName + outExt);
  return { outPath, baseName, isExcel };
}

async function processFile(parser, filePath, outDir, format) {
  const fileName = path.basename(filePath);
  const { outPath, baseName, isExcel } = getOutputPath(filePath, outDir, format);

  // Skip if a non-empty output already exists (empty = interrupted/truncated prior run, re-parse it)
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

  // LiteParse takes OCR config in the constructor; the second parse() arg is `quiet`.
  const result = await parser.parse(filePath, true);

  let outputContent = "";

  if (isExcel) {
    outputContent = JSON.stringify(result, null, 2);
  } else if (format === "json") {
    outputContent = JSON.stringify(result, null, 2);
  } else if (format === "txt") {
    if (result.pages) {
      outputContent = result.pages.map(p => p.text).join("\n\n---\n\n");
    } else {
      outputContent = JSON.stringify(result, null, 2);
    }
  } else {
    if (result.pages) {
      const pages = result.pages.map((p, i) => `## Page ${i + 1}\n\n${p.text}`);
      outputContent = `# ${baseName}\n\n${pages.join("\n\n---\n\n")}`;
    } else {
      outputContent = `# ${baseName}\n\n${JSON.stringify(result, null, 2)}`;
    }
  }

  await fs.writeFile(outPath, outputContent);

  stdout.write(JSON.stringify({ type: "progress", file: fileName, status: "completed", path: outPath }) + "\n");

  return "completed";
}

async function run(config) {
  const { files, outputDir, format, ocrEnabled = true, ocrLanguage = "eng", workers = 4 } = config;

  try {
    const concurrency = Math.max(1, Number(workers) || 4);
    const parser = new LiteParse({ ocrEnabled, ocrLanguage });

    await fs.mkdir(outputDir, { recursive: true });
    const limit = pLimit(concurrency);

    // Emit start event with total file count
    stdout.write(JSON.stringify({ type: "start", total: files.length }) + "\n");

    let parsed = 0;
    let skipped = 0;
    let errors = 0;

    const tasks = files.map(filePath =>
      limit(async () => {
        try {
          const result = await processFile(parser, filePath, outputDir, format);
          if (result === "skipped") {
            skipped++;
          } else {
            parsed++;
          }
        } catch (error) {
          errors++;
          stdout.write(JSON.stringify({ type: "progress", file: path.basename(filePath), status: "error", error: error.message }) + "\n");
        }
      })
    );

    await Promise.all(tasks);
    stdout.write(JSON.stringify({ type: "done", parsed, skipped, errors }) + "\n");
  } catch (error) {
    stdout.write(JSON.stringify({ type: "error", message: error.message }) + "\n");
  }
}

// Receive config via stdin as a single JSON line
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
