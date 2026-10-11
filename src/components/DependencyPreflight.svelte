<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { Command } from "@tauri-apps/plugin-shell";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import { takeCachedDependencies, type DepStatus } from "../lib/depsCache";
  import { t } from "../lib/i18n.svelte";
  import { depRowAriaLabel } from "../lib/depsAria";
  import {
    depsPopDelayMs,
    depsStaggerDelayMs,
    easingDecelerate,
    MOTION_DEPS_ENTER_MS,
    rowFlyOut,
  } from "../lib/motion";

  const BREW_URL = "https://brew.sh";

  /** Primary name + optional subtitle for converter rows (mockup layout). */
  const DEP_COPY: Record<string, { nameKey: string; detailKey?: string }> = {
    pdf: { nameKey: "deps.pdf" },
    libreoffice: { nameKey: "deps.libreofficeName", detailKey: "deps.libreofficeDetail" },
    imagemagick: { nameKey: "deps.imagemagickName", detailKey: "deps.imagemagickDetail" },
    tesseract: { nameKey: "deps.tesseractName", detailKey: "deps.tesseractDetail" },
  };

  const reducedMotion = $derived(prefersReducedMotion.current);

  let deps = $state<DepStatus[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let listVisible = $state(false);
  let animGeneration = $state(0);
  let copiedDepId = $state<string | null>(null);
  let copyError = $state<string | null>(null);

  const missing = $derived(deps.filter((d) => !d.installed && d.optional));

  function itemFly(index: number) {
    return {
      y: reducedMotion ? 0 : 10,
      duration: reducedMotion ? 0 : MOTION_DEPS_ENTER_MS,
      delay: reducedMotion ? 0 : depsStaggerDelayMs(index),
      easing: easingDecelerate,
    };
  }

  function depName(id: string, fallbackKey: string): string {
    return t(DEP_COPY[id]?.nameKey ?? fallbackKey);
  }

  function depDetail(id: string): string | null {
    const key = DEP_COPY[id]?.detailKey;
    return key ? t(key) : null;
  }

  onMount(() => {
    void refresh({ initial: true });
  });

  async function refresh(options?: { initial?: boolean }) {
    const initial = options?.initial ?? false;
    error = null;

    // Only consume prefetch cache on first settings load; Recheck always hits check_dependencies.
    if (initial) {
      const cached = takeCachedDependencies();
      if (cached) {
        deps = cached;
        loading = false;
        listVisible = true;
        return;
      }
    }

    // Recheck immediately — do not wait for exit animation (Emil: no delay for cosmetics).
    loading = true;
    try {
      const result = await invoke<DepStatus[]>("check_dependencies");
      deps = result;
      animGeneration += 1;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
      listVisible = deps.length > 0 || !!error;
    }
  }

  async function copyBrewCommand(dep: DepStatus) {
    if (!dep.brewHint) return;
    copyError = null;
    try {
      await invoke("copy_text_to_clipboard", { text: dep.brewHint });
      try {
        await invoke("trigger_haptic");
      } catch {
        /* optional */
      }
      copiedDepId = dep.id;
      setTimeout(() => {
        if (copiedDepId === dep.id) copiedDepId = null;
      }, 2500);
    } catch (e) {
      copiedDepId = null;
      copyError = e instanceof Error ? e.message : String(e) || t("deps.copyFailed");
    }
  }

  async function openBrewSite() {
    try {
      await Command.create("open", [BREW_URL]).spawn();
    } catch {
      window.open(BREW_URL, "_blank", "noopener,noreferrer");
    }
  }
</script>

<div class="deps-preflight settings-card">
  <div class="settings-section-header deps-preflight-header">
    <span class="settings-section-title settings-section-title--caps">{t("deps.title")}</span>
    <button
      type="button"
      class="secondary deps-refresh-btn"
      disabled={loading}
      aria-busy={loading}
      onclick={() => refresh()}
    >
      {loading ? t("deps.checking") : t("deps.recheck")}
    </button>
  </div>
  <p class="settings-hint deps-card-hint">{t("deps.hint")}</p>

  {#if loading && deps.length === 0}
    <p class="settings-hint deps-checking-line" transition:fade={{ duration: reducedMotion ? 0 : 120 }}>
      {t("deps.checking")}
    </p>
  {/if}

  {#if error}
    <p class="settings-hint deps-error" role="alert" transition:fade={{ duration: reducedMotion ? 0 : 120 }}>
      {error}
    </p>
  {/if}

  {#if copyError}
    <p class="settings-hint deps-error" role="alert" transition:fade={{ duration: reducedMotion ? 0 : 120 }}>
      {copyError}
    </p>
  {/if}

  {#if listVisible || !loading}
    <ul
      class="deps-list"
      in:fade={{ duration: reducedMotion ? 80 : 120 }}
      out:fade={{ duration: reducedMotion ? 60 : 120 }}
    >
      <li
        class="deps-item deps-item-installed"
        aria-label={depRowAriaLabel(
          t("deps.pdf"),
          true,
          t("deps.statusInstalled"),
          t("deps.statusMissing")
        )}
        in:fly={itemFly(0)}
        out:fly={rowFlyOut(reducedMotion)}
      >
        <div class="deps-item-row">
          <span
            class="deps-status-badge deps-status-installed"
            aria-hidden="true"
          >
            <CheckIcon class="deps-check-icon" size={12} weight="bold" />
          </span>
          <div class="deps-item-copy">
            <span class="deps-item-label">{t("deps.pdf")}</span>
          </div>
          <span class="deps-built-in-badge">{t("deps.builtIn")}</span>
        </div>
      </li>

      {#each deps as dep, index (dep.id)}
        <li
          class="deps-item"
          class:deps-item-installed={dep.installed}
          class:deps-item-missing={!dep.installed}
          aria-label={depRowAriaLabel(
            depName(dep.id, dep.labelKey),
            dep.installed,
            t("deps.statusInstalled"),
            t("deps.statusMissing")
          )}
          in:fly={itemFly(index + 1)}
          out:fly={rowFlyOut(reducedMotion)}
        >
          <div class="deps-item-row">
            <span
              class="deps-status-badge"
              class:deps-status-installed={dep.installed}
              class:deps-status-missing={!dep.installed}
              class:deps-status-pop={dep.installed && animGeneration > 0}
              style:--deps-pop-delay="{reducedMotion ? 0 : depsPopDelayMs(index + 1)}ms"
              aria-hidden="true"
            >
              {#if dep.installed}
                <CheckIcon class="deps-check-icon" size={12} weight="bold" />
              {:else}
                <XIcon class="deps-missing-icon" size={12} weight="bold" />
              {/if}
            </span>
            <div class="deps-item-copy">
              <span class="deps-item-label">{depName(dep.id, dep.labelKey)}</span>
              {#if depDetail(dep.id)}
                <span class="deps-item-detail">{depDetail(dep.id)}</span>
              {/if}
            </div>
            {#if dep.installed && !dep.optional}
              <span class="deps-built-in-badge">{t("deps.builtIn")}</span>
            {:else if dep.installed && dep.optional}
              <span class="deps-available-badge">{t("deps.available")}</span>
            {/if}
          </div>
          {#if !dep.installed && dep.brewHint}
            <div
              class="deps-install-block"
              in:fade={{ duration: reducedMotion ? 0 : 150 }}
              out:fade={{ duration: reducedMotion ? 0 : 120 }}
            >
              <code class="deps-brew-hint">
                {dep.brewHint}
              </code>
              <div class="deps-install-actions">
                <button
                  type="button"
                  class="secondary deps-install-btn"
                  class:deps-install-copied={copiedDepId === dep.id}
                  onclick={() => copyBrewCommand(dep)}
                >
                  {copiedDepId === dep.id ? t("deps.copied") : t("deps.copyInstall")}
                </button>
                <button type="button" class="secondary deps-install-btn" onclick={openBrewSite}>
                  {t("deps.getHomebrew")}
                </button>
              </div>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
    {#if missing.length > 0}
      <p class="settings-hint deps-missing-note" transition:fade={{ duration: reducedMotion ? 0 : 150 }}>
        {t("deps.missingNote")}
      </p>
    {/if}
  {/if}
</div>
