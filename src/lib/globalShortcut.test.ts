import { describe, expect, it } from "vitest";
import {
  DEFAULT_GLOBAL_SHORTCUT,
  formatShortcutDisplay,
  keyboardEventToShortcut,
} from "./globalShortcut";

describe("globalShortcut", () => {
  it("uses Control+Shift+KeyM as default", () => {
    expect(DEFAULT_GLOBAL_SHORTCUT).toBe("Control+Shift+KeyM");
  });

  it("formats default shortcut for macOS display", () => {
    expect(formatShortcutDisplay("Control+Shift+KeyM")).toBe("⌃⇧M");
  });

  it("builds shortcut from keyboard event", () => {
    const event = {
      code: "KeyM",
      key: "m",
      ctrlKey: true,
      shiftKey: true,
      altKey: false,
      metaKey: false,
    } as KeyboardEvent;
    expect(keyboardEventToShortcut(event)).toBe("Control+Shift+KeyM");
  });
});