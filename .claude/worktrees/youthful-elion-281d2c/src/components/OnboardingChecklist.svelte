<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  let {
    outputDirSet,
    filesReady,
    onDismiss,
    onPickOutput,
    showInstallHint = false,
  }: {
    outputDirSet: boolean;
    filesReady: boolean;
    onDismiss: () => void;
    onPickOutput: () => void;
    showInstallHint?: boolean;
  } = $props();

  const step1Done = $derived(outputDirSet);
  const step2Done = $derived(filesReady);
  const step3Ready = $derived(outputDirSet && filesReady);
</script>

<div class="onboarding-card" role="region" aria-labelledby="onboarding-title">
  <div class="onboarding-header">
    <span id="onboarding-title" class="onboarding-title">{t("onboarding.title")}</span>
    <button type="button" class="onboarding-dismiss" onclick={onDismiss}>
      {t("onboarding.dismiss")}
    </button>
  </div>
  <p class="settings-hint onboarding-lead">{t("onboarding.lead")}</p>
  {#if showInstallHint}
    <p class="settings-hint onboarding-install-hint">{t("onboarding.installHint")}</p>
  {/if}
  <ol class="onboarding-steps">
    <li class:done={step1Done}>
      <span class="onboarding-step-num">1</span>
      <div class="onboarding-step-body">
        <span>{t("onboarding.stepOutput")}</span>
        {#if !step1Done}
          <button type="button" class="secondary onboarding-step-btn" onclick={onPickOutput}>
            {t("onboarding.chooseOutput")}
          </button>
        {/if}
      </div>
    </li>
    <li class:done={step2Done}>
      <span class="onboarding-step-num">2</span>
      <span>{t("onboarding.stepFiles")}</span>
    </li>
    <li class:done={step3Ready}>
      <span class="onboarding-step-num">3</span>
      <span>{t("onboarding.stepRun")}</span>
    </li>
  </ol>
  <p class="settings-hint onboarding-tray-hint">{t("onboarding.trayHint")}</p>
</div>