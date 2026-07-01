<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { hintFadeIn, hintFadeOut } from "../lib/motion";
  import { t } from "../lib/i18n.svelte";
  import { isConverterDependencyError } from "../lib/converterErrors";
  import {
    hudErrorCount,
    hudIsComplete,
    hudProgressPercent,
    hudSuccessCount,
    PROGRESS_HUD_DISMISS_MS,
    type ProgressHudState,
  } from "../lib/progressHud";
  import { formatTokenCount } from "../lib/tokenStats";
  import type { FileProgress } from "../lib/types";

  let {
    hudState,
  }: {
    hudState: ProgressHudState;
  } = $props();

  let selectedErrorId = $state<string | null>(null);
  let dismissTimer: ReturnType<typeof setTimeout> | undefined;

  const reducedMotion = $derived(prefersReducedMotion.current);
  const fadeInParams = $derived(hintFadeIn(reducedMotion));
  const fadeOutParams = $derived(hintFadeOut(reducedMotion));

  const total = $derived(hudState.total || hudState.files.length);
  const finished = $derived(
    hudState.files.filter(
      (f) => f.status === "done" || f.status === "error" || f.status === "skipped",
    ).length,
  );
  const successCount = $derived(hudSuccessCount(hudState.files));
  const errorCount = $derived(hudErrorCount(hudState.files));
  const progressPercent = $derived(hudProgressPercent(hudState.files, total));
  const isComplete = $derived(hudIsComplete(hudState.files, total, hudState.isParsing));
  const failedFiles = $derived(hudState.files.filter((f) => f.status === "error"));
  const selectedError = $derived(
    selectedErrorId
      ? (failedFiles.find((f) => f.id === selectedErrorId) ?? null)
      : null,
  );
  const tokensSaved = $derived(hudState.batchTokenSavings.tokensSaved);

  $effect(() => {
    if (!isComplete) {
      clearTimeout(dismissTimer);
      return;
    }
    clearTimeout(dismissTimer);
    dismissTimer = setTimeout(() => {
      void invoke("hide_progress_hud").catch(() => {});
    }, PROGRESS_HUD_DISMISS_MS);
    return () => clearTimeout(dismissTimer);
  });

  $effect(() => {
    if (errorCount === 0) {
      selectedErrorId = null;
    }
  });

  function toggleFailedDetail(file: FileProgress) {
    selectedErrorId = selectedErrorId === file.id ? null : file.id;
  }

  async function openFileSupport() {
    try {
      await emit("hud-open-file-support");
      await invoke("show_main_window");
    } catch {
      /* dev / web */
    }
  }
</script>

<div class="progress-hud" role="status" aria-live="polite">
  <div class="progress-hud-header">
    <span class="progress-hud-title">
      {#if isComplete}
        {t("progressHud.complete")}
      {:else}
        {t("progressHud.parsing")}
      {/if}
    </span>
    <span class="progress-hud-fraction">
      {t("progress.ofTotal", { done: finished, total })}
    </span>
  </div>

  <div
    class="progress-bar progress-hud-bar"
    role="progressbar"
    aria-valuenow={progressPercent}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label={t("progressHud.parsing")}
  >
    <div
      class="progress-fill"
      class:progress-fill-complete={isComplete}
      style="transform: scaleX({progressPercent / 100})"
    ></div>
  </div>

  <div class="progress-hud-stats">
    <span class="progress-hud-stat-ok">
      {t("progressHud.success", { count: successCount })}
    </span>
    {#if errorCount > 0}
      <button
        type="button"
        class="progress-hud-stat-err"
        onclick={() => {
          if (failedFiles[0]) toggleFailedDetail(failedFiles[0]);
        }}
      >
        {t("progressHud.failed", { count: errorCount })}
      </button>
    {/if}
  </div>

  {#if isComplete}
    <div class="progress-hud-complete" in:fade={fadeInParams} out:fade={fadeOutParams}>
      {#if tokensSaved > 0}
        <span class="progress-hud-tokens">
          {t("progressHud.tokensSaved", {
            count: formatTokenCount(tokensSaved),
          })}
        </span>
      {/if}
      <span class="progress-hud-summary">
        {t("progress.summary", {
          parsed: successCount,
          errors: errorCount,
          total,
        })}
      </span>
    </div>
  {/if}

  {#if errorCount > 0 && selectedError}
    <div
      class="progress-hud-error-detail"
      role="region"
      aria-label={selectedError.name}
      in:fade={fadeInParams}
      out:fade={fadeOutParams}
    >
      <button
        type="button"
        class="progress-hud-error-file"
        onclick={() => toggleFailedDetail(selectedError)}
        title={selectedError.error ?? ""}
      >
        {selectedError.name}
      </button>
      {#if selectedError.error}
        <p class="progress-hud-error-msg">{selectedError.error}</p>
      {/if}
      {#if isConverterDependencyError(selectedError.error ?? "")}
        <button type="button" class="progress-hud-error-link" onclick={openFileSupport}>
          {t("errors.openFileSupport")}
        </button>
      {/if}
      {#if failedFiles.length > 1}
        <div class="progress-hud-error-nav">
          {#each failedFiles as file (file.id)}
            <button
              type="button"
              class="progress-hud-error-chip"
              class:active={file.id === selectedErrorId}
              onclick={() => toggleFailedDetail(file)}
            >
              {file.name}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>