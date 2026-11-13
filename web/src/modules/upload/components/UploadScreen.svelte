<script lang="ts">
  import { UploadZone } from '../upload-zone.svelte';
  import { ACCEPTED_TYPES } from '../utils/accepted';

  interface Props {
    onfile: (file: File) => void;
    /** Why the last image couldn't be opened, if it couldn't. */
    error?: string | null;
  }

  let { onfile, error = null }: Props = $props();

  const zone = new UploadZone((file) => onfile(file));
</script>

<section class="upload">
  <div class="notice" role="note">
    <strong>Logos and flat, vector-like art only.</strong>
    Images with solid areas of colour work best. Photos, gradients and soft shading aren’t supported.
    A transparent background marks what’s outside the piece.
  </div>

  <label
    class={['drop', zone.dragging && 'dragging']}
    ondragenter={(event) => {
      event.preventDefault();
      zone.dragging = true;
    }}
    ondragover={(event) => event.preventDefault()}
    ondragleave={() => (zone.dragging = false)}
    ondrop={(event) => {
      event.preventDefault();
      zone.take(event.dataTransfer?.files);
    }}
  >
    <span class="title">Drop an image here, or click to choose one</span>
    <span class="types">PNG, JPG or WebP. Nothing is uploaded: it all runs in your browser.</span>
    <input
      type="file"
      accept={ACCEPTED_TYPES.join(',')}
      onchange={(event) => {
        zone.take(event.currentTarget.files);
        event.currentTarget.value = '';
      }}
    />
  </label>

  {#if zone.rejected ?? error}
    <p class="error" role="alert">{zone.rejected ?? error}</p>
  {/if}
</section>

<style>
  .upload {
    display: grid;
    gap: 1rem;
    max-width: 40rem;
    margin: 3rem auto;
    padding: 0 1rem;
  }

  .notice {
    padding: 0.75rem 1rem;
    line-height: 1.45;
    background: var(--color-surface);
    border-left: 4px solid var(--color-accent);
    border-radius: var(--radius);
  }

  .drop {
    display: grid;
    gap: 0.4rem;
    justify-items: center;
    padding: 3rem 1rem;
    text-align: center;
    cursor: pointer;
    background: var(--color-surface);
    border: 2px dashed var(--color-border);
    border-radius: var(--radius);
  }

  .drop:hover,
  .drop:focus-within,
  .dragging {
    border-color: var(--color-accent);
  }

  .title {
    font-weight: 600;
  }

  .types {
    font-size: 0.85rem;
    color: var(--color-muted);
  }

  input[type='file'] {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
  }

  .error {
    margin: 0;
    color: var(--color-danger);
  }
</style>
