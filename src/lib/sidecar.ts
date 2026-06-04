import { Command, type Child } from "@tauri-apps/plugin-shell";

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
  type: "start" | "progress" | "done" | "error";
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
}

export interface ParseRunHandle {
  promise: Promise<void>;
  cancel: () => void;
}

function friendlySidecarMessage(raw: string): string {
  const lower = raw.toLowerCase();
  if (lower.includes("imagemagick")) {
    return "ImageMagick is required for images. Install with: brew install imagemagick";
  }
  if (lower.includes("libreoffice")) {
    return "LibreOffice is required for Office documents. Install with: brew install --cask libreoffice";
  }
  if (
    lower.includes("not allowed") ||
    lower.includes("permission") ||
    lower.includes("shell:allow-spawn") ||
    lower.includes("shell:allow-stdin-write")
  ) {
    return "Parse engine permission denied. Restart the app after rebuilding (npm run tauri:dev).";
  }
  if (lower.includes("sidecar") || lower.includes("not found")) {
    return "Parse engine could not start. Rebuild the app or run npm run build:sidecar in dev.";
  }
  return raw;
}

export function runParse(
  config: ParseConfig,
  onEvent: (event: ParseEvent) => void
): ParseRunHandle {
  let child: Child | null = null;
  let stderrTail = "";

  const promise = new Promise<void>(async (resolve, reject) => {
    let settled = false;
    const finish = (fn: () => void) => {
      if (!settled) {
        settled = true;
        fn();
      }
    };

    try {
      const command = Command.sidecar("binaries/parsedock-sidecar");

      command.on("error", (error) => {
        const message = friendlySidecarMessage(String(error));
        onEvent({ type: "error", message });
        finish(() => reject(new Error(message)));
      });

      command.stdout.on("data", (line) => {
        const lines = line.split("\n").filter(Boolean);
        for (const l of lines) {
          try {
            const event: ParseEvent = JSON.parse(l);
            onEvent(event);
            if (event.type === "done") {
              finish(() => resolve());
            }
          } catch (e) {
            console.error("Failed to parse sidecar output:", l, e);
          }
        }
      });

      command.stderr.on("data", (line) => {
        stderrTail = `${stderrTail}${line}`.slice(-2000);
        console.warn("[sidecar stderr]", line);
      });

      command.on("close", (data) => {
        if (data.code !== 0 && data.code !== null) {
          const message = friendlySidecarMessage(
            stderrTail.trim() || `Sidecar exited with code ${data.code}`
          );
          onEvent({ type: "error", message });
          finish(() => reject(new Error(message)));
        }
      });

      child = await command.spawn();
      await child.write(JSON.stringify(config) + "\n");
    } catch (err) {
      const message = friendlySidecarMessage(String(err));
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