import type { OutputPalette } from '../../engine';

/**
 * An output label map as RGBA pixels: each cell in its output colour's
 * display colour, outside (and any unknown label) transparent.
 */
export function paintLabels(
  labels: Uint16Array,
  outside: number,
  palette: OutputPalette,
): Uint8ClampedArray<ArrayBuffer> {
  const colours = new Map(palette.colours.map((c) => [c.id, c.display]));
  const rgba = new Uint8ClampedArray(labels.length * 4);
  labels.forEach((label, i) => {
    const colour = label === outside ? undefined : colours.get(label);
    if (colour === undefined) return;
    rgba.set([colour.r, colour.g, colour.b, 255], i * 4);
  });
  return rgba;
}
