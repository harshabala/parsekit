import { describe, expect, it } from "vitest";
import {
  ACTIVATION_EVENT,
  isActivated,
  qualifiesForActivation,
} from "./activation";

describe("activation", () => {
  it("exports a stable activation event name", () => {
    expect(ACTIVATION_EVENT).toBe(
      "first_successful_convert_with_token_estimate",
    );
  });

  it("isActivated requires a timestamp", () => {
    expect(isActivated(null)).toBe(false);
    expect(isActivated("")).toBe(false);
    expect(isActivated("2026-07-10T00:00:00.000Z")).toBe(true);
  });

  it("qualifies when first success with token estimate", () => {
    expect(
      qualifiesForActivation({
        alreadyActivated: false,
        successCount: 1,
        tokensSavedInBatch: 0,
      }),
    ).toBe(true);
    expect(
      qualifiesForActivation({
        alreadyActivated: false,
        successCount: 0,
        tokensSavedInBatch: 100,
      }),
    ).toBe(false);
    expect(
      qualifiesForActivation({
        alreadyActivated: true,
        successCount: 5,
        tokensSavedInBatch: 100,
      }),
    ).toBe(false);
  });
});
