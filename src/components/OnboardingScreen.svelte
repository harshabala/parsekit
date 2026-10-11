<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import InfoIcon from "phosphor-svelte/lib/InfoIcon";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import { t } from "../lib/i18n.svelte";
  import { hintFadeIn, hintFadeOut } from "../lib/motion";
  import appIcon from "../assets/app-icon.png";

  let {
    showInstallHint,
    outputDirSet,
    onComplete,
    onPickOutput,
  }: {
    showInstallHint: boolean;
    outputDirSet: boolean;
    onComplete: () => void;
    onPickOutput: () => void;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));

  const totalSteps = $derived(showInstallHint ? 2 : 1);
  let step = $state(1);

  let gatekeeperCopied = $state(false);
  let gatekeeperCopyError = $state<string | null>(null);
  let gatekeeperCommand = $state("");
  let installing = $state(false);
  let installError = $state<string | null>(null);

  const step1Done = $derived(outputDirSet);
  const step2Done = $derived(false);

  $effect(() => {
    void loadGatekeeperCommand();
  });

  async function loadGatekeeperCommand() {
    try {
      gatekeeperCommand = await invoke<string>("gatekeeper_fix_command");
    } catch {
      gatekeeperCommand =
        "xattr -cr /Applications/ParseKit.app && xattr -d com.apple.FinderInfo /Applications/ParseKit.app";
    }
  }

  async function copyGatekeeperCommand() {
    gatekeeperCopyError = null;
    try {
      await invoke("copy_text_to_clipboard", { text: gatekeeperCommand });
      try {
        await invoke("trigger_haptic");
      } catch {
        /* optional */
      }
      gatekeeperCopied = true;
      setTimeout(() => {
        gatekeeperCopied = false;
      }, 2500);
    } catch (e) {
      gatekeeperCopied = false;
      gatekeeperCopyError =
        e instanceof Error ? e.message : String(e) || t("gatekeeper.copyFailed");
    }
  }

  async function installParseKit() {
    installError = null;
    installing = true;
    try {
      await invoke("install_and_relaunch");
    } catch (e) {
      installError =
        e instanceof Error ? e.message : String(e) || t("onboarding.installFailed");
      installing = false;
    }
  }
</script>

<div class="onboarding-root">
  <div
    class="onboarding-progress"
    role="progressbar"
    aria-valuenow={step}
    aria-valuemin={1}
    aria-valuemax={totalSteps}
    aria-label={t("onboarding.progress", { current: step, total: totalSteps })}
  >
    <span>{t("onboarding.progress", { current: step, total: totalSteps })}</span>
    <div class="onboarding-progress-dots" aria-hidden="true">
      {#each Array(totalSteps) as _, i}
        {@const n = i + 1}
        {#if i > 0}<span class="onboarding-progress-line"></span>{/if}
        <span class="onboarding-progress-dot" class:active={n === step}></span>
      {/each}
    </div>
  </div>

  <div class="onboarding-content">
    {#if step === 1 && showInstallHint}
      <div
        class="onboarding-step-panel"
        in:fade={{ duration: reducedMotion ? 0 : 200 }}
        out:fade={{ duration: reducedMotion ? 0 : 120 }}
      >
        <div class="onboarding-header-group">
          <img class="onboarding-install-icon" src={appIcon} width="128" height="128" alt="" />
          <h1 class="onboarding-title">{t("onboarding.installTitle")}</h1>
          <p class="onboarding-subtitle">{t("onboarding.installSubtitle")}</p>
        </div>

        <div class="onboarding-actions">
          <button
            type="button"
            class="onboarding-btn-primary onboarding-btn-install"
            onclick={installParseKit}
            disabled={installing}
          >
            {installing ? t("onboarding.installing") : t("onboarding.installButton")}
          </button>
        </div>

        {#if installError}
          <p class="onboarding-helper onboarding-error" role="alert">{installError}</p>
        {/if}

        <p class="onboarding-helper">{t("onboarding.installHelper")}</p>

        <div class="onboarding-code-card" role="region" aria-label={t("onboarding.terminalCommand")}>
          <div class="onboarding-code-toolbar">
            <div class="onboarding-traffic-lights" aria-hidden="true">
              <span></span><span></span><span></span>
            </div>
            <button
              type="button"
              class="onboarding-code-copy"
              onclick={copyGatekeeperCommand}
            >
              {#key gatekeeperCopied}
                <span in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
                  {gatekeeperCopied ? t("gatekeeper.copied") : t("onboarding.copyCommand")}
                </span>
              {/key}
            </button>
          </div>
          <div class="onboarding-code-body">
            <div><span class="prompt">$ </span><span class="cmd">{gatekeeperCommand}</span></div>
          </div>
        </div>
        {#if gatekeeperCopyError}
          <p class="onboarding-helper onboarding-error" role="alert">{gatekeeperCopyError}</p>
        {/if}
      </div>
    {:else}
      <div
        class="onboarding-step-panel"
        in:fade={{ duration: reducedMotion ? 0 : 200 }}
        out:fade={{ duration: reducedMotion ? 0 : 120 }}
      >
        <div class="onboarding-header-group">
          <h1 class="onboarding-title">{t("onboarding.getStartedTitle")}</h1>
          <p class="onboarding-subtitle">{t("onboarding.getStartedSubtitle")}</p>
        </div>

        <ol class="onboarding-steps-list">
          <li class:done={step1Done}>
            <span class="onboarding-step-badge" aria-hidden="true">
              {#if step1Done}
                <CheckIcon size={12} weight="bold" />
              {:else}
                1
              {/if}
            </span>
            <div class="onboarding-step-content">
              <span>{t("onboarding.stepOutput")}</span>
              {#if !step1Done}
                <button type="button" class="onboarding-btn-secondary" onclick={onPickOutput}>
                  {t("onboarding.chooseOutput")}
                </button>
              {/if}
            </div>
          </li>
          <li class:done={step2Done}>
            <span class="onboarding-step-badge" aria-hidden="true">
              {#if step2Done}
                <CheckIcon size={12} weight="bold" />
              {:else}
                2
              {/if}
            </span>
            <span>{t("onboarding.stepFiles")}</span>
          </li>
          <li>
            <span class="onboarding-step-badge">3</span>
            <span>{t("onboarding.stepRun")}</span>
          </li>
        </ol>

        <div class="onboarding-info-card" style="margin-top: 12px">
          <div>
            <p class="onboarding-info-title">{t("onboarding.firstConvertTitle")}</p>
            <p class="onboarding-info-body">{t("onboarding.firstConvertBody")}</p>
          </div>
        </div>

        <div class="onboarding-info-card">
          <span class="onboarding-info-icon" aria-hidden="true">
            <InfoIcon size={20} weight="duotone" />
          </span>
          <div>
            <p class="onboarding-info-title">{t("onboarding.fileSupportTitle")}</p>
            <p class="onboarding-info-body">{t("onboarding.fileSupportBody")}</p>
          </div>
        </div>

        <div class="onboarding-actions">
          <button type="button" class="onboarding-btn-primary" onclick={onComplete}>
            {t("onboarding.continue")}
          </button>
          <p class="onboarding-tray-note">{t("onboarding.trayHint")}</p>
        </div>
      </div>
    {/if}
  </div>
</div>