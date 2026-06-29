import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("UI token CSS", () => {
  const css = readFileSync(resolve("src/index.css"), "utf8");

  it("uses linear easing on progress fill", () => {
    expect(css).toMatch(/\.progress-fill\s*\{[^}]*transition:[^}]*linear/s);
  });

  it("styles ::selection", () => {
    expect(css).toContain("::selection");
    expect(css).toMatch(/::selection\s*\{[^}]*background:/s);
  });

  it("offsets settings repo link underline", () => {
    expect(css).toMatch(/\.settings-repo-link[^{]*\{[^}]*text-underline-offset:\s*2px/s);
  });

  it("balances about section headings", () => {
    expect(css).toMatch(/\.about-section-title\s*\{[^}]*text-wrap:\s*balance/s);
    expect(css).toMatch(/\.about-hero\s*\{[^}]*text-wrap:\s*balance/s);
  });

  it("expands workers slider hit area", () => {
    expect(css).toMatch(/\.workers-slider-track-wrap\s*\{[^}]*min-height:\s*44px/s);
    expect(css).toMatch(/\.workers-slider-input::-webkit-slider-thumb\s*\{[^}]*width:\s*22px/s);
  });
});