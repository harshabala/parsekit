import { describe, expect, it } from "vitest";
import {
  hudErrorCount,
  hudFinishedCount,
  hudIsComplete,
  hudProgressPercent,
  hudSuccessCount,
} from "./progressHud";
import type { FileProgress } from "./types";

function row(id: string, status: FileProgress["status"]): FileProgress {
  return { id, name: id.split("/").pop() ?? id, status };
}

describe("progressHud helpers", () => {
  it("counts finished, success, and error rows", () => {
    const files = [
      row("/a/one.pdf", "done"),
      row("/a/two.pdf", "error"),
      row("/a/three.pdf", "parsing"),
      row("/a/four.pdf", "pending"),
    ];
    expect(hudFinishedCount(files)).toBe(2);
    expect(hudSuccessCount(files)).toBe(1);
    expect(hudErrorCount(files)).toBe(1);
    expect(hudProgressPercent(files, 4)).toBe(50);
  });

  it("detects completion only when parsing stopped and all rows settled", () => {
    const files = [row("/a/one.pdf", "done"), row("/a/two.pdf", "error")];
    expect(hudIsComplete(files, 2, false)).toBe(true);
    expect(hudIsComplete(files, 2, true)).toBe(false);
    expect(hudIsComplete([row("/a/one.pdf", "parsing")], 2, false)).toBe(false);
  });
});