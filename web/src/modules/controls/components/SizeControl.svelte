<script lang="ts">
  import type { Controls } from '../controls.svelte';
  import { LIMITS } from '../utils/limits';
  import { roundMm } from '../utils/size';
  import NumberBox from './NumberBox.svelte';

  let { controls }: { controls: Controls } = $props();

  const id = $props.id();
  const { min, max } = LIMITS.longestSide;
</script>

<fieldset>
  <legend>Size</legend>
  <input
    type="range"
    aria-label="Size (longest side)"
    {min}
    {max}
    step={LIMITS.longestSide.step}
    value={controls.longestSide}
    oninput={(event) => controls.setSize('longest', event.currentTarget.valueAsNumber)}
  />
  <div class="boxes">
    <label for="{id}-x">X</label>
    <NumberBox
      id="{id}-x"
      value={roundMm(controls.size.x)}
      min={0.01}
      {max}
      step={0.01}
      onchange={(mm: number) => controls.setSize('x', mm)}
    />
    <span aria-hidden="true">×</span>
    <label for="{id}-y">Y</label>
    <NumberBox
      id="{id}-y"
      value={roundMm(controls.size.y)}
      min={0.01}
      {max}
      step={0.01}
      onchange={(mm: number) => controls.setSize('y', mm)}
    />
    <span class="unit">mm</span>
  </div>
  <p class="hint">The aspect ratio is fixed by the art. The longest side is {min}–{max} mm.</p>
</fieldset>

<style>
  fieldset {
    display: grid;
    gap: 0.4rem;
    margin: 0;
    padding: 0;
    border: none;
  }

  legend {
    margin-bottom: 0.25rem;
    font-size: 0.9rem;
  }

  .boxes {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .unit,
  .hint {
    font-size: 0.8rem;
    color: var(--color-muted);
  }

  .hint {
    margin: 0;
  }
</style>
