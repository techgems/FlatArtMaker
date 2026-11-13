<script lang="ts">
  import type { MappedPreview } from '../../engine';
  import { paintLabels } from '../utils/paint-labels';

  let { mapped }: { mapped: MappedPreview } = $props();

  function draw(canvas: HTMLCanvasElement): void {
    const { width, height, labels, outside, palette } = mapped;
    canvas.width = width;
    canvas.height = height;
    const pixels = new ImageData(paintLabels(labels, outside, palette), width, height);
    canvas.getContext('2d')?.putImageData(pixels, 0, 0);
  }
</script>

<canvas {@attach draw} aria-label="The image in its output colours"></canvas>

<style>
  canvas {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    image-rendering: pixelated;
  }
</style>
