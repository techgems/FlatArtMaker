<script lang="ts">
  import { getSession } from '../../session';
  import { MeshPreview } from '../mesh-preview.svelte';

  const session = getSession();
  // three.js is fetched the first time the 3D tab opens.
  const preview = new MeshPreview(session.client, () =>
    import('../three-mesh-viewer').then((m) => m.createThreeMeshViewer),
  );

  // New engine results (any settings or edit change) mean a new mesh.
  $effect(() => {
    if (session.previews) void preview.refresh();
  });

  // Hand each new mesh to the viewer; the renderer stays.
  $effect(() => {
    if (preview.viewer && preview.mesh) preview.viewer.show(preview.mesh);
  });
</script>

<div class="model">
  <div class="canvas" {@attach (container: HTMLElement) => preview.attach(container)}></div>
  {#if preview.phase === 'loading'}
    <p class="status">Building the 3D model…</p>
  {:else if preview.phase === 'failed'}
    <p class="status error" role="alert">The 3D model couldn’t be built: {preview.error}</p>
  {/if}
</div>

<style>
  .model {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .canvas {
    position: absolute;
    inset: 0;
  }

  .status {
    position: absolute;
    inset: auto 0 0.5rem;
    margin: 0;
    text-align: center;
    color: var(--color-muted);
  }

  .error {
    color: var(--color-danger);
  }
</style>
