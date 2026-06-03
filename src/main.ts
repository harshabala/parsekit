import { mount } from "svelte";
import App from "./App.svelte";
import {
  initLocale,
  localeFromLegacyOcr,
  normalizeLocale,
} from "./lib/i18n";
import { getSetting } from "./lib/store";
import { applyTheme, DEFAULT_THEME, normalizeThemeMode } from "./lib/theme";

void Promise.all([
  getSetting("theme", DEFAULT_THEME).then((saved) => {
    applyTheme(normalizeThemeMode(saved));
  }),
  getSetting("locale", null).then(async (saved) => {
    if (saved !== null) {
      initLocale(normalizeLocale(saved));
      return;
    }
    const legacyOcr = await getSetting("ocrLanguage", "eng");
    initLocale(localeFromLegacyOcr(String(legacyOcr)));
  }),
]);

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
