import { Command } from "@tauri-apps/plugin-shell";

export interface ParseConfig {
  inputDir: string;
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
    try {
      const command = Command.sidecar("binaries/parsedock-sidecar");

      command.on("error", (error) => {
        onEvent({ type: "error", message: String(error) });
        reject(new Error(String(error)));
      });

      command.stdout.on("data", (line) => {
        // stdout data may contain multiple JSON lines
        const lines = line.split("\n").filter(Boolean);
        for (const l of lines) {
          try {
            const event: ParseEvent = JSON.parse(l);
            onEvent(event);
            if (event.type === "done") {
              resolve();
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
          reject(new Error(`Sidecar exited with code ${data.code}`));
        }
      });

      const child = await command.spawn();
      // Write config as a single JSON line (readline expects \n)
      await child.write(JSON.stringify(config) + "\n");
    } catch (err) {
      reject(err);
    }
  });
}
