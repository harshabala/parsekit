<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { t } from "../lib/i18n.svelte";
  import {
    hintFadeOut,
    MOTION_ENTER_MS,
    MOTION_ENTER_Y,
    easingDecelerate,
  } from "../lib/motion";
  import {
    approximateChatGptMessages,
    formatTokenCount,
    tokensSavedToday,
    type TokenStats,
  } from "../lib/tokenStats";
  import type { ConvertDestination } from "../lib/activation";
  import type { BatchTokenSavings } from "../lib/progress";

  let {
    batch,
    stats,
    successCount,
    failedCount,
    successRateLabel,
    showDestinationPrompt = false,
    onOpenOutput,
    onConvertMore,
    onDestination,
  }: {
    batch: BatchTokenSavings;
    stats: TokenStats | null;
    successCount: number;
    failedCount: number;
    successRateLabel: string;
    showDestinationPrompt?: boolean;
    onOpenOutput: () => void;
    onConvertMore: () => void;
    onDestination: (value: ConvertDestination) => void;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const fadeOut = $derived(hintFadeOut(reducedMotion));
  // Enter from current offset with ease-out — not scale(0); respects reduced motion
  const enterTransition = $derived(
    reducedMotion
      ? { y: 0, duration: 0 }
      : {
          y: MOTION_ENTER_Y,
          duration: MOTION_ENTER_MS,
          easing: easingDecelerate,
        },
  );

  const batchTokens = $derived(batch.tokensSaved);
  const todayTokens = $derived(stats ? tokensSavedToday(stats) : 0);
  const lifetimeTokens = $derived(stats?.total_tokens_saved ?? 0);
  const lifetimeFiles = $derived(stats?.total_files_converted ?? 0);
  const chatApprox = $derived(approximateChatGptMessages(lifetimeTokens));
  const partialFail = $derived(failedCount > 0 && successCount > 0);
  const allFailed = $derived(failedCount > 0 && successCount === 0);
  const tone = $derived(
    allFailed ? "fail" : partialFail ? "warn" : "success",
  );
</script>

<section
  class="batch-scoreboard"
  class:tone-success={tone === "success"}
  class:tone-warn={tone === "warn"}
  class:tone-fail={tone === "fail"}
  aria-label={t("scoreboard.aria")}
  in:fly={enterTransition}
  out:fade={fadeOut}
>
  <header class="batch-scoreboard-header">
    <h2 class="batch-scoreboard-title">
      {#if allFailed}
        {t("scoreboard.titleFailed")}
      {:else if partialFail}
        {t("scoreboard.titlePartial")}
      {:else}
        {t("scoreboard.titleSuccess")}
      {/if}
    </h2>
  </header>

  <!-- Hero: one number the eye lands on (achievement + purpose) -->
  <div class="batch-scoreboard-hero">
    <span class="batch-scoreboard-hero-value" aria-live="polite">
      {formatTokenCount(batchTokens)}
    </span>
    <span class="batch-scoreboard-hero-label">{t("scoreboard.thisBatch")}</span>
    {#if batchTokens > 0}
      <p class="batch-scoreboard-hero-sub">
        {t("scoreboard.chatApprox", {
          count: formatTokenCount(approximateChatGptMessages(batchTokens)),
        })}
      </p>
    {/if}
  </div>

  <!-- Secondary metrics: grouped platter, not competing with hero -->
  <dl class="batch-scoreboard-stats">
    <div class="batch-scoreboard-stat">
      <dt>{t("scoreboard.today")}</dt>
      <dd>{formatTokenCount(todayTokens)}</dd>
    </div>
    <div class="batch-scoreboard-stat">
      <dt>{t("scoreboard.allTime")}</dt>
      <dd>{formatTokenCount(lifetimeTokens)}</dd>
    </div>
    <div class="batch-scoreboard-stat">
      <dt>{t("scoreboard.filesAllTime")}</dt>
      <dd>{formatTokenCount(lifetimeFiles)}</dd>
    </div>
    <div class="batch-scoreboard-stat">
      <dt>{t("scoreboard.successRate")}</dt>
      <dd title={t("scoreboard.successRateHint")}>{successRateLabel}</dd>
    </div>
  </dl>

  {#if partialFail || allFailed}
    <p class="batch-scoreboard-fail-hint" role="status">
      {t("scoreboard.failHint", {
        ok: successCount,
        fail: failedCount,
      })}
    </p>
  {/if}

  <!-- Primary path first; secondary one step down (simplicity) -->
  <div class="batch-scoreboard-actions">
    <button type="button" class="run-parse-btn" onclick={onOpenOutput}>
      {t("run.openOutput")}
    </button>
    <button type="button" class="secondary" onclick={onConvertMore}>
      {t("scoreboard.convertMore")}
    </button>
  </div>

  {#if showDestinationPrompt && successCount > 0}
    <div class="batch-scoreboard-dest" role="group" aria-label={t("scoreboard.destAria")}>
      <p class="batch-scoreboard-dest-title">{t("scoreboard.destTitle")}</p>
      <div class="batch-scoreboard-dest-btns">
        {#each [
          ["claude", "Claude"],
          ["chatgpt", "ChatGPT"],
          ["gemini", "Gemini"],
          ["obsidian", "Obsidian"],
        ] as [id, label]}
          <button
            type="button"
            class="batch-scoreboard-chip"
            onclick={() => onDestination(id as ConvertDestination)}
          >
            {label}
          </button>
        {/each}
        <button
          type="button"
          class="batch-scoreboard-chip"
          onclick={() => onDestination("other")}
        >
          {t("scoreboard.destOther")}
        </button>
        <button
          type="button"
          class="batch-scoreboard-chip batch-scoreboard-chip--quiet"
          onclick={() => onDestination("skipped")}
        >
          {t("scoreboard.destSkip")}
        </button>
      </div>
    </div>
  {/if}

  <!-- Progressive disclosure: method + privacy not competing with the number -->
  <details class="batch-scoreboard-details">
    <summary>{t("scoreboard.aboutEstimate")}</summary>
    <p class="batch-scoreboard-method">{t("scoreboard.method")}</p>
    <p class="batch-scoreboard-privacy">{t("scoreboard.privacy")}</p>
    {#if lifetimeTokens > 0}
      <p class="batch-scoreboard-lifetime-note">
        {t("tokenSavings.chatGptApprox", { count: formatTokenCount(chatApprox) })}
        <span class="token-savings-approx-label">{t("tokenSavings.approximateLabel")}</span>
      </p>
    {/if}
  </details>
</section>

<style>
  .batch-scoreboard {
    margin-top: 12px;
    padding: 16px 14px 12px;
    border-radius: 12px;
    /* Border OR soft elevation — not both (ghost-card ban) */
    border: 1px solid var(--border-color);
    background: var(--glass-bg);
    box-shadow: none;
  }

  .batch-scoreboard.tone-success {
    border-color: color-mix(in srgb, var(--status-success) 35%, var(--border-color));
  }
  .batch-scoreboard.tone-warn {
    border-color: color-mix(in srgb, var(--status-warning) 40%, var(--border-color));
  }
  .batch-scoreboard.tone-fail {
    border-color: color-mix(in srgb, var(--status-error) 40%, var(--border-color));
  }

  .batch-scoreboard-header {
    margin-bottom: 10px;
  }
  .batch-scoreboard-title {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
    letter-spacing: -0.015em;
    line-height: 1.2;
    font-family: var(--font-sans, system-ui, sans-serif);
  }

  .batch-scoreboard-hero {
    text-align: center;
    margin: 4px 0 14px;
    padding: 8px 0 4px;
  }
  .batch-scoreboard-hero-value {
    display: block;
    font-size: 2.1rem;
    font-weight: 650;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.03em;
    line-height: 1.05;
    color: var(--text-primary);
  }
  .batch-scoreboard-hero-label {
    display: block;
    margin-top: 4px;
    font-size: 0.72rem;
    letter-spacing: 0.01em;
    color: var(--text-secondary);
    line-height: 1.3;
  }
  .batch-scoreboard-hero-sub {
    margin: 6px 0 0;
    font-size: 0.72rem;
    color: var(--text-secondary);
    line-height: 1.35;
  }

  .batch-scoreboard-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    margin: 0 0 12px;
  }
  .batch-scoreboard-stat {
    padding: 8px 10px;
    border-radius: 8px;
    background: var(--secondary-bg, color-mix(in srgb, var(--text-color) 5%, transparent));
  }
  .batch-scoreboard-stats dt {
    font-size: 0.65rem;
    letter-spacing: 0.02em;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.2;
  }
  .batch-scoreboard-stats dd {
    margin: 3px 0 0;
    font-size: 0.95rem;
    font-weight: 600;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.01em;
    line-height: 1.15;
    color: var(--text-primary);
  }

  .batch-scoreboard-fail-hint {
    margin: 0 0 10px;
    padding: 8px 10px;
    border-radius: 8px;
    font-size: 0.72rem;
    line-height: 1.4;
    color: var(--text-primary);
    background: color-mix(in srgb, var(--status-warning) 14%, var(--secondary-bg));
  }
  .tone-fail .batch-scoreboard-fail-hint {
    background: color-mix(in srgb, var(--status-error) 12%, var(--secondary-bg));
  }

  .batch-scoreboard-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  /* Primary already gets global :active scale(0.97) — keep hierarchy clear */
  .batch-scoreboard-actions .run-parse-btn {
    flex: 1 1 auto;
    min-width: 8rem;
  }

  .batch-scoreboard-dest {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px solid var(--border-color);
  }
  .batch-scoreboard-dest-title {
    margin: 0 0 8px;
    font-size: 0.78rem;
    font-weight: 500;
    letter-spacing: -0.01em;
    opacity: 0.85;
  }
  .batch-scoreboard-dest-btns {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .batch-scoreboard-chip {
    margin: 0;
    padding: 6px 12px;
    min-height: 32px;
    border-radius: 999px;
    border: 1px solid var(--border-color);
    background: transparent;
    color: var(--text-primary);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: -0.01em;
    cursor: pointer;
    transition:
      transform 100ms var(--easing-decelerate, ease-out),
      background-color 120ms var(--easing-standard, ease),
      border-color 120ms var(--easing-standard, ease);
  }
  .batch-scoreboard-chip:hover {
    background: var(--secondary-bg);
  }
  .batch-scoreboard-chip:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }
  .batch-scoreboard-chip:active {
    transform: scale(0.97); /* press feedback on pointer-down path */
  }
  .batch-scoreboard-chip--quiet {
    opacity: 0.65;
  }

  .batch-scoreboard-details {
    margin-top: 12px;
    font-size: 0.7rem;
    line-height: 1.4;
    opacity: 0.72;
  }
  .batch-scoreboard-details summary {
    cursor: pointer;
    font-weight: 500;
    letter-spacing: 0.01em;
    list-style: none;
    user-select: none;
  }
  .batch-scoreboard-details summary::-webkit-details-marker {
    display: none;
  }
  .batch-scoreboard-details summary::before {
    content: "› ";
    display: inline-block;
    transition: transform 120ms ease-out;
  }
  .batch-scoreboard-details[open] summary::before {
    transform: rotate(90deg);
  }
  .batch-scoreboard-method,
  .batch-scoreboard-privacy,
  .batch-scoreboard-lifetime-note {
    margin: 8px 0 0;
  }
  .batch-scoreboard-privacy {
    opacity: 0.9;
  }

  @media (prefers-reduced-motion: reduce) {
    .batch-scoreboard-chip,
    .batch-scoreboard-details summary::before {
      transition: none;
    }
  }
  @media (prefers-reduced-transparency: reduce) {
    .batch-scoreboard {
      box-shadow: none;
    }
    .batch-scoreboard-stat {
      background: var(--bg-elevated, #f4f4f4);
    }
  }
</style>
