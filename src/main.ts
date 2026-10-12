import "./index.css";
import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App.svelte";
import OnboardingApp from "./OnboardingApp.svelte";
import ProgressHudApp from "./ProgressHudApp.svelte";
import { applyTheme, DEFAULT_THEME } from "./lib/theme";

function bootstrap() {
  applyTheme(DEFAULT_THEME);
  try {
    const target = document.getElementById("app");
    if (!target) throw new Error("Mount target #app not found");
    const label = getCurrentWindow().label;
    if (label === "onboarding" || label === "progress-hud") {
      document.body.dataset.window = label;
    }
    const component =
      label === "progress-hud" ? ProgressHudApp : label === "onboarding" ? OnboardingApp : App;
    const app = mount(component, { target });
    document.getElementById("boot-fallback")?.remove();
    console.info(`[ParseKit] UI mounted (${label})`);
    return app;
  } catch (error) {
    console.error("[ParseKit] UI bootstrap failed:", error);
    const fallback = document.getElementById("boot-fallback");
    if (fallback) {
      fallback.textContent = `ParseKit failed to load: ${error instanceof Error ? error.message : String(error)}`;
      fallback.setAttribute("role", "alert");
      fallback.style.cssText = "color: #c0392b; font-size: 12px;";
    }
  }
}

bootstrap();
