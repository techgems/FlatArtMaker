import { describe, expect, it } from 'vitest';
import { paintLabels } from './paint-labels';

describe('paintLabels', () => {
  it('paints each label in its colour and leaves outside transparent', () => {
    const palette = {
      colours: [
        { id: 3, name: 'Red', display: { r: 255, g: 0, b: 0 } },
        { id: 7, name: 'Blue', display: { r: 0, g: 0, b: 255 } },
      ],
      base: 3,
    };
    const rgba = paintLabels(new Uint16Array([7, 0xffff, 3]), 0xffff, palette);
    expect(Array.from(rgba)).toEqual([0, 0, 255, 255, 0, 0, 0, 0, 255, 0, 0, 255]);
  });
});
