<script lang="ts">
  import type { Controls } from '../controls.svelte';
  import { LIMITS } from '../utils/limits';
  import NumberField from './NumberField.svelte';

  let { controls }: { controls: Controls } = $props();
</script>

<fieldset>
  <legend>Printability</legend>
  <label class="toggle">
    <input
      type="checkbox"
      checked={controls.fixThin}
      onchange={(event) => controls.setFixThin(event.currentTarget.checked)}
    />
    Fix thin features
  </label>
  <NumberField
    label="Thin-feature threshold"
    value={controls.threshold}
    limit={LIMITS.minFeature}
    unit="mm"
    disabled={!controls.fixThin}
    hint={controls.fixThin
      ? 'Parts narrower than this in any direction are merged into a neighbour.'
      : 'Off: thin details are kept as traced.'}
    onchange={(mm: number) => controls.setThreshold(mm)}
  />
</fieldset>

<style>
  fieldset {
    display: grid;
    gap: 0.5rem;
    margin: 0;
    padding: 0;
    border: none;
  }

  legend {
    margin-bottom: 0.25rem;
    font-weight: 600;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.9rem;
  }
</style>
