import { describe, expect, it } from 'vitest';
import type { ArtBounds, Settings } from '../engine';
import { Controls, type SettingsSource } from './controls.svelte';
import { fit } from './utils/clamp';
import { LIMITS } from './utils/limits';
import { scaleFor } from './utils/size';

/** Art 400 × 300 px, at 100 mm on its longest side. */
const bounds: ArtBounds = { x: 0, y: 0, width: 400, height: 300 };

function defaults(): Settings {
  return {
    version: 1,
    colourHeight: 1,
    baseHeight: 3.4,
    scale: 100 / 400,
    minFeature: 0.25,
    load: { alphaThreshold: 128, background: undefined },
    advanced: {
      detectedColours: 24,
      outputColours: 8,
      speckleArea: 16,
      traceUpscale: 2,
      smoothing: 0.5,
      corner: 60,
      curveTolerance: 0.02,
      fillBaseHoles: true,
    },
  };
}

/** A source that keeps whatever the controls send it. */
function source(): SettingsSource & { settings: Settings } {
  return {
    settings: defaults(),
    bounds,
    update(settings) {
      this.settings = settings;
    },
  };
}

describe('X/Y size', () => {
  it('sets Y from the aspect ratio when X is typed', () => {
    const s = source();
    const controls = new Controls(s);
    controls.setSize('x', 200);
    expect(controls.size.x).toBeCloseTo(200, 10);
    expect(controls.size.y).toBeCloseTo(150, 10);
  });

  it('sets X from the aspect ratio when Y is typed', () => {
    const controls = new Controls(source());
    controls.setSize('y', 60);
    expect(controls.size.x).toBeCloseTo(80, 10);
  });

  it('keeps the longest side within 20–300 mm', () => {
    expect((scaleFor('x', 1000, bounds) ?? 0) * 400).toBeCloseTo(300, 10);
    expect((scaleFor('y', 1, bounds) ?? 0) * 400).toBeCloseTo(20, 10);
    expect(scaleFor('x', Number.NaN, bounds)).toBeNull();
  });
});

describe('slider limits', () => {
  it('never lets C be 0', () => {
    const s = source();
    const controls = new Controls(s);
    controls.setColourHeight(0);
    expect(s.settings.colourHeight).toBe(LIMITS.colourHeight.min);
    controls.setColourHeight(-3);
    expect(s.settings.colourHeight).toBe(0.1);
  });

  it('never lets the threshold go below 0.25 mm', () => {
    const s = source();
    const controls = new Controls(s);
    controls.setThreshold(0.1);
    expect(s.settings.minFeature).toBe(0.25);
    controls.setThreshold(5);
    expect(s.settings.minFeature).toBe(1.2);
  });

  it('snaps to the step and ignores what isn’t a number', () => {
    expect(fit(0.33, LIMITS.colourHeight)).toBe(0.35);
    expect(fit(Number.NaN, LIMITS.colourHeight)).toBeNull();
    expect(fit(0.0049, LIMITS.curveTolerance)).toBe(0.005);
  });
});

describe('base height', () => {
  it('B = 0 means no base and disables the base-only controls', () => {
    const s = source();
    const controls = new Controls(s);
    expect(controls.noBase).toBe(false);
    controls.setBaseHeight(0);
    expect(s.settings.baseHeight).toBeUndefined();
    expect(controls.noBase).toBe(true);
    controls.setBaseHeight(2);
    expect(s.settings.baseHeight).toBe(2);
    expect(controls.noBase).toBe(false);
  });
});

describe('thin features', () => {
  it('keeps the threshold while the fix is off', () => {
    const s = source();
    const controls = new Controls(s);
    controls.setThreshold(0.6);
    controls.setFixThin(false);
    expect(s.settings.minFeature).toBeUndefined();
    expect(controls.threshold).toBe(0.6);
    controls.setFixThin(true);
    expect(s.settings.minFeature).toBe(0.6);
  });
});
