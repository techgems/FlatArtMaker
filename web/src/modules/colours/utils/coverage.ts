import type { Face, MappedPreview, OutputColour } from '../../engine';

/** One output colour as the summary shows it. */
export interface ColourCoverage {
  readonly colour: OutputColour;
  /** Its filament slot, or `null` when it has no part in the model (empty). */
  readonly slot: number | null;
  /** Its share of the piece's pixels, 0–1. */
  readonly share: number;
}

/**
 * Each output colour's slot and coverage. Slots count, in palette order,
 * only colours with a part in the model (any traced face), so they run
 * 1, 2, 3, … without gaps, as in the exports.
 */
export function coverage(mapped: MappedPreview, faces: readonly Face[]): ColourCoverage[] {
  const pixels = new Map<number, number>();
  let total = 0;
  for (const label of mapped.labels) {
    if (label === mapped.outside) continue;
    pixels.set(label, (pixels.get(label) ?? 0) + 1);
    total++;
  }
  const modelled = new Set(faces.map((face) => face.colour));
  let slot = 0;
  return mapped.palette.colours.map((colour) => ({
    colour,
    slot: modelled.has(colour.id) ? ++slot : null,
    share: total === 0 ? 0 : (pixels.get(colour.id) ?? 0) / total,
  }));
}
