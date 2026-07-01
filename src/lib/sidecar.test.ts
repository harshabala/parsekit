import { describe, expect, it } from "vitest";
import { parseSidecarLine } from "./sidecar";

describe("parseSidecarLine", () => {
  it("parses token_savings events", () => {
    const event = parseSidecarLine(
      JSON.stringify({
        type: "token_savings",
        file: "report.pdf",
        file_type: "pdf",
        tokens_saved: 340,
        pages_unlocked: 0,
        documents_unlocked: 0,
      }),
    );
    expect(event).toEqual({
      type: "token_savings",
      file: "report.pdf",
      file_type: "pdf",
      tokens_saved: 340,
      pages_unlocked: 0,
      documents_unlocked: 0,
    });
  });

  it("returns null for blank lines", () => {
    expect(parseSidecarLine("   ")).toBeNull();
  });

  it("returns null for invalid JSON", () => {
    expect(parseSidecarLine("not json")).toBeNull();
  });
});