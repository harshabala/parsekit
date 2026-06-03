import { Command } from "@tauri-apps/plugin-shell";

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
  status?: string;
  path?: string;
  message?: string;
  total?: number;
  parsed?: number;
  skipped?: number;
  errors?: number;
  error?: string;
}

export function runParse(
  config: ParseConfig,
  onEvent: (event: ParseEvent) => void
): Promise<void> {
  return new Promise(async (resolve, reject) => {
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
        onEvent({ type: "error", message: String(error) });
        finish(() => reject(new Error(String(error))));
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
            console.error("Failed to parse sidecar output:", l);
          }
        }
      });

      command.stderr.on("data", (line) => {
        console.warn("[sidecar stderr]", line);
      });

      command.on("close", (data) => {
        if (data.code !== 0) {
          finish(() => reject(new Error(`Sidecar exited with code ${data.code}`)));
        }
      });

      const child = await command.spawn();
      await child.write(JSON.stringify(config) + "\n");
    } catch (err) {
      finish(() => reject(err));
    }
  });
}
