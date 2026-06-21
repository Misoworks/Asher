<script lang="ts">
  let {
    label,
    value,
    min,
    max,
    step = 1,
    unit = "",
    format = (value: number) => `${value}${unit}`,
    onChange,
  }: {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    unit?: string;
    format?: (value: number) => string;
    onChange: (value: number) => void;
  } = $props();

  let dragging = $state(false);
  let draftValue = $state<number | null>(null);

  const displayValue = $derived(snap(draftValue ?? value));
  const ratio = $derived((displayValue - min) / Math.max(1, max - min));

  function snap(value: number) {
    const stepped = Math.round((value - min) / step) * step + min;
    return Number(Math.min(max, Math.max(min, stepped)).toFixed(2));
  }

  function valueFromPointer(event: PointerEvent) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return min + ((event.clientX - rect.left) / rect.width) * (max - min);
  }

  function preview(value: number) {
    draftValue = snap(value);
  }

  function commit(value = displayValue) {
    const next = snap(value);
    draftValue = next;
    onChange(next);
    window.setTimeout(() => {
      if (!dragging) draftValue = null;
    }, 160);
  }

  function pointerdown(event: PointerEvent) {
    event.preventDefault();
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    dragging = true;
    preview(valueFromPointer(event));
  }

  function pointermove(event: PointerEvent) {
    if (dragging) preview(valueFromPointer(event));
  }

  function pointerup(event: PointerEvent) {
    if (!dragging) return;
    const target = event.currentTarget as HTMLElement;
    if (target.hasPointerCapture(event.pointerId)) {
      target.releasePointerCapture(event.pointerId);
    }
    dragging = false;
    commit();
  }

  function keydown(event: KeyboardEvent) {
    if (event.key === "ArrowLeft" || event.key === "ArrowDown") {
      event.preventDefault();
      commit(displayValue - step);
    } else if (event.key === "ArrowRight" || event.key === "ArrowUp") {
      event.preventDefault();
      commit(displayValue + step);
    } else if (event.key === "Home") {
      event.preventDefault();
      commit(min);
    } else if (event.key === "End") {
      event.preventDefault();
      commit(max);
    }
  }
</script>

<div class="settings-row settings-slider-row" style:--settings-slider-ratio={ratio}>
  <span>{label}</span>
  <button
    type="button"
    class="settings-slider"
    role="slider"
    aria-label={label}
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={displayValue}
    aria-valuetext={format(displayValue)}
    onpointerdown={pointerdown}
    onpointermove={pointermove}
    onpointerup={pointerup}
    onpointercancel={pointerup}
    onkeydown={keydown}
  >
    <span class="settings-slider-fill"></span>
    <span class="settings-slider-thumb"></span>
  </button>
  <strong>{format(displayValue)}</strong>
</div>
