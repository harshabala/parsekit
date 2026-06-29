<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade, fly, slide } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
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

  const reducedMotion = $derived(prefersReducedMotion.current);
  const listSlide = $derived({
    duration: reducedMotion ? 0 : 200,
    easing: easingDecelerate,
  });

  let deps = $state<DepStatus[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let listVisible = $state(false);
  let animGeneration = $state(0);

  const missing = $derived(deps.filter((d) => !d.installed && d.optional));

  function itemFly(index: number) {
    return {
      y: reducedMotion ? 0 : 10,
      duration: reducedMotion ? 0 : MOTION_DEPS_ENTER_MS,
      delay: reducedMotion ? 0 : depsStaggerDelayMs(index),
      easing: easingDecelerate,
    };
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

    if (!initial && deps.length > 0) {
      listVisible = false;
      await tick();
      if (!reducedMotion) {
        await new Promise((r) => setTimeout(r, 180));
      }
    }

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
</script>

<div class="deps-preflight">
  <div class="deps-preflight-header">
    <span class="settings-section-title">{t("deps.title")}</span>
    <button type="button" class="secondary deps-refresh-btn" disabled={loading} onclick={() => refresh()}>
      {loading ? t("deps.checking") : t("deps.recheck")}
    </button>
  </div>
  <p class="settings-hint">{t("deps.hint")}</p>

  {#if loading && deps.length === 0}
    <p class="settings-hint deps-checking-line" transition:fade={{ duration: reducedMotion ? 0 : 120 }}>
      {t("deps.checking")}
    </p>
  {/if}

  {#if error}
    <p class="settings-hint deps-error" transition:fade={{ duration: reducedMotion ? 0 : 120 }}>{error}</p>
  {/if}

  {#if listVisible && deps.length > 0}
    <ul class="deps-list" transition:slide={listSlide}>
      {#each deps as dep, index (dep.id)}
        <li
          class="deps-item"
          class:deps-item-installed={dep.installed}
          class:deps-item-missing={!dep.installed}
          aria-label={depRowAriaLabel(
            t(dep.labelKey),
            dep.installed,
            t("deps.statusInstalled"),
            t("deps.statusMissing")
          )}
          in:fly={itemFly(index)}
          out:fly={rowFlyOut(reducedMotion)}
        >
          <div class="deps-item-row">
            <span
              class="deps-status-badge"
              class:deps-status-installed={dep.installed}
              class:deps-status-missing={!dep.installed}
              class:deps-status-pop={dep.installed && animGeneration > 0}
              style:--deps-pop-delay="{reducedMotion ? 0 : depsPopDelayMs(index)}ms"
              aria-hidden="true"
            >
              {#if dep.installed}
                <svg class="deps-check-icon" width="11" height="11" viewBox="0 0 12 12" fill="none">
                  <path
                    d="M2.5 6.2 4.8 8.5 9.5 3.5"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              {:else}
                <svg class="deps-missing-icon" width="11" height="11" viewBox="0 0 12 12" fill="none">
                  <path
                    d="M3.2 3.2 8.8 8.8M8.8 3.2 3.2 8.8"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linecap="round"
                  />
                </svg>
              {/if}
            </span>
            <span class="deps-item-label">{t(dep.labelKey)}</span>
          </div>
          {#if !dep.installed && dep.brewHint}
            <code class="deps-brew-hint" transition:fade={{ duration: reducedMotion ? 0 : 150 }}>
              {dep.brewHint}
            </code>
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