import { LiteParse } from "@llamaindex/liteparse";
import fs from "fs/promises";
import path from "path";
import pLimit from "p-limit";
import { createInterface } from "readline";
import { stdin, stdout } from "process";

const supportedExts = [".pdf", ".docx", ".doc", ".pptx", ".ppt", ".xlsx", ".xls", ".png", ".jpg", ".jpeg", ".tiff", ".tif", ".bmp"];

async function walkDir(dir) {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...await walkDir(fullPath));
    } else {
      const ext = path.extname(entry.name).toLowerCase();
      if (supportedExts.includes(ext)) {
        files.push(fullPath);
      }
    }
  }
  return files;
}

async function processFile(parser, filePath, outDir, format) {
  stdout.write(JSON.stringify({ type: "progress", file: path.basename(filePath), status: "parsing" }) + "\n");

  const ext = path.extname(filePath).toLowerCase();
  const baseName = path.basename(filePath, ext);

  const isExcel = [".xlsx", ".xls"].includes(ext);
  // LiteParse takes OCR config in the constructor; the second parse() arg is `quiet`.
  const result = await parser.parse(filePath, true);

  let outputContent = "";
  let outExt = ".md";

  if (isExcel) {
    outputContent = JSON.stringify(result, null, 2);
    outExt = ".json";
  } else if (format === "json") {
    outputContent = JSON.stringify(result, null, 2);
    outExt = ".json";
  } else if (format === "txt") {
    if (result.pages) {
      outputContent = result.pages.map(p => p.text).join("\n\n---\n\n");
    } else {
      outputContent = JSON.stringify(result, null, 2);
    }
    outExt = ".txt";
  } else {
    if (result.pages) {
      const pages = result.pages.map((p, i) => `## Page ${i + 1}\n\n${p.text}`);
      outputContent = `# ${baseName}\n\n${pages.join("\n\n---\n\n")}`;
    } else {
      outputContent = `# ${baseName}\n\n${JSON.stringify(result, null, 2)}`;
    }
    outExt = ".md";
  }

  const outPath = path.join(outDir, baseName + outExt);
  await fs.writeFile(outPath, outputContent);

  stdout.write(JSON.stringify({ type: "progress", file: path.basename(filePath), status: "completed", path: outPath }) + "\n");

  return "completed";
}

async function run(config) {
  const { inputDir, outputDir, format, ocrEnabled = true, ocrLanguage = "eng", workers = 4 } = config;

  try {
    const concurrency = Math.max(1, Number(workers) || 4);
    const parser = new LiteParse({ ocrEnabled, ocrLanguage });

    await fs.mkdir(outputDir, { recursive: true });
    const files = await walkDir(inputDir);
    const limit = pLimit(concurrency);

    // Emit start event with total file count
    stdout.write(JSON.stringify({ type: "start", total: files.length }) + "\n");

    let parsed = 0;
    let errors = 0;

    const tasks = files.map(filePath =>
      limit(async () => {
        try {
          await processFile(parser, filePath, outputDir, format);
          parsed++;
        } catch (error) {
          errors++;
          stdout.write(JSON.stringify({ type: "progress", file: path.basename(filePath), status: "error", error: error.message }) + "\n");
        }
      })
    );

    await Promise.all(tasks);
    stdout.write(JSON.stringify({ type: "done", parsed, skipped: files.length - parsed - errors, errors }) + "\n");
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
