import { describe, expect, it } from "vitest";
import { depRowAriaLabel } from "./depsAria";

describe("depRowAriaLabel", () => {
  it("uses installed copy when tool is present", () => {
    expect(depRowAriaLabel("ImageMagick", true, "installed", "not detected")).toBe(
      "ImageMagick: installed"
    );
  });

  it("uses missing copy when tool is absent", () => {
    expect(depRowAriaLabel("LibreOffice", false, "installed", "not detected")).toBe(
      "LibreOffice: not detected"
    );
  });
});