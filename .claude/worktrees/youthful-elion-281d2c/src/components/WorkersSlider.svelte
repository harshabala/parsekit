<script lang="ts">
  let {
    value,
    min = 1,
    max = 16,
    label,
    onChange,
  }: {
    value: number;
    min?: number;
    max?: number;
    label: string;
    onChange: (value: number) => void;
  } = $props();

  let fillPercent = $derived(
    max > min ? ((value - min) / (max - min)) * 100 : 0
  );
</script>

<div class="workers-slider" style="--workers-fill: {fillPercent}%">
  <div class="workers-slider-track-wrap">
    <div class="workers-slider-track" aria-hidden="true">
      <div class="workers-slider-fill"></div>
    </div>
    <input
      type="range"
      class="workers-slider-input"
      {min}
      {max}
      step="1"
      {value}
      aria-label={label}
      oninput={(e) => onChange(Number((e.currentTarget as HTMLInputElement).value))}
    />
  </div>
  <span class="workers-value" aria-hidden="true">{value}</span>
</div>