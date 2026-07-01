import { Command, type Child } from "@tauri-apps/plugin-shell";

/**
 * Sidecar stdout protocol (JSON lines):
 * - `start` — batch began
 * - `progress` — per-file updates (`status`: parsing | completed | error | skipped)
 * - `done` — batch finished successfully
 * - `error` — fatal batch failure (process also exits non-zero or closes); UI must stop parsing
 * - `token_savings` — per-file token savings after successful parse (frontend records via IPC)
 *
 * Per-file failures use `progress` with `status: "error"`, not global `error`.
 */

export interface ParseConfig {
  /** Informational only — directory scanning is done frontend-side; the sidecar is driven by `files`. */
  inputDir: string;
  files: string[];
  outputDir: string;
  format: "md" | "json" | "txt";
  ocrEnabled?: boolean;
  ocrLanguage?: string;
  workers?: number;
}

export interface ParseEvent {
  type: "start" | "progress" | "done" | "error" | "token_savings";
  file?: string;
  /** Full path of the input file being processed */
  sourcePath?: string;
  status?: string;
  path?: string;
  message?: string;
  total?: number;
  parsed?: number;
  skipped?: number;
  errors?: number;
  error?: string;
  /** Present on `token_savings` events */
  file_type?: string;
  tokens_saved?: number;
  pages_unlocked?: number;
  documents_unlocked?: number;
}

export interface ParseRunHandle {
  promise: Promise<void>;
  cancel: () => void;
}

function friendlySidecarMessage(raw: string): string {
  const trimmed = raw.trim();
  const lower = trimmed.toLowerCase();
  if (lower.includes("imagemagick")) {
    return "ImageMagick is required for images. Open Settings → File Support to install.";
  }
  if (lower.includes("libreoffice")) {
    return "LibreOffice is required for Office documents. Open Settings → File Support to install.";
  }
  if (
    lower.includes("scoped command") && lower.includes("not found")
  ) {
    return `Parse engine scope missing in this build. Install ParseKit from the latest DMG. (${trimmed})`;
  }
  if (lower.includes("not allowed by acl") || lower.includes("shell:allow-spawn")) {
    return `Parse engine blocked by app permissions. Quit ParseKit, reinstall the latest build, then try again. (${trimmed})`;
  }
  if (lower.includes("sidecar not configured")) {
    return `Parse engine misconfigured. Rebuild with npm run build:sidecar. (${trimmed})`;
  }
  if (lower.includes("failed to create the path to the command")) {
    return `Parse engine binary missing next to the app. Run npm run build:sidecar, then restart. (${trimmed})`;
  }
  if (
    lower.includes("image too small to scale") ||
    lower.includes("line cannot be recognized")
  ) {
    const firstLine = trimmed.split("\n").find((l) => l.trim())?.trim() ?? trimmed;
    const short =
      firstLine.length > 220 ? `${firstLine.slice(0, 217)}…` : firstLine;
    return `OCR could not read part of a document (${short}). Other files in the batch can still finish.`;
  }
  return trimmed || "Parse engine failed to start (unknown error).";
}

/** Parse one JSON line from sidecar stdout; returns null for blank or invalid lines. */
export function parseSidecarLine(line: string): ParseEvent | null {
  if (!line.trim()) return null;
  try {
    return JSON.parse(line) as ParseEvent;
  } catch {
    return null;
  }
}

function handleSidecarLine(
  line: string,
  onEvent: (event: ParseEvent) => void,
  onBatchActivity: () => void,
  finish: (fn: () => void) => void,
  resolve: () => void
): void {
  const event = parseSidecarLine(line);
  if (!event) {
    if (line.trim()) {
      console.error("Failed to parse sidecar output:", line);
    }
    return;
  }
  onEvent(event);
  if (
    event.type === "start" ||
    event.type === "progress" ||
    event.type === "token_savings"
  ) {
    onBatchActivity();
  }
  if (event.type === "done") {
    finish(() => resolve());
  }
}

export function runParse(
  config: ParseConfig,
  onEvent: (event: ParseEvent) => void
): ParseRunHandle {
  let child: Child | null = null;
  let stderrTail = "";
  let stdoutBuffer = "";

  const promise = new Promise<void>(async (resolve, reject) => {
    let settled = false;
    let sawBatchActivity = false;
    const finish = (fn: () => void) => {
      if (!settled) {
        settled = true;
        fn();
      }
    };
    const markActivity = () => {
      sawBatchActivity = true;
    };

    try {
      const command = Command.sidecar("binaries/parsekit-sidecar");

      command.on("error", (error) => {
        const raw = String(error);
        console.error("[sidecar spawn error]", raw);
        const message = friendlySidecarMessage(raw);
        onEvent({ type: "error", message });
        finish(() => reject(new Error(message)));
      });

      command.stdout.on("data", (chunk) => {
        stdoutBuffer += chunk;
        const parts = stdoutBuffer.split("\n");
        stdoutBuffer = parts.pop() ?? "";
        for (const line of parts) {
          handleSidecarLine(line, onEvent, markActivity, finish, resolve);
        }
      });

      command.stderr.on("data", (line) => {
        stderrTail = `${stderrTail}${line}`.slice(-2000);
        console.warn("[sidecar stderr]", line);
      });

      command.on("close", (data) => {
        if (settled) return;
        if (stdoutBuffer.trim()) {
          handleSidecarLine(stdoutBuffer, onEvent, markActivity, finish, resolve);
          stdoutBuffer = "";
        }
        if (settled) return;

        const stderr = stderrTail.trim();
        const cleanExit = data.code === 0 || data.code == null;

        // Only recover a missing `done` line when the engine actually ran.
        if (cleanExit && !stderr && sawBatchActivity) {
          onEvent({ type: "done", parsed: 0, skipped: 0, errors: 0 });
          finish(() => resolve());
          return;
        }

        const exitHint = !sawBatchActivity
          ? "Parse engine exited before starting. Quit ParseKit, reinstall from the latest DMG, then try again."
          : data.code != null && data.code !== 0
            ? `Sidecar exited with code ${data.code}`
            : "Parse engine stopped before the batch finished.";
        const message = friendlySidecarMessage(stderr || exitHint);
        onEvent({ type: "error", message });
        finish(() => reject(new Error(message)));
      });

      child = await command.spawn();
      const payload = JSON.stringify(config) + "\n";
      await child.write(payload);
    } catch (err) {
      const raw = String(err);
      console.error("[sidecar run error]", raw);
      const message = friendlySidecarMessage(raw);
      onEvent({ type: "error", message });
      finish(() => reject(new Error(message)));
    }
  });

  return {
    promise,
    cancel: () => {
      void child?.kill();
      child = null;
    },
  };
}