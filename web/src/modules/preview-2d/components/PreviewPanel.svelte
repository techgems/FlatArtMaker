<script lang="ts">
  import { getSession } from '../../session';
  import { PREVIEW_TABS, PreviewTabs } from '../preview-tabs.svelte';
  import MappedView from './MappedView.svelte';

  const session = getSession();
  const tabs = new PreviewTabs();
  const id = $props.id();

  const thin = $derived(session.previews?.report.thinFeatures.length ?? 0);
</script>

<div class="preview">
  <div class="bar">
    <div role="tablist" aria-label="Previews">
      {#each PREVIEW_TABS as tab (tab.id)}
        <button
          role="tab"
          id="{id}-{tab.id}"
          aria-selected={tabs.current === tab.id}
          aria-controls="{id}-panel"
          onclick={() => (tabs.current = tab.id)}
        >
          {tab.label}
        </button>
      {/each}
    </div>
    {#if session.updating}
      <span class="status" aria-live="polite">Updating…</span>
    {/if}
  </div>

  <div class="stage" role="tabpanel" id="{id}-panel" aria-labelledby="{id}-{tabs.current}">
    {#if tabs.current === 'original' && session.imageUrl}
      <img src={session.imageUrl} alt="The uploaded original" />
    {:else if tabs.current === 'mapped' && session.previews}
      <MappedView mapped={session.previews.mapped} />
    {:else if tabs.current === 'vector' && session.svgUrl}
      <img src={session.svgUrl} alt="The traced vector art" />
    {:else if tabs.current === '3d'}
      <p class="placeholder">The 3D preview is coming soon.</p>
    {/if}
  </div>

  {#if session.error}
    <p class="message error" role="alert">{session.error}</p>
  {/if}
  {#if thin > 0}
    <p class="message warning">
      {thin === 1 ? '1 feature is' : `${thin} features are`} thinner than the threshold and couldn’t be
      fixed without changing the outline.
    </p>
  {/if}
</div>

<style>
  .preview {
    display: grid;
    grid-template-rows: auto 1fr auto auto;
    gap: 0.5rem;
    height: 100%;
    min-height: 0;
  }

  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  [role='tablist'] {
    display: flex;
    gap: 0.25rem;
  }

  [role='tab'] {
    padding: 0.4rem 0.8rem;
    font: inherit;
    color: var(--color-muted);
    cursor: pointer;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
  }

  [role='tab'][aria-selected='true'] {
    color: var(--color-text);
    border-bottom-color: var(--color-accent);
  }

  .status {
    font-size: 0.85rem;
    color: var(--color-muted);
  }

  .stage {
    display: grid;
    place-items: center;
    min-height: 0;
    padding: 1rem;
    overflow: hidden;
    border-radius: var(--radius);
    /* A checkerboard, so transparent "outside" areas show. */
    background: repeating-conic-gradient(var(--color-check) 0 25%, var(--color-surface) 0 50%) 0 0 /
      20px 20px;
  }

  .stage > :global(img) {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .placeholder {
    color: var(--color-muted);
  }

  .message {
    margin: 0;
    font-size: 0.9rem;
  }

  .error {
    color: var(--color-danger);
  }

  .warning {
    color: var(--color-warning);
  }
</style>
