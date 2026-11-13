<script lang="ts">
  import type { Limit } from '../utils/limits';
  import NumberBox from './NumberBox.svelte';

  interface Props {
    label: string;
    value: number;
    limit: Limit;
    unit?: string;
    disabled?: boolean;
    /** Says why the field is disabled, or what a special value means. */
    hint?: string;
    onchange: (value: number) => void;
  }

  let { label, value, limit, unit = '', disabled = false, hint, onchange }: Props = $props();

  const id = $props.id();
</script>

<div class="field">
  <label for="{id}-box">{label}</label>
  <div class="inputs">
    <input
      type="range"
      aria-label={label}
      min={limit.min}
      max={limit.max}
      step={limit.step}
      {value}
      {disabled}
      oninput={(event) => onchange(event.currentTarget.valueAsNumber)}
    />
    <NumberBox
      id="{id}-box"
      {value}
      min={limit.min}
      max={limit.max}
      step={limit.step}
      {disabled}
      {onchange}
    />
    <span class="unit">{unit}</span>
  </div>
  {#if hint}
    <p class="hint">{hint}</p>
  {/if}
</div>

<style>
  .field {
    display: grid;
    gap: 0.25rem;
  }

  label {
    font-size: 0.9rem;
  }

  .inputs {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  input[type='range'] {
    flex: 1;
    min-width: 0;
  }

  .unit {
    width: 2rem;
    font-size: 0.85rem;
    color: var(--color-muted);
  }

  .hint {
    margin: 0;
    font-size: 0.8rem;
    color: var(--color-muted);
  }
</style>
