import { describe, expect, it, vi } from 'vitest';
import { createFakeEngineClient } from '../engine';
import type { EngineClient } from '../engine';
import { Downloads } from './downloads.svelte';
import type { SlicerStorage } from './utils/slicer-choice';

function memoryStorage(): SlicerStorage {
  const items = new Map<string, string>();
  return {
    getItem: (key) => items.get(key) ?? null,
    setItem: (key, value) => void items.set(key, value),
  };
}

async function loadedClient(): Promise<EngineClient> {
  const client = createFakeEngineClient();
  await client.loadImage({ width: 1, height: 1, data: new Uint8Array([9, 9, 9, 255]) });
  return client;
}

describe('Downloads', () => {
  it('remembers the slicer choice between visits', async () => {
    const storage = memoryStorage();
    const client = await loadedClient();
    expect(new Downloads({ client, name: 'a' }, { storage }).slicer).toBe('prusa');
    new Downloads({ client, name: 'a' }, { storage }).setSlicer('bambu');
    expect(new Downloads({ client, name: 'a' }, { storage }).slicer).toBe('bambu');
  });

  it('still works when storage is blocked', async () => {
    const blocked: SlicerStorage = {
      getItem: () => {
        throw new Error('blocked');
      },
      setItem: () => {
        throw new Error('blocked');
      },
    };
    const downloads = new Downloads(
      { client: await loadedClient(), name: 'a' },
      { storage: blocked },
    );
    downloads.setSlicer('bambu');
    expect(downloads.slicer).toBe('bambu');
  });

  it('saves the 3MF for the chosen slicer, named after the image', async () => {
    const save = vi.fn();
    const client = await loadedClient();
    const downloads = new Downloads({ client, name: 'jolteon' }, { storage: null, save });
    downloads.setSlicer('bambu');
    await downloads.download('3mf');
    expect(save).toHaveBeenCalledWith(expect.any(Uint8Array), 'jolteon-bambu.3mf', 'model/3mf');
  });

  it('shows STEP as working until its file is ready', async () => {
    let finish: (bytes: Uint8Array) => void = () => undefined;
    const client: EngineClient = {
      ...(await loadedClient()),
      exportStep: () => new Promise((resolve) => (finish = resolve)),
    };
    const save = vi.fn();
    const downloads = new Downloads({ client, name: 'jolteon' }, { storage: null, save });
    const done = downloads.download('step');
    expect(downloads.working).toBe('step');
    finish(new Uint8Array([1]));
    await done;
    expect(downloads.working).toBeNull();
    expect(save).toHaveBeenCalledWith(expect.any(Uint8Array), 'jolteon.step', 'model/step');
  });

  it('reports a failed download', async () => {
    const client: EngineClient = {
      ...(await loadedClient()),
      exportSvg: () => Promise.reject(new Error('no art')),
    };
    const downloads = new Downloads({ client, name: 'x' }, { storage: null, save: vi.fn() });
    await downloads.download('svg');
    expect(downloads.error).toBe('The SVG download failed: no art');
    expect(downloads.working).toBeNull();
  });
});
