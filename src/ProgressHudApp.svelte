<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getSetting } from "./lib/store";
  import { initLocale, localeFromLegacyOcr, normalizeLocale, type AppLocale } from "./lib/i18n.svelte";
  import { applyTheme, DEFAULT_THEME, normalizeThemeMode } from "./lib/theme";

  import {
    createEmptyHudState,
    PROGRESS_HUD_SYNC_EVENT,
    type ProgressHudState,
  } from "./lib/progressHud";
  import ProgressHud from "./components/ProgressHud.svelte";
  import "./index.css";

  let hudState = $state<ProgressHudState>(createEmptyHudState());

  onMount(() => {
    let unlisten: (() => void) | undefined;

    void (async () => {
      const theme = normalizeThemeMode(await getSetting("theme", DEFAULT_THEME));
      applyTheme(theme);

      const savedLocale = await getSetting<AppLocale | null>("locale", null);
      const resolvedLocale = savedLocale
        ? normalizeLocale(savedLocale)
        : localeFromLegacyOcr(await getSetting("ocrLanguage", "eng"));
      initLocale(resolvedLocale);

      unlisten = await listen<ProgressHudState>(PROGRESS_HUD_SYNC_EVENT, (event) => {
        if (event.payload) {
          hudState = event.payload;
        }
      });
    })();

    return () => {
      unlisten?.();
    };
  });
</script>

<div class="progress-hud-shell">
  <ProgressHud {hudState} />
</div>