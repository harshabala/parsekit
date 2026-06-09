<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import type { ThemeMode } from "../lib/types";

  let { value, onChange }: { value: ThemeMode; onChange: (mode: ThemeMode) => void } = $props();

  const options: { labelKey: string; value: ThemeMode }[] = [
    { labelKey: "theme.light", value: "light" },
    { labelKey: "theme.dark", value: "dark" },
    { labelKey: "theme.system", value: "system" },
  ];
</script>

<div class="segmented-control theme-selector" role="group" aria-label={t("theme.aria")}>
  {#each options as option}
    <button
      type="button"
      class="segment"
      class:active={value === option.value}
      aria-pressed={value === option.value}
      onclick={() => onChange(option.value)}
    >
      {t(option.labelKey)}
    </button>
  {/each}
</div>