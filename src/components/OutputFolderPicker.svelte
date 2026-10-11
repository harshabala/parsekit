<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { pickOutputFolder } from "../lib/picker";
  import { truncatePath } from "../lib/pathDisplay";
  import FolderIcon from "phosphor-svelte/lib/FolderIcon";

  let { value, onSelect }: { value: string; onSelect: (path: string) => void } = $props();

  const displayPath = $derived(value ? truncatePath(value, 36) : t("config.downloads"));

  async function pick() {
    const selected = await pickOutputFolder();
    if (selected) {
      onSelect(selected);
    }
  }
</script>

<div class="config-control-row">
  <div class="config-control-label">
    <FolderIcon size={16} weight="regular" aria-hidden="true" />
    <span>{t("config.outputFolder")}</span>
  </div>
  <div class="config-control-fields">
    <button
      type="button"
      class="output-folder-path-field"
      title={value || t("config.downloads")}
      aria-label="{t('config.outputFolder')}: {value || t('config.downloads')}"
      onclick={pick}
    >
      <span class="output-folder-path-text">{displayPath}</span>
    </button>
    <button type="button" class="secondary output-folder-change-btn" onclick={pick}>
      {t("config.change")}
    </button>
  </div>
</div>
