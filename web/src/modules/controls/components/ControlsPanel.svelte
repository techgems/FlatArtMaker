<script lang="ts">
  import { getSession } from '../../session';
  import { Controls } from '../controls.svelte';
  import { LIMITS } from '../utils/limits';
  import AdvancedSection from './AdvancedSection.svelte';
  import NumberField from './NumberField.svelte';
  import PrintabilityGroup from './PrintabilityGroup.svelte';
  import SizeControl from './SizeControl.svelte';

  const controls = new Controls(getSession());
</script>

{#if controls.ready}
  <div class="panel">
    <section class="group">
      <NumberField
        label="Colour layer height (C)"
        value={controls.colourHeight}
        limit={LIMITS.colourHeight}
        unit="mm"
        onchange={(mm: number) => controls.setColourHeight(mm)}
      />
      <NumberField
        label="Base height (B)"
        value={controls.baseHeight}
        limit={LIMITS.baseHeight}
        unit="mm"
        hint={controls.noBase ? 'No base: the colours start at 0.' : undefined}
        onchange={(mm: number) => controls.setBaseHeight(mm)}
      />
      <SizeControl {controls} />
    </section>
    <section class="group">
      <PrintabilityGroup {controls} />
    </section>
    <section class="group">
      <AdvancedSection {controls} />
    </section>
  </div>
{/if}

<style>
  .panel {
    display: grid;
    gap: 1rem;
  }

  .group {
    display: grid;
    gap: 0.9rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .group:last-child {
    border-bottom: none;
  }
</style>
