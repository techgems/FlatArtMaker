import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { EngineClient } from './engine-client';
import type { RgbaPixels } from './types';

const [WIDTH, HEIGHT] = [12, 6];

/** Whether a cell of `sampleImage` is transparent. */
function isClear(i: number): boolean {
  return i % WIDTH >= 10 || Math.floor(i / WIDTH) >= 5;
}

/**
 * 12 × 6: a red and a blue 5 × 5 block side by side at the top left, the
 * rest transparent. Blocks this big survive speckle clean-up.
 */
export function sampleImage(): RgbaPixels {
  const data = new Uint8Array(WIDTH * HEIGHT * 4);
  for (let i = 0; i < WIDTH * HEIGHT; i++) {
    if (isClear(i)) continue;
    data.set(i % WIDTH < 5 ? [255, 0, 0, 255] : [0, 0, 255, 255], i * 4);
  }
  return { width: WIDTH, height: HEIGHT, data };
}

/**
 * What every EngineClient must do. Run it against each implementation, so
 * the fake the UI tests use behaves like the real engine.
 */
export function engineClientContract(
  name: string,
  create: () => EngineClient | Promise<EngineClient>,
): void {
  describe(`EngineClient contract: ${name}`, () => {
    let client: EngineClient;

    beforeEach(async () => {
      client = await create();
    });

    afterEach(() => {
      client.dispose();
    });

    it('reports a version', async () => {
      expect(await client.version()).toMatch(/\S/);
    });

    it('rejects project calls until an image is loaded', async () => {
      await expect(client.previews()).rejects.toThrow();
      await expect(client.mesh()).rejects.toThrow();
      await expect(client.export3mf('prusa', 'x')).rejects.toThrow();
      await expect(client.exportSvg('x')).rejects.toThrow();
    });

    it('loads an image: its art bounds, and defaults sized to 100 mm', async () => {
      const { bounds, settings } = await client.loadImage(sampleImage());
      expect(bounds).toEqual({ x: 0, y: 0, width: 10, height: 5 });
      expect(settings.scale * Math.max(bounds.width, bounds.height)).toBeCloseTo(100, 9);
    });

    it('rejects pixels that do not match the size', async () => {
      const image = sampleImage();
      await expect(client.loadImage({ ...image, width: 4 })).rejects.toThrow();
    });

    it('previews cover the image, outside cells marked', async () => {
      await client.loadImage(sampleImage());
      const { mapped, vector, report } = await client.previews();
      expect([mapped.width, mapped.height]).toEqual([WIDTH, HEIGHT]);
      expect(mapped.labels).toHaveLength(WIDTH * HEIGHT);
      const ids = new Set(mapped.palette.colours.map((c) => c.id));
      mapped.labels.forEach((label, i) => {
        expect(isClear(i) ? label === mapped.outside : ids.has(label)).toBe(true);
      });
      expect(vector.palette).toEqual(mapped.palette);
      expect(report).toEqual({ thinFeatures: [], droppedEdits: [] });
    });

    it('rejects invalid settings and keeps the current ones', async () => {
      const { settings } = await client.loadImage(sampleImage());
      await expect(client.setSettings({ ...settings, colourHeight: 0 })).rejects.toThrow();
      const { vector } = await client.previews();
      expect(vector.extrusion.colour).toBe(settings.colourHeight);
    });

    it('builds a base part (holding the base colour) only when there is a base height', async () => {
      const { settings } = await client.loadImage(sampleImage());
      const withBase = await client.mesh();
      expect(withBase.parts.map((p) => p.kind)).toEqual(['base', 'colour']);

      await client.setSettings({ ...settings, baseHeight: undefined });
      const flat = await client.mesh();
      expect(flat.parts.map((p) => p.kind)).toEqual(['colour', 'colour']);
    });

    it('gives mesh buffers that index their own vertices, slots without gaps', async () => {
      await client.loadImage(sampleImage());
      const { parts } = await client.mesh();
      for (const part of parts) {
        expect(part.positions.length % 3).toBe(0);
        expect(part.indices.length % 3).toBe(0);
        expect(part.indices.length).toBeGreaterThan(0);
        const vertices = part.positions.length / 3;
        expect(part.indices.every((v) => v < vertices)).toBe(true);
      }
      const slots = parts.map((p) => p.slot ?? 0).sort((a, b) => a - b);
      expect(slots).toEqual([1, 2]);
    });

    it('lists edits that cannot apply, and a new image clears the edits', async () => {
      await client.loadImage(sampleImage());
      await client.applyEdits({ ops: [{ op: 'setBase', id: 999 }], position: 1 });
      const { report } = await client.previews();
      expect(report.droppedEdits).toEqual([{ position: 0, reason: 'unknownOutputColour' }]);

      await client.loadImage(sampleImage());
      expect((await client.previews()).report.droppedEdits).toEqual([]);
    });

    it('exports a 3MF package and a named SVG', async () => {
      await client.loadImage(sampleImage());
      const threeMf = await client.export3mf('bambu', 'tiny');
      expect([...threeMf.slice(0, 4)]).toEqual([0x50, 0x4b, 3, 4]);
      const svg = new TextDecoder().decode(await client.exportSvg('tiny'));
      expect(svg).toContain('<title>tiny</title>');
    });
  });
}
