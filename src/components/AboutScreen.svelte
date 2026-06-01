<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleOverlayKeydown(e: KeyboardEvent) {
    if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " " || e.key === "Escape")) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="about-overlay"
  onclick={handleOverlayClick}
  onkeydown={handleOverlayKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="about-title"
  tabindex="-1"
>
  <div class="about-panel" role="document">
    <div class="about-header" id="about-title">ParseDock</div>
    <div class="about-version">v0.1.0</div>
    <div class="about-tagline">100% local document processing</div>

    <div class="about-divider"></div>

    <div class="about-section">
      <div class="about-section-title">Powered By</div>
      <div class="about-text">@llamaindex/liteparse</div>
    </div>

    <div class="about-section">
      <div class="about-section-title">License</div>
      <div class="about-text">Apache-2.0</div>
      <div class="about-text-small">
        ParseDock uses @llamaindex/liteparse, licensed under the Apache License 2.0.
      </div>
    </div>

    <button class="about-close-btn" onclick={onClose}>Close</button>
  </div>
</div>
