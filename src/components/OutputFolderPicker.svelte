<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { pickOutputFolder } from "../lib/picker";
  import { truncatePath } from "../lib/pathDisplay";

  let { value, onSelect }: { value: string; onSelect: (path: string) => void } = $props();

  const displayPath = $derived(value ? truncatePath(value, 40) : t("config.downloads"));

  async function pick() {
    const selected = await pickOutputFolder();
    if (selected) {
      onSelect(selected);
    }
  }
</script>

<div class="output-folder-row">
  <div class="output-folder-label">
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path
        d="M2 4.5C2 3.67 2.67 3 3.5 3H6l1.5 1.5H12.5c.83 0 1.5.67 1.5 1.5V12c0 .83-.67 1.5-1.5 1.5H3.5A1.5 1.5 0 0 1 2 12V4.5Z"
        stroke="currentColor"
        stroke-width="1.2"
        stroke-linejoin="round"
      />
    </svg>
    <span>{t("config.outputFolder")}</span>
  </div>
  <button
    type="button"
    class="output-folder-path-btn"
    title={value || t("config.downloads")}
    aria-label="{t('config.outputFolder')}: {value || t('config.downloads')}"
    onclick={pick}
  >
    <span class="output-folder-path-text">{displayPath}</span>
    <span class="output-folder-change-label">{t("config.change")}</span>
  </button>
</div>