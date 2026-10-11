import { describe, expect, it } from "vitest";
import {
  approximateChatGptMessages,
  endOfMonth,
  endOfWeek,
  filterEventsInRange,
  formatTokenCount,
  pagesUnlockedThisMonth,
  startOfMonth,
  startOfWeek,
  sumPagesUnlockedInRange,
  sumTokensInRange,
  tokensForPeriod,
  tokensSavedThisMonth,
  tokensSavedThisWeek,
  tokensSavedToday,
  type TokenEvent,
  type TokenStats,
} from "./tokenStats";

function event(
  ts: string,
  tokensSaved: number,
  pagesUnlocked = 0,
  fileType = "pdf",
): TokenEvent {
  return { ts, file_type: fileType, tokens_saved: tokensSaved, pages_unlocked: pagesUnlocked };
}

function stats(events: TokenEvent[]): TokenStats {
  return {
    total_files_converted: events.length,
    total_tokens_saved: events.reduce((sum, e) => sum + e.tokens_saved, 0),
    total_pages_unlocked: events.reduce((sum, e) => sum + e.pages_unlocked, 0),
    total_documents_unlocked: 0,
    by_file_type: {},
    events,
  };
}

describe("tokenStats aggregation", () => {
  it("filters events by inclusive start and exclusive end", () => {
    const events = [
      event("2026-07-01T10:00:00Z", 100),
      event("2026-07-15T10:00:00Z", 200),
      event("2026-08-01T10:00:00Z", 300),
    ];
    const july = filterEventsInRange(
      events,
      new Date("2026-07-01T00:00:00Z"),
      new Date("2026-08-01T00:00:00Z"),
    );
    expect(july).toHaveLength(2);
    expect(sumTokensInRange(july, new Date(0), new Date(8_640_000_000_000_000))).toBe(300);
  });

  it("sums tokens and pages separately in a range", () => {
    const events = [
      event("2026-07-02T12:00:00Z", 50, 3),
      event("2026-07-03T12:00:00Z", 75, 0),
    ];
    const start = new Date("2026-07-01T00:00:00Z");
    const end = new Date("2026-08-01T00:00:00Z");
    expect(sumTokensInRange(events, start, end)).toBe(125);
    expect(sumPagesUnlockedInRange(events, start, end)).toBe(3);
  });

  it("computes this month from event timestamps", () => {
    const now = new Date("2026-07-20T15:00:00Z");
    const fixture = stats([
      event("2026-07-05T10:00:00Z", 400),
      event("2026-06-28T10:00:00Z", 900),
      event("2026-08-02T10:00:00Z", 100),
    ]);
    expect(tokensSavedThisMonth(fixture, now)).toBe(400);
    expect(pagesUnlockedThisMonth(fixture, now)).toBe(0);
  });

  it("computes this week with Monday start", () => {
    const now = new Date("2026-07-03T12:00:00Z");
    const fixture = stats([
      event("2026-06-30T10:00:00Z", 100),
      event("2026-07-01T10:00:00Z", 200),
      event("2026-07-05T10:00:00Z", 300),
      event("2026-07-08T10:00:00Z", 400),
    ]);
    const weekStart = startOfWeek(now);
    const weekEnd = endOfWeek(now);
    expect(weekEnd.getTime() - weekStart.getTime()).toBe(7 * 24 * 60 * 60 * 1000);
    expect(tokensSavedThisWeek(fixture, now)).toBe(600);
  });

  it("exposes calendar period boundaries", () => {
    const now = new Date("2026-07-15T12:00:00Z");
    expect(startOfMonth(now).getMonth()).toBe(6);
    expect(startOfMonth(now).getDate()).toBe(1);
    expect(endOfMonth(now).getMonth()).toBe(7);
    expect(endOfMonth(now).getDate()).toBe(1);
  });

  it("selects lifetime, month, or today totals for the banner", () => {
    const now = new Date("2026-07-20T15:00:00");
    const fixture = stats([
      event("2026-07-20T10:00:00", 50),
      event("2026-07-05T10:00:00", 400),
      event("2026-06-28T10:00:00", 900),
    ]);
    expect(tokensForPeriod(fixture, "lifetime", now)).toBe(1350);
    expect(tokensForPeriod(fixture, "month", now)).toBe(450);
    expect(tokensForPeriod(fixture, "today", now)).toBe(50);
    expect(tokensSavedToday(fixture, now)).toBe(50);
  });

  it("formats token counts and approximate ChatGPT messages", () => {
    expect(formatTokenCount(18400)).toBe("18,400");
    expect(approximateChatGptMessages(0)).toBe(0);
    expect(approximateChatGptMessages(400)).toBe(1);
    expect(approximateChatGptMessages(18400)).toBe(23);
  });
});