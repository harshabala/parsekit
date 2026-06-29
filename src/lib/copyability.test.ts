import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("copyability CSS", () => {
  const css = readFileSync(resolve("src/index.css"), "utf8");

  it("does not set user-select none on body", () => {
    expect(css).not.toMatch(/body\s*\{[^}]*user-select:\s*none/s);
  });

  it("defines selectable-content helper", () => {
    expect(css).toContain(".selectable-content");
    expect(css).toMatch(/\.selectable-content\s*\{[^}]*user-select:\s*text/s);
  });
});