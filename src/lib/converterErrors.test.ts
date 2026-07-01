import { describe, expect, it } from "vitest";
import { isConverterDependencyError } from "./converterErrors";

describe("isConverterDependencyError", () => {
  it("detects ImageMagick messages", () => {
    expect(isConverterDependencyError("ImageMagick is required for images.")).toBe(true);
  });

  it("detects LibreOffice messages", () => {
    expect(isConverterDependencyError("LibreOffice is required for Office documents.")).toBe(
      true
    );
  });

  it("detects brew install hints", () => {
    expect(isConverterDependencyError("Install with: brew install imagemagick")).toBe(true);
  });

  it("returns false for unrelated errors", () => {
    expect(isConverterDependencyError("Parse engine stopped responding.")).toBe(false);
    expect(isConverterDependencyError(null)).toBe(false);
  });
});