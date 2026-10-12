import { describe, expect, it } from "vitest";
import en from "../locales/en.json";
import es from "../locales/es.json";
import zh from "../locales/zh.json";

function flatten(obj: Record<string, unknown>, prefix = ""): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [key, value] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object") {
      Object.assign(out, flatten(value as Record<string, unknown>, path));
    } else {
      out[path] = String(value);
    }
  }
  return out;
}

const placeholders = (s: string) => (s.match(/\{[a-zA-Z0-9_]+\}/g) ?? []).sort().join(",");

describe("locale catalogs", () => {
  const base = flatten(en);
  for (const [name, catalog] of [
    ["es", es],
    ["zh", zh],
  ] as const) {
    const other = flatten(catalog as Record<string, unknown>);

    it(`${name} has exactly the same keys as en`, () => {
      expect(Object.keys(other).sort()).toEqual(Object.keys(base).sort());
    });

    it(`${name} has no empty strings`, () => {
      expect(
        Object.entries(other)
          .filter(([, v]) => !v.trim())
          .map(([k]) => k),
      ).toEqual([]);
    });

    it(`${name} keeps the same {placeholders} as en`, () => {
      const mismatched = Object.keys(base).filter(
        (k) => k in other && placeholders(base[k]) !== placeholders(other[k]),
      );
      expect(mismatched).toEqual([]);
    });
  }
});
