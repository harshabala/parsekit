import { readFileSync } from "node:fs";

const readme = readFileSync(new URL("../README.md", import.meta.url), "utf8");
const ratios = JSON.parse(
  readFileSync(new URL("./benchmark-ratios.json", import.meta.url), "utf8"),
);
const results = readFileSync(
  new URL("../docs/benchmark-results.md", import.meta.url),
  "utf8",
);
const tokenStats = readFileSync(
  new URL("../src/lib/tokenStats.ts", import.meta.url),
  "utf8",
);

const messageMatch = tokenStats.match(
  /LONG_CHATGPT_MESSAGE_TOKENS = (\d+)/,
);
if (!messageMatch) {
  console.error("missing LONG_CHATGPT_MESSAGE_TOKENS");
  process.exit(1);
}
const messageTokens = messageMatch[1];

const lines = [
  "tokenizer cl100k_base",
  `long ChatGPT message ${messageTokens} tokens`,
];
for (const [type, entry] of Object.entries(ratios.by_file_type)) {
  lines.push(`${type} avg_reduction_ratio ${entry.avg_reduction_ratio}`);
}
const deltas = [...results.matchAll(/\| (\S+\.\w+) \|[^|]+\|[^|]+\|[^|]+\|[^|]+\| ([0-9.]+%) \|/g)];
if (deltas.length === 0) {
  console.error("benchmark table has no delta column");
  process.exit(1);
}
for (const match of deltas) {
  lines.push(`${match[1]} delta ${match[2]}`);
}

for (const line of lines) {
  console.log(line);
}

const required = ["cl100k_base", messageTokens, "0.0%"];
const missing = required.filter((item) => !readme.includes(item));
if (missing.length > 0) {
  console.error(`README is missing: ${missing.join(", ")}`);
  process.exit(1);
}
if (readme.includes("4 chars/token")) {
  console.error("README still claims 4 chars/token");
  process.exit(1);
}
