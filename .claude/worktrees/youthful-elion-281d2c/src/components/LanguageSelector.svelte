<script lang="ts">
  import type { AppLocale } from "../lib/i18n.svelte";
  import { SUPPORTED_LOCALES, t } from "../lib/i18n.svelte";

  let { value, onChange }: { value: AppLocale; onChange: (code: AppLocale) => void } = $props();
</script>

<div class="segmented-control language-selector" role="group" aria-label={t("settings.appLanguageTitle")}>
  {#each SUPPORTED_LOCALES as code}
    <button
      type="button"
      class="segment"
      class:active={value === code}
      aria-pressed={value === code}
      onclick={() => onChange(code)}
    >
      {t(`language.${code}`)}
    </button>
  {/each}
</div>