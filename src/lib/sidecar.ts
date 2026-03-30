import { Command } from "@tauri-apps/plugin-shell";
import { resolveResource } from "@tauri-apps/api/path";

export interface ParseConfig {
  inputDir: string;
  outputDir: string;
  format: "md" | "json" | "txt";
  ocrLanguage?: string;
}

export interface ParseEvent {
  type: "progress" | "done" | "error";
  file?: string;
  status?: string;
  path?: string;
  message?: string;
}

export async function runParse(config: ParseConfig, onEvent: (event: ParseEvent) => void) {
  const scriptPath = await resolveResource("sidecar/index.js");
  
  // Use 'node' as the command, passing the script path as an argument
  const command = Command.create("node", [scriptPath]);
  
  command.on("error", (error) => {
    onEvent({ type: "error", message: error });
  });

  command.stdout.on("data", (line) => {
    try {
      const event = JSON.parse(line);
      onEvent(event);
    } catch (e) {
      console.error("Failed to parse sidecar output:", line);
    }
  });

  const child = await command.spawn();
  await child.write(JSON.stringify(config));
  // Signal EOF to stdin so the script starts processing
  // In sidecar/index.js we listen for 'end' on stdin
}
