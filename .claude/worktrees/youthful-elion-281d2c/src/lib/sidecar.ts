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
  const trimmed = raw.trim();
  const lower = trimmed.toLowerCase();
  if (lower.includes("imagemagick")) {
    return "ImageMagick is required for images. Install with: brew install imagemagick";
  }
  if (lower.includes("libreoffice")) {
    return "LibreOffice is required for Office documents. Install with: brew install --cask libreoffice";
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
  return trimmed || "Parse engine failed to start (unknown error).";
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
      const command = Command.sidecar("binaries/parsekit-sidecar");

      command.on("error", (error) => {
        const raw = String(error);
        console.error("[sidecar spawn error]", raw);
        const message = friendlySidecarMessage(raw);
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
        if (settled) return;
        const exitHint =
          data.code != null && data.code !== 0
            ? `Sidecar exited with code ${data.code}`
            : "Parse engine stopped before the batch finished.";
        const message = friendlySidecarMessage(stderrTail.trim() || exitHint);
        onEvent({ type: "error", message });
        finish(() => reject(new Error(message)));
      });

      child = await command.spawn();
      await child.write(JSON.stringify(config) + "\n");
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