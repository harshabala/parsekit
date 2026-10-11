<script lang="ts">
  /**
   * Settings section icons via Phosphor (phosphor-svelte).
   * Same icon family as @phosphor-icons/react — Svelte binding required for this app.
   * @see https://phosphoricons.com/
   */
  import GlobeIcon from "phosphor-svelte/lib/GlobeIcon";
  import PaletteIcon from "phosphor-svelte/lib/PaletteIcon";
  import PowerIcon from "phosphor-svelte/lib/PowerIcon";
  import KeyboardIcon from "phosphor-svelte/lib/KeyboardIcon";
  import ChartBarIcon from "phosphor-svelte/lib/ChartBarIcon";
  import ShieldIcon from "phosphor-svelte/lib/ShieldIcon";
  import FolderIcon from "phosphor-svelte/lib/FolderIcon";
  import DownloadSimpleIcon from "phosphor-svelte/lib/DownloadSimpleIcon";
  import InfoIcon from "phosphor-svelte/lib/InfoIcon";

  let {
    name,
    size = 16,
    weight = "regular",
  }: {
    name:
      | "globe"
      | "palette"
      | "power"
      | "keyboard"
      | "chart"
      | "shield"
      | "folder"
      | "download"
      | "info";
    size?: number;
    weight?: "thin" | "light" | "regular" | "bold" | "fill" | "duotone";
  } = $props();

  const icons = {
    globe: GlobeIcon,
    palette: PaletteIcon,
    power: PowerIcon,
    keyboard: KeyboardIcon,
    chart: ChartBarIcon,
    shield: ShieldIcon,
    folder: FolderIcon,
    download: DownloadSimpleIcon,
    info: InfoIcon,
  } as const;

  const Icon = $derived(icons[name]);
</script>

<span class="settings-section-icon" aria-hidden="true">
  <Icon {size} {weight} />
</span>
