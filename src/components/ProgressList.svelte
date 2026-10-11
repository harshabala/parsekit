<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { t } from "../lib/i18n.svelte";
  import {
    iconFadeIn,
    iconFadeOut,
    hintFadeIn,
    hintFadeOut,
    rowFlyIn,
    rowFlyOut,
    sectionFlyInMaybeLight,
    sectionFlyOutMaybeLight,
    MOTION_ROW_STAGGER_CAP,
  } from "../lib/motion";
  import { isConverterDependencyError } from "../lib/converterErrors";
  import { resolvePrimaryParsingId } from "../lib/progress";
  import type { FileProgress } from "../lib/types";

  let {
    files,
    total,
    isParsing,
    lastParsingId = null,
    onOpenFileSupport,
    /** Hotkey/background batches: skip fly-in Y, shorter duration (Emil). */
    lightMotion = false,
  }: {
    files: FileProgress[];
    total: number;
    isParsing: boolean;
    lastParsingId?: string | null;
    onOpenFileSupport?: () => void;
    lightMotion?: boolean;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const iconFadeInParams = $derived(iconFadeIn(reducedMotion));
  const iconFadeOutParams = $derived(iconFadeOut(reducedMotion));
  const sectionFlyInParams = $derived(
    sectionFlyInMaybeLight(reducedMotion, lightMotion),
  );
  const sectionFlyOutParams = $derived(
    sectionFlyOutMaybeLight(reducedMotion, lightMotion),
  );
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));

  let prevFileCount = $state(0);
  let rowStaggerActive = $state(false);

  let listEl = $state<HTMLDivElement | null>(null);
  let userScrolledUntil = $state(0);

  let completedCount = $derived(files.filter((f) => f.status === "done").length);
  let errorCount = $derived(files.filter((f) => f.status === "error").length);
  let skippedCount = $derived(files.filter((f) => f.status === "skipped").length);
  let parsingCount = $derived(files.filter((f) => f.status === "parsing").length);
  let pendingCount = $derived(files.filter((f) => f.status === "pending").length);

  let finishedCount = $derived(completedCount + errorCount + skippedCount);
  let progressPercent = $derived(
    total > 0 ? Math.round((finishedCount / total) * 100) : 0
  );

  let primaryActiveId = $derived(resolvePrimaryParsingId(files, lastParsingId));
  let primaryActive = $derived(
    primaryActiveId ? files.find((f) => f.id === primaryActiveId) ?? null : null
  );

  let showCompletePulse = $state(false);
  let prevIsParsing = $state(false);
  let pulseTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    const justFinished = prevIsParsing && !isParsing;
    const parseComplete =
      total > 0 && (progressPercent >= 100 || finishedCount >= total);

    if (justFinished && parseComplete && !reducedMotion) {
      showCompletePulse = true;
      clearTimeout(pulseTimer);
      pulseTimer = setTimeout(() => {
        showCompletePulse = false;
      }, 400);
    }

    prevIsParsing = isParsing;

    return () => clearTimeout(pulseTimer);
  });

  $effect(() => {
    const n = files.length;
    if (prevFileCount === 0 && n > 0 && n <= MOTION_ROW_STAGGER_CAP) {
      rowStaggerActive = true;
    } else if (n === 0) {
      rowStaggerActive = false;
    }
    prevFileCount = n;
  });

  $effect(() => {
    if (!isParsing || !primaryActiveId || reducedMotion || !listEl) return;
    if (Date.now() < userScrolledUntil) return;

    const selector = `[data-file-id="${CSS.escape(primaryActiveId)}"]`;
    const row = listEl.querySelector<HTMLElement>(selector);
    if (!row) return;

    const listTop = listEl.scrollTop;
    const listBottom = listTop + listEl.clientHeight;
    const rowTop = row.offsetTop;
    const rowBottom = rowTop + row.offsetHeight;
    if (rowTop < listTop + 4 || rowBottom > listBottom - 4) {
      row.scrollIntoView({
        block: "nearest",
        behavior: reducedMotion ? "auto" : "smooth",
      });
    }
  });

  function onListWheel() {
    userScrolledUntil = Date.now() + 4000;
  }

  function statusLabel(status: FileProgress["status"]): string {
    switch (status) {
      case "parsing":
        return t("progress.statusParsing");
      case "done":
        return t("progress.statusDone");
      case "error":
        return t("progress.statusError");
      case "skipped":
        return t("progress.statusSkipped");
      default:
        return t("progress.statusWaiting");
    }
  }
</script>

<div class="section progress-section">
  <div class="section-title">{t("progress.title")}</div>
  <div class="progress-panel">
    <div class="row progress-header">
      {#key isParsing}
        <span
          class="progress-heading"
          in:fade={iconFadeInParams}
          out:fade={iconFadeOutParams}
        >
          {isParsing ? t("progress.parsing") : t("progress.complete")}
        </span>
      {/key}
      <span class="progress-fraction" aria-live="polite">
        {t("progress.ofTotal", { done: finishedCount, total })}
      </span>
    </div>

    <div
      class="progress-bar"
      role="progressbar"
      aria-valuenow={progressPercent}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={t("progress.parsing")}
    >
      <div
        class="progress-fill"
        class:progress-fill-complete={showCompletePulse}
        style="transform: scaleX({progressPercent / 100})"
      ></div>
    </div>
    <div class="progress-percent-row">
      <span class="progress-percent">{progressPercent}%</span>
    </div>

    {#if isParsing && primaryActive}
      <p
        class="progress-now"
        aria-live="polite"
        in:fly={sectionFlyInParams}
        out:fly={sectionFlyOutParams}
      >
        <span class="parse-spinner parse-spinner-inline" aria-hidden="true"></span>
        <span class="progress-now-text">
          {t("progress.nowParsing", { name: primaryActive.name })}
          {#if parsingCount > 1}
            <span class="progress-now-more">
              {t("progress.andMore", { count: parsingCount - 1 })}
            </span>
          {/if}
        </span>
      </p>
    {/if}

    <div class="progress-stats" aria-live="polite">
      <span>{t("progress.completed", { count: completedCount })}</span>
      <span class="progress-stat-sep" aria-hidden="true">·</span>
      <span class="progress-stat-active">{t("progress.inProgress", { count: parsingCount })}</span>
      <span class="progress-stat-sep" aria-hidden="true">·</span>
      <span>{t("progress.waiting", { count: pendingCount })}</span>
      {#if errorCount > 0}
        <span class="progress-stat-sep" aria-hidden="true">·</span>
        <span class="progress-stat-err" in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
          {t("progress.failed", { count: errorCount })}
        </span>
      {/if}
      {#if skippedCount > 0}
        <span class="progress-stat-sep" aria-hidden="true">·</span>
        <span in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
          {t("progress.skipped", { count: skippedCount })}
        </span>
      {/if}
    </div>

    {#if finishedCount > 0}
      <div
        class="progress-summary"
        in:fade={hintFadeInParams}
        out:fade={hintFadeOutParams}
      >
        {t("progress.summary", {
          parsed: completedCount,
          errors: errorCount,
          total,
        })}
      </div>
    {/if}

    <div class="file-list" bind:this={listEl} onwheel={onListWheel}>
      {#each files as file, index (file.id)}
        <div
          class="file-row"
          data-file-id={file.id}
          class:file-row-active={file.status === "parsing"}
          class:file-row-error={file.status === "error"}
          class:file-row-done={file.status === "done"}
          in:fly={rowFlyIn(reducedMotion, index, rowStaggerActive && !lightMotion)}
          out:fly={rowFlyOut(reducedMotion)}
        >
          <div class="file-row-main">
            <span class="file-name" title={file.name}>{file.name}</span>
            {#if file.status === "error" && file.error}
              <span
                class="file-error"
                title={file.error}
                in:fade={hintFadeInParams}
                out:fade={hintFadeOutParams}
              >{file.error}</span>
              {#if onOpenFileSupport && isConverterDependencyError(file.error)}
                <button
                  type="button"
                  class="file-error-link"
                  in:fade={hintFadeInParams}
                  out:fade={hintFadeOutParams}
                  onclick={onOpenFileSupport}
                >
                  {t("errors.openFileSupport")}
                </button>
              {/if}
            {/if}
          </div>
          <div class="file-row-status" aria-label={statusLabel(file.status)}>
            {#key file.status}
              <span
                class="file-status-badge status-{file.status}"
                in:fade={iconFadeInParams}
                out:fade={iconFadeOutParams}
              >
                {#if file.status === "parsing"}
                  <span class="parse-spinner" aria-hidden="true"></span>
                {:else if file.status === "done"}
                  <span class="status-glyph" aria-hidden="true">✓</span>
                {:else if file.status === "error"}
                  <span class="status-glyph" aria-hidden="true">✕</span>
                {:else if file.status === "skipped"}
                  <span class="status-glyph" aria-hidden="true">—</span>
                {:else}
                  <span class="status-dot" aria-hidden="true"></span>
                {/if}
                <span class="status-label">{statusLabel(file.status)}</span>
              </span>
            {/key}
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>