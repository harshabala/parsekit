<script lang="ts">
  import { OCR_LANGUAGES, type OcrLanguageCode } from "../lib/ocrLanguages";
  import { t } from "../lib/i18n.svelte";

  let {
    value,
    onChange,
    disabled = false,
  }: {
    value: OcrLanguageCode;
    onChange: (code: OcrLanguageCode) => void;
    disabled?: boolean;
  } = $props();
</script>

<label class="ocr-language-field">
  <span class="ocr-language-label">{t("settings.ocrLanguageLabel")}</span>
  <select
    class="ocr-language-select"
    {disabled}
    value={value}
    onchange={(e) => onChange(e.currentTarget.value as OcrLanguageCode)}
    aria-label={t("settings.ocrLanguageLabel")}
  >
    {#each OCR_LANGUAGES as option}
      <option value={option.code}>{option.label}</option>
    {/each}
  </select>
</label>