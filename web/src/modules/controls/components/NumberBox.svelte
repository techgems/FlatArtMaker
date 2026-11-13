<script lang="ts">
  import { tick } from 'svelte';

  interface Props {
    id: string;
    value: number;
    min: number;
    max: number;
    step: number;
    disabled?: boolean;
    label?: string;
    /** Receives what was typed; the parent decides what to keep. */
    onchange: (value: number) => void;
  }

  let { id, value, min, max, step, disabled = false, label, onchange }: Props = $props();

  async function commit(input: HTMLInputElement): Promise<void> {
    onchange(input.valueAsNumber);
    // A clamped or rejected entry leaves `value` as it was, which wouldn't
    // redraw the box: show the value in use.
    await tick();
    input.value = String(value);
  }
</script>

<input
  {id}
  type="number"
  {min}
  {max}
  {step}
  {value}
  {disabled}
  aria-label={label}
  onchange={(event) => void commit(event.currentTarget)}
/>

<style>
  input {
    width: 4.5rem;
    padding: 0.2rem 0.3rem;
    font: inherit;
    color: inherit;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }
</style>
