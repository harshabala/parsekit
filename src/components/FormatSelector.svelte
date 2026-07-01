<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import type { OutputFormat } from "../lib/types";

  let { value, onChange }: { value: OutputFormat; onChange: (format: OutputFormat) => void } = $props();

  const formats: { label: string; value: OutputFormat }[] = [
    { label: "TXT", value: "txt" },
    { label: "MD", value: "md" },
    { label: "JSON", value: "json" },
  ];
</script>

<div class="segmented-control" role="group" aria-label={t("config.format")}>
  {#each formats as fmt}
    <button
      type="button"
      class="segment"
      class:active={value === fmt.value}
      aria-pressed={value === fmt.value}
      onclick={() => onChange(fmt.value)}
    >
      {fmt.label}
    </button>
  {/each}
</div>