/** Tauri global-shortcut string; default ⌃⇧M on macOS. */
export const DEFAULT_GLOBAL_SHORTCUT = "Control+Shift+KeyM";

const MODIFIER_LABELS: Record<string, string> = {
  Control: "⌃",
  Shift: "⇧",
  Alt: "⌥",
  Super: "⌘",
  CommandOrControl: "⌃",
};

function formatKeyToken(token: string): string {
  if (token.startsWith("Key") && token.length === 4) {
    return token.slice(3);
  }
  if (token.startsWith("Digit") && token.length === 6) {
    return token.slice(5);
  }
  if (token === "Space") return "Space";
  return token;
}

/** Human-readable shortcut for settings UI (macOS symbols). */
export function formatShortcutDisplay(shortcut: string): string {
  const parts = shortcut.split("+").filter(Boolean);
  if (parts.length === 0) return shortcut;
  const key = parts[parts.length - 1];
  const mods = parts.slice(0, -1);
  const modText = mods.map((m) => MODIFIER_LABELS[m] ?? m).join("");
  return `${modText}${formatKeyToken(key)}`;
}

/** Build a Tauri shortcut string from a keyboard event while recording. */
export function keyboardEventToShortcut(event: KeyboardEvent): string | null {
  if (event.key === "Escape") return null;
  const code = event.code;
  const isKey =
    code.startsWith("Key") ||
    code.startsWith("Digit") ||
    code === "Space" ||
    code.startsWith("F") && /^F\d+$/.test(code);
  if (!isKey) return null;

  const mods: string[] = [];
  if (event.ctrlKey) mods.push("Control");
  if (event.altKey) mods.push("Alt");
  if (event.shiftKey) mods.push("Shift");
  if (event.metaKey) mods.push("Super");
  if (mods.length === 0) return null;

  return [...mods, code].join("+");
}