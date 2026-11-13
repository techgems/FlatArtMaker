<script lang="ts">
  import type { AdvancedNumber, Controls } from '../controls.svelte';
  import { LIMITS } from '../utils/limits';
  import NumberField from './NumberField.svelte';

  let { controls }: { controls: Controls } = $props();

  const fields: ReadonlyArray<{ key: AdvancedNumber; label: string; unit: string }> = [
    { key: 'detectedColours', label: 'Detected colours (max)', unit: '' },
    { key: 'outputColours', label: 'Output colours', unit: '' },
    { key: 'speckleArea', label: 'Speckle area', unit: 'px' },
    { key: 'traceUpscale', label: 'Trace upscale', unit: '×' },
    { key: 'smoothing', label: 'Smoothing tolerance', unit: 'px' },
    { key: 'corner', label: 'Corner threshold', unit: '°' },
    { key: 'curveTolerance', label: 'Curve tolerance (mesh only)', unit: 'mm' },
  ];
</script>

<details>
  <summary>Advanced</summary>
  <div class="fields">
    {#each fields as field (field.key)}
      <NumberField
        label={field.label}
        value={controls.advanced(field.key)}
        limit={LIMITS[field.key]}
        unit={field.unit}
        onchange={(value: number) => controls.setAdvanced(field.key, value)}
      />
    {/each}
    <label class="toggle">
      <input
        type="checkbox"
        checked={controls.advanced('fillBaseHoles')}
        disabled={controls.noBase}
        onchange={(event) => controls.setFillBaseHoles(event.currentTarget.checked)}
      />
      Fill holes in base
    </label>
    {#if controls.noBase}
      <p class="hint">No base (B = 0), so there are no holes to fill.</p>
    {/if}
  </div>
</details>

<style>
  summary {
    cursor: pointer;
    font-weight: 600;
  }

  .fields {
    display: grid;
    gap: 0.75rem;
    padding-top: 0.75rem;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.9rem;
  }

  .hint {
    margin: 0;
    font-size: 0.8rem;
    color: var(--color-muted);
  }
</style>
