import type { ArtBounds } from '../../engine';
import { LIMITS } from './limits';

/** The printed size of the art, in mm. */
export interface PrintSize {
  readonly x: number;
  readonly y: number;
}

/** Which size the user set: a box, or the slider (the longest side). */
export type SizeSide = 'x' | 'y' | 'longest';

export function printSize(scale: number, bounds: ArtBounds): PrintSize {
  return { x: scale * bounds.width, y: scale * bounds.height };
}

export function longestSide(scale: number, bounds: ArtBounds): number {
  return scale * Math.max(bounds.width, bounds.height);
}

/**
 * The mm per source pixel that makes `side` `mm` long. The other side
 * follows from the art's aspect ratio, which can't change; the longest
 * side stays within its limits. `null` for something that isn't a size.
 */
export function scaleFor(side: SizeSide, mm: number, bounds: ArtBounds): number | null {
  if (!Number.isFinite(mm) || mm <= 0) return null;
  const longest = Math.max(bounds.width, bounds.height);
  const pixels = { x: bounds.width, y: bounds.height, longest };
  const scale = mm / pixels[side];
  const { min, max } = LIMITS.longestSide;
  return Math.min(max, Math.max(min, scale * longest)) / longest;
}

/** A length shown to 0.01 mm. */
export function roundMm(mm: number): number {
  return Math.round(mm * 100) / 100;
}
