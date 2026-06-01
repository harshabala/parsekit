<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  let { label, value, onSelect }: { label: string; value: string; onSelect: (path: string) => void } = $props();

  function truncatePath(path: string, maxLen = 30): string {
    if (!path) return "Not selected...";
    if (path.length <= maxLen) return path;
    const parts = path.split("/");
    if (parts.length <= 2) return "..." + path.slice(-maxLen);
    return ".../" + parts.slice(-2).join("/");
  }

  async function pick() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      onSelect(selected as string);
    }
  }
</script>

<div class="row">
  <button class="secondary" onclick={pick}>{label}</button>
  <span class="path-preview" title={value}>{truncatePath(value)}</span>
</div>
