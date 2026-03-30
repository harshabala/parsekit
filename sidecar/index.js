import { LiteParse } from "@llamaindex/liteparse";
import fs from "fs/promises";
import path from "path";
import pLimit from "p-limit";
import { stdin, stdout } from "process";

const parser = new LiteParse();

async function processFile(filePath, outDir, format, ocrLanguage) {
  try {
    const stats = await fs.stat(filePath);
    if (stats.isDirectory()) return;

    const ext = path.extname(filePath).toLowerCase();
    const supportedExts = [".pdf", ".docx", ".doc", ".pptx", ".ppt", ".xlsx", ".xls", ".png", ".jpg", ".jpeg"];
    if (!supportedExts.includes(ext)) return;

    stdout.write(JSON.stringify({ type: "progress", file: path.basename(filePath), status: "parsing" }) + "\n");

    const result = await parser.parse(filePath, {
      language: ocrLanguage || "eng"
    });

    let outputContent = "";
    let outExt = ".md";

    if (format === "json") {
      outputContent = JSON.stringify(result, null, 2);
      outExt = ".json";
    } else if (format === "txt") {
      outputContent = result.pages.map(p => p.text).join("\n\n---\n\n");
      outExt = ".txt";
    } else {
      // Default to Markdown
      outputContent = result.pages.map((p, i) => `# Page ${i + 1}\n\n${p.text}`).join("\n\n---\n\n");
      outExt = ".md";
    }

    const outPath = path.join(outDir, path.basename(filePath, ext) + outExt);
    await fs.writeFile(outPath, outputContent);

    stdout.write(JSON.stringify({ type: "progress", file: path.basename(filePath), status: "completed", path: outPath }) + "\n");
  } catch (error) {
    stdout.write(JSON.stringify({ type: "progress", file: path.basename(filePath), status: "error", error: error.message }) + "\n");
  }
}

async function run(config) {
  const { inputDir, outputDir, format, ocrLanguage, concurrency = 4 } = config;

  try {
    await fs.mkdir(outputDir, { recursive: true });
    const files = await fs.readdir(inputDir);
    const limit = pLimit(concurrency);

    const tasks = files.map(file => {
      const filePath = path.join(inputDir, file);
      return limit(() => processFile(filePath, outputDir, format, ocrLanguage));
    });

    await Promise.all(tasks);
    stdout.write(JSON.stringify({ type: "done" }) + "\n");
  } catch (error) {
    stdout.write(JSON.stringify({ type: "error", message: error.message }) + "\n");
  }
}

// Receive config via stdin
let data = "";
stdin.on("data", chunk => {
  data += chunk;
});

stdin.on("end", () => {
  try {
    const config = JSON.parse(data);
    run(config);
  } catch (error) {
    stdout.write(JSON.stringify({ type: "error", message: "Invalid JSON input" }) + "\n");
  }
});
