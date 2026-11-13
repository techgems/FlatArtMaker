import { describe, expect, it } from 'vitest';
import type { Face, MappedPreview } from '../../engine';
import { textOn } from './contrast';
import { coverage } from './coverage';

const OUT = 0xffff;

function mapped(labels: number[]): MappedPreview {
  return {
    width: labels.length,
    height: 1,
    outside: OUT,
    labels: new Uint16Array(labels),
    palette: {
      colours: [
        { id: 0, name: 'Black', display: { r: 0, g: 0, b: 0 } },
        { id: 1, name: 'White', display: { r: 255, g: 255, b: 255 } },
        { id: 2, name: 'Red', display: { r: 255, g: 0, b: 0 } },
      ],
      base: 0,
    },
    detected: { colours: [] },
    mapping: { targets: [] },
    similarity: { colours: 0, edgeTones: 0, merges: [] },
  };
}

const face = (colour: number): Face => ({ colour, outer: { steps: [] }, holes: [] });

describe('coverage', () => {
  it('numbers only colours in the model, without gaps, and shares the opaque pixels', () => {
    const rows = coverage(mapped([0, 0, 2, OUT]), [face(0), face(2)]);
    expect(rows.map((r) => r.slot)).toEqual([1, null, 2]);
    expect(rows.map((r) => r.share)).toEqual([2 / 3, 0, 1 / 3]);
  });
});

describe('textOn', () => {
  it('picks the readable text colour', () => {
    expect(textOn({ r: 0, g: 0, b: 0 })).toBe('#fff');
    expect(textOn({ r: 255, g: 230, b: 120 })).toBe('#000');
  });
});
