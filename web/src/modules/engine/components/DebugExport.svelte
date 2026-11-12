<script lang="ts">
  // TEMPORARY (tasks 14, 19): dev-only export check. Remove with debug-export.svelte.ts.
  import { getEngineClient } from '../context';
  import { DebugExport, type DebugExportFormat } from '../debug-export.svelte';

  const debug = new DebugExport(getEngineClient());
  let format = $state<DebugExportFormat>('prusa');

  function onFileChange(event: Event & { currentTarget: HTMLInputElement }): void {
    const input = event.currentTarget;
    const file = input.files?.[0];
    if (!file) return;
    void debug.run(file, format);
    // Clearing lets the same file be chosen again for a re-run.
    input.value = '';
  }
</script>

<section class="debug">
  <h2>Debug: export (temporary, dev only)</h2>
  <div class="controls">
    <label>
      Format
      <select bind:value={format} disabled={debug.phase === 'working'}>
        <option value="prusa">3MF (prusa)</option>
        <option value="bambu">3MF (bambu)</option>
        <option value="step">STEP</option>
      </select>
    </label>
    <label>
      Image
      <input
        type="file"
        accept="image/png,image/jpeg,image/webp"
        disabled={debug.phase === 'working'}
        onchange={onFileChange}
      />
    </label>
  </div>

  <p class={['status', debug.phase]} data-testid="debug-phase">
    {#if debug.phase === 'idle'}
      Choose an image to export.
    {:else if debug.phase === 'working'}
      Exporting…
    {:else if debug.phase === 'done'}
      Done.
    {:else}
      Failed: {debug.error}
    {/if}
  </p>

  {#if debug.phase === 'done' && debug.result}
    <dl>
      <dt>File</dt>
      <dd data-testid="debug-file-name">{debug.result.fileName}</dd>
      <dt>Bytes</dt>
      <dd data-testid="debug-bytes">{debug.result.bytes}</dd>
      <dt>SHA-256</dt>
      <dd data-testid="debug-sha256" class="hash">{debug.result.sha256}</dd>
    </dl>
  {/if}
</section>

<style>
  .debug {
    margin: 1rem 1.5rem;
    padding: 0.75rem 1rem;
    border: 1px dashed var(--color-muted);
    background: var(--color-surface);
  }

  h2 {
    margin: 0 0 0.5rem;
    font-size: 0.95rem;
    color: var(--color-muted);
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .status {
    color: var(--color-muted);
  }

  .failed {
    color: var(--color-danger);
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.25rem 1rem;
    margin: 0;
  }

  dd {
    margin: 0;
  }

  .hash {
    font-family: monospace;
    overflow-wrap: anywhere;
  }
</style>
