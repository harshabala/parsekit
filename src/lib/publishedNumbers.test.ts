import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { describe, expect, it } from "vitest";
import { LONG_CHATGPT_MESSAGE_TOKENS } from "./tokenStats";

const readme = readFileSync(new URL("../../README.md", import.meta.url), "utf8");
const ratios = JSON.parse(
  readFileSync(new URL("../../scripts/benchmark-ratios.json", import.meta.url), "utf8"),
) as {
  by_file_type: Record<string, { avg_reduction_ratio: number }>;
};
const en = JSON.parse(
  readFileSync(new URL("../locales/en.json", import.meta.url), "utf8"),
) as { tokenSavings: { panelHint: string } };
const es = JSON.parse(
  readFileSync(new URL("../locales/es.json", import.meta.url), "utf8"),
) as { tokenSavings: { panelHint: string } };
const zh = JSON.parse(
  readFileSync(new URL("../locales/zh.json", import.meta.url), "utf8"),
) as { tokenSavings: { panelHint: string } };

describe("published token numbers", () => {
  it("quotes the measured formula, not a four-character estimate", () => {
    expect(readme).not.toContain("4 chars/token");
    expect(readme).toContain("cl100k_base");
    expect(readme).toContain(String(LONG_CHATGPT_MESSAGE_TOKENS));
    expect(readme).toContain("0.0%");
    for (const entry of Object.values(ratios.by_file_type)) {
      expect(entry.avg_reduction_ratio).toBe(0);
    }
    expect(en.tokenSavings.panelHint).toContain("cl100k");
    expect(en.tokenSavings.panelHint).not.toContain("avoided sending");
    expect(es.tokenSavings.panelHint).not.toContain("evitaste enviar");
    expect(zh.tokenSavings.panelHint).not.toContain("避免的 token");

    const printed = execFileSync("node", ["scripts/reproduce_published_numbers.mjs"], {
      encoding: "utf8",
    });
    expect(printed).toContain("tokenizer cl100k_base");
    expect(printed).toContain(`long ChatGPT message ${LONG_CHATGPT_MESSAGE_TOKENS} tokens`);
    expect(printed).toContain("pdf avg_reduction_ratio 0");
    expect(printed).toContain("spreadsheet.xlsx delta 0.0%");
  });
});
