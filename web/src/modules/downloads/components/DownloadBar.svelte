<script lang="ts">
  import { getSession } from '../../session';
  import { Downloads } from '../downloads.svelte';
  import { SLICERS } from '../utils/slicer-choice';

  const session = getSession();
  const downloads = new Downloads(session);
  const id = $props.id();

  // Pending settings would be missed by an export started now.
  const disabled = $derived(
    session.phase !== 'ready' || session.updating || downloads.working !== null,
  );
</script>

<div class="downloads">
  <label for="{id}-slicer" class="visually-hidden">Slicer</label>
  <select
    id="{id}-slicer"
    value={downloads.slicer}
    onchange={(event) => {
      const choice = SLICERS.find((s) => s.id === event.currentTarget.value);
      if (choice) downloads.setSlicer(choice.id);
    }}
  >
    {#each SLICERS as slicer (slicer.id)}
      <option value={slicer.id}>{slicer.label}</option>
    {/each}
  </select>
  <button type="button" class="primary" {disabled} onclick={() => void downloads.download('3mf')}>
    {#if downloads.working === '3mf'}<span class="spinner" aria-hidden="true"></span>{/if}
    Download 3MF
  </button>
  <button type="button" {disabled} onclick={() => void downloads.download('step')}>
    {#if downloads.working === 'step'}<span class="spinner" aria-hidden="true"></span>{/if}
    Download STEP
  </button>
  <button type="button" {disabled} onclick={() => void downloads.download('svg')}>
    Download SVG
  </button>
  <span class="visually-hidden" aria-live="polite">
    {downloads.working ? `Preparing the ${downloads.working.toUpperCase()} file…` : ''}
  </span>
  {#if downloads.error}
    <p class="error" role="alert">{downloads.error}</p>
  {/if}
</div>

<style>
  .downloads {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }

  select,
  button {
    padding: 0.45rem 0.75rem;
    font: inherit;
    color: inherit;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    cursor: pointer;
  }

  button:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .primary {
    font-weight: 600;
    color: var(--color-on-accent);
    background: var(--color-accent);
    border-color: var(--color-accent);
  }

  .spinner {
    width: 0.9em;
    height: 0.9em;
    border: 2px solid currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error {
    flex-basis: 100%;
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-danger);
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
