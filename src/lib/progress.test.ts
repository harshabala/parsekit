import { describe, expect, it } from "vitest";
import {
  applyParseProgressEvent,
  applyTokenSavingsEvent,
  createBatchTokenSavings,
  resolvePrimaryParsingId,
  pathsMatchForProgress,
  settleBatchOnStop,
  settleInFlightOnAbort,
} from "./progress";
import type { FileProgress } from "./types";

function row(id: string, status: FileProgress["status"]): FileProgress {
  return { id, name: id.split("/").pop() ?? id, status };
}

describe("applyParseProgressEvent", () => {
  it("updates by sourcePath", () => {
    const files = [row("/a/one.pdf", "pending"), row("/a/two.pdf", "pending")];
    const { files: next } = applyParseProgressEvent(files, {
      type: "progress",
      sourcePath: "/a/one.pdf",
      file: "one.pdf",
      status: "parsing",
    });
    expect(next[0].status).toBe("parsing");
    expect(next[1].status).toBe("pending");
  });

  it("tracks lastParsingId on parse start", () => {
    const files = [row("/a/one.pdf", "pending")];
    const r1 = applyParseProgressEvent(files, {
      type: "progress",
      sourcePath: "/a/one.pdf",
      status: "parsing",
    });
    expect(r1.lastParsingId).toBe("/a/one.pdf");

    const withTwo = [
      ...r1.files,
      row("/a/two.pdf", "pending"),
    ];
    const r2 = applyParseProgressEvent(withTwo, {
      type: "progress",
      sourcePath: "/a/two.pdf",
      status: "parsing",
    }, r1.lastParsingId);
    expect(r2.lastParsingId).toBe("/a/two.pdf");
  });

  it("continues after one file errors", () => {
    let files = [row("/a/one.pdf", "pending"), row("/a/two.pdf", "pending")];
    let last: string | null = null;

    ({ files, lastParsingId: last } = applyParseProgressEvent(files, {
      type: "progress",
      sourcePath: "/a/one.pdf",
      status: "parsing",
    }, last));

    ({ files, lastParsingId: last } = applyParseProgressEvent(files, {
      type: "progress",
      sourcePath: "/a/one.pdf",
      status: "error",
      error: "bad pdf",
    }, last));

    ({ files, lastParsingId: last } = applyParseProgressEvent(files, {
      type: "progress",
      sourcePath: "/a/two.pdf",
      status: "parsing",
    }, last));

    expect(files[0].status).toBe("error");
    expect(files[1].status).toBe("parsing");
    expect(last).toBe("/a/two.pdf");
  });

  it("returns new array reference on update", () => {
    const files = [row("/a/one.pdf", "pending")];
    const { files: next } = applyParseProgressEvent(files, {
      type: "progress",
      sourcePath: "/a/one.pdf",
      status: "done",
    });
    expect(next).not.toBe(files);
  });
});

describe("pathsMatchForProgress", () => {
  it("treats /var and /private/var as the same file", () => {
    expect(
      pathsMatchForProgress(
        "/var/folders/x/report.pdf",
        "/private/var/folders/x/report.pdf"
      )
    ).toBe(true);
  });
});

describe("settleBatchOnStop", () => {
  it("marks parsing and pending as error with distinct messages", () => {
    const files = [
      row("/a/done.pdf", "done"),
      row("/a/active.pdf", "parsing"),
      row("/a/wait.pdf", "pending"),
    ];
    const next = settleBatchOnStop(files, {
      parsing: "Stopped mid-file",
      pending: "Never started",
    });
    expect(next[0].status).toBe("done");
    expect(next[1].status).toBe("error");
    expect(next[1].error).toBe("Stopped mid-file");
    expect(next[2].status).toBe("error");
    expect(next[2].error).toBe("Never started");
  });
});

describe("settleInFlightOnAbort", () => {
  it("marks both parsing and pending", () => {
    const files = [row("/a/wait.pdf", "pending"), row("/a/active.pdf", "parsing")];
    const next = settleInFlightOnAbort(files, "Stopped");
    expect(next.every((f) => f.status === "error")).toBe(true);
  });
});

describe("applyTokenSavingsEvent", () => {
  it("accumulates batch totals and floors negatives", () => {
    let batch = createBatchTokenSavings();
    batch = applyTokenSavingsEvent(batch, {
      type: "token_savings",
      file_type: "pdf",
      tokens_saved: 120,
      pages_unlocked: 2,
      documents_unlocked: 1,
    });
    batch = applyTokenSavingsEvent(batch, {
      type: "token_savings",
      file_type: "docx",
      tokens_saved: -5,
      pages_unlocked: 0,
      documents_unlocked: 0,
    });
    expect(batch).toEqual({
      tokensSaved: 120,
      pagesUnlocked: 2,
      documentsUnlocked: 1,
    });
  });
});

describe("resolvePrimaryParsingId", () => {
  it("prefers lastParsingId when still parsing", () => {
    const files = [
      { ...row("/a/one.pdf", "parsing") },
      { ...row("/a/two.pdf", "parsing") },
    ];
    expect(resolvePrimaryParsingId(files, "/a/two.pdf")).toBe("/a/two.pdf");
  });

  it("falls back to first parsing row", () => {
    const files = [row("/a/one.pdf", "parsing")];
    expect(resolvePrimaryParsingId(files, null)).toBe("/a/one.pdf");
  });
});