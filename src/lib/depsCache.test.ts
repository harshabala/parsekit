import { afterEach, describe, expect, it } from "vitest";
import {
  peekCachedDependencies,
  resetDepsCacheForTests,
  setCachedDependenciesForTests,
  takeCachedDependencies,
} from "./depsCache";

describe("depsCache", () => {
  afterEach(() => resetDepsCacheForTests());

  it("peek returns null when empty", () => {
    expect(peekCachedDependencies()).toBeNull();
  });

  it("take returns cached deps and clears", () => {
    const deps = [{ id: "tesseract", labelKey: "x", installed: true, optional: false, brewHint: "" }];
    setCachedDependenciesForTests(deps);
    expect(takeCachedDependencies()).toEqual(deps);
    expect(peekCachedDependencies()).toBeNull();
  });
});
