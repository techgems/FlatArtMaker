/** A control's range and step (SPEC "Controls"). */
export interface Limit {
  readonly min: number;
  readonly max: number;
  readonly step: number;
}

/** Every numeric control's range. The engine rejects anything outside its own. */
export const LIMITS = {
  /** C: never 0. */
  colourHeight: { min: 0.1, max: 10, step: 0.05 },
  /** B: 0 means no base. */
  baseHeight: { min: 0, max: 10, step: 0.05 },
  /** The art's longest side, in mm (the X/Y size slider). */
  longestSide: { min: 20, max: 300, step: 1 },
  minFeature: { min: 0.25, max: 1.2, step: 0.05 },
  detectedColours: { min: 2, max: 32, step: 1 },
  outputColours: { min: 1, max: 16, step: 1 },
  speckleArea: { min: 0, max: 500, step: 1 },
  traceUpscale: { min: 1, max: 4, step: 1 },
  smoothing: { min: 0.1, max: 2, step: 0.05 },
  corner: { min: 0, max: 180, step: 1 },
  curveTolerance: { min: 0.005, max: 0.2, step: 0.005 },
} as const satisfies Record<string, Limit>;
