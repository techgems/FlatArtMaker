import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createFakeEngineClient } from '../engine';
import type { RgbaPixels, Settings } from '../engine';
import { Session } from './session.svelte';

/** Two opaque pixels, red and blue, whatever the file. */
function decode(): Promise<RgbaPixels> {
  return Promise.resolve({
    width: 2,
    height: 1,
    data: new Uint8Array([255, 0, 0, 255, 0, 0, 255, 255]),
  });
}

const file = new File([new Uint8Array([1])], 'jolteon.png', { type: 'image/png' });

async function opened(): Promise<Session> {
  const session = new Session(createFakeEngineClient(), { decode, delayMs: 100 });
  await session.open(file);
  return session;
}

function withHeight(settings: Settings | null, colourHeight: number): Settings {
  if (settings === null) throw new Error('no settings');
  return { ...settings, colourHeight };
}

describe('Session', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('opens an image with its default settings and previews', async () => {
    const session = await opened();
    expect(session.phase).toBe('ready');
    expect(session.name).toBe('jolteon');
    expect(session.bounds).toEqual({ x: 0, y: 0, width: 2, height: 1 });
    expect(session.settings?.colourHeight).toBe(1);
    expect(session.previews?.mapped.palette.colours).toHaveLength(2);
    expect(session.svgUrl).not.toBeNull();
  });

  it('applies only the last of quick settings changes, after a pause', async () => {
    const session = await opened();
    const setSettings = vi.spyOn(session.client, 'setSettings');
    for (const height of [2, 3, 4]) session.update(withHeight(session.settings, height));

    expect(session.settings?.colourHeight).toBe(4);
    expect(session.updating).toBe(true);
    expect(setSettings).not.toHaveBeenCalled();

    await vi.runAllTimersAsync();
    expect(setSettings).toHaveBeenCalledOnce();
    expect(setSettings).toHaveBeenCalledWith(expect.objectContaining({ colourHeight: 4 }));
    expect(session.updating).toBe(false);
  });

  it('reports settings the engine rejects', async () => {
    const session = await opened();
    session.update(withHeight(session.settings, 0));
    await vi.runAllTimersAsync();
    expect(session.error).toMatch(/layer height/);
    expect(session.phase).toBe('ready');
  });

  it('fails with the reason when the image can’t be read', async () => {
    const session = new Session(createFakeEngineClient(), {
      decode: () => Promise.reject(new Error('not an image')),
    });
    await session.open(file);
    expect(session.phase).toBe('failed');
    expect(session.error).toBe('not an image');
  });

  it('closes back to the upload screen', async () => {
    const session = await opened();
    session.close();
    expect(session.phase).toBe('empty');
    expect(session.imageUrl).toBeNull();
    expect(session.settings).toBeNull();
  });
});
