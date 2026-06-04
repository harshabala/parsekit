<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { t } from "../lib/i18n.svelte";
  import { iconFadeIn, iconFadeOut } from "../lib/motion";
  import { resolvePrimaryParsingId } from "../lib/progress";
  import type { FileProgress } from "../lib/types";

  const reducedMotion = $derived(prefersReducedMotion.current);
  const iconFadeInParams = $derived(iconFadeIn(reducedMotion));
  const iconFadeOutParams = $derived(iconFadeOut(reducedMotion));

  let {
    files,
    total,
    isParsing,
    lastParsingId = null,
  }: {
    files: FileProgress[];
    total: number;
    isParsing: boolean;
    lastParsingId?: string | null;
  } = $props();

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
      row.scrollIntoView({ block: "nearest", behavior: "smooth" });
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
      <span class="progress-heading">{isParsing ? t("progress.parsing") : t("progress.complete")}</span>
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
      <p class="progress-now" aria-live="polite">
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
        <span class="progress-stat-err">{t("progress.failed", { count: errorCount })}</span>
      {/if}
      {#if skippedCount > 0}
        <span class="progress-stat-sep" aria-hidden="true">·</span>
        <span>{t("progress.skipped", { count: skippedCount })}</span>
      {/if}
    </div>

    {#if finishedCount > 0}
      <div class="progress-summary">
        {t("progress.summary", {
          parsed: completedCount,
          errors: errorCount,
          total,
        })}
      </div>
    {/if}

    <div class="file-list" bind:this={listEl} onwheel={onListWheel}>
      {#each files as file (file.id)}
        <div
          class="file-row"
          data-file-id={file.id}
          class:file-row-active={file.status === "parsing"}
          class:file-row-error={file.status === "error"}
          class:file-row-done={file.status === "done"}
        >
          <div class="file-row-main">
            <span class="file-name" title={file.name}>{file.name}</span>
            {#if file.status === "error" && file.error}
              <span class="file-error" title={file.error}>{file.error}</span>
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