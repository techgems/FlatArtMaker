import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { EditLog, Settings } from '../engine';
import {
  MemoryStorage,
  createLocalProjectStore,
  createMemoryProjectStore,
} from './local-project-store';
import { StorageFullError, type ProjectWork } from './project-store';
import { Projects } from './projects.svelte';
import { memoryImageStore } from './utils/image-store';
import { migrate } from './utils/record';

function settings(colourHeight = 1): Settings {
  return {
    version: 1,
    colourHeight,
    baseHeight: undefined,
    scale: 0.25,
    minFeature: 0.4,
    load: { alphaThreshold: 128, background: { r: 255, g: 255, b: 255 } },
    advanced: {
      detectedColours: 20,
      outputColours: 5,
      speckleArea: 30,
      traceUpscale: 3,
      smoothing: 0.75,
      corner: 45,
      curveTolerance: 0.01,
      fillBaseHoles: false,
    },
  };
}

/** A mapping, a painted area and a rename, with the last one undone. */
const edits: EditLog = {
  ops: [
    { op: 'map', detected: { l: 0.5, a: 0.1, b: -0.1 }, output: 2 },
    { op: 'paintRegion', at: { x: 120.5, y: 33 }, colour: 1 },
    { op: 'rename', id: 2, name: 'Sky' },
  ],
  position: 2,
};

const work = (colourHeight = 1): ProjectWork => ({ settings: settings(colourHeight), edits });
const image = new Blob([new Uint8Array([137, 80, 78, 71])], { type: 'image/png' });

describe('project store', () => {
  it('loads exactly the settings and edits it saved, with the image', async () => {
    const store = createMemoryProjectStore();
    const { id } = await store.create({ name: 'jolteon.png', image, ...work() });
    await store.save(id, work(2.5));
    const loaded = await store.load(id);
    expect(loaded.settings).toEqual(settings(2.5));
    expect(loaded.edits).toEqual(edits);
    expect(loaded.name).toBe('jolteon.png');
    expect(loaded.image.type).toBe('image/png');
    expect(loaded.image.size).toBe(4);
  });

  it('lists the most recently changed first, and renames and deletes', async () => {
    let time = 0;
    const store = createMemoryProjectStore({ now: () => ++time });
    const a = await store.create({ name: 'a', image, ...work() });
    const b = await store.create({ name: 'b', image, ...work() });
    expect((await store.list()).map((p) => p.name)).toEqual(['b', 'a']);
    await store.save(a.id, work(3));
    await store.rename(b.id, 'b2');
    expect((await store.list()).map((p) => p.name)).toEqual(['a', 'b2']);
    await store.remove(a.id);
    expect((await store.list()).map((p) => p.name)).toEqual(['b2']);
    await expect(store.load(a.id)).rejects.toThrow(/no longer exists/);
  });

  it('turns a full quota into StorageFullError and leaves nothing half-made', async () => {
    const storage = new MemoryStorage();
    const images = memoryImageStore();
    const store = createLocalProjectStore({ storage, images, newId: () => 'p1' });
    storage.full = true;
    await expect(store.create({ name: 'a', image, ...work() })).rejects.toBeInstanceOf(
      StorageFullError,
    );
    expect(await images.get('p1')).toBeUndefined();
    storage.full = false;
    expect(await store.list()).toEqual([]);
  });
});

describe('project records', () => {
  const v1 = {
    schemaVersion: 1,
    id: 'p1',
    name: 'old',
    createdAt: 1,
    updatedAt: 2,
    settings: settings(),
    edits,
  };

  it('reads a version 1 project as it was saved', () => {
    expect(migrate(v1)).toEqual(v1);
  });

  it('migrates a version 1 project through each step in turn', () => {
    const steps = {
      1: (r: Record<string, unknown>) => ({ ...r, name: `${String(r.name)} (v2)` }),
      2: (r: Record<string, unknown>) => ({ ...r, name: `${String(r.name)} (v3)` }),
    };
    const migrated = migrate(v1, steps, 3);
    expect(migrated.schemaVersion).toBe(3);
    expect(migrated.name).toBe('old (v2) (v3)');
    expect(migrated.settings).toEqual(v1.settings);
  });

  it('refuses a project from a newer app, or with a missing step', () => {
    expect(() => migrate({ ...v1, schemaVersion: 9 })).toThrow(/newer version/);
    expect(() => migrate(v1, {}, 2)).toThrow(/No migration from project version 1/);
    expect(() => migrate({ hello: 1 })).toThrow(/isn’t a saved project/);
  });
});

describe('Projects', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('saves once, about 500 ms after the last change', async () => {
    const store = createMemoryProjectStore();
    const save = vi.spyOn(store, 'save');
    const projects = new Projects(store);
    await projects.create('jolteon.png', image, work());
    for (const height of [2, 3, 4]) projects.changed(work(height));
    await vi.advanceTimersByTimeAsync(499);
    expect(save).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(save).toHaveBeenCalledOnce();
    const id = projects.currentId ?? '';
    expect((await store.load(id)).settings.colourHeight).toBe(4);
  });

  it('when storage is full, says so, keeps the work, and saves once space is freed', async () => {
    const storage = new MemoryStorage();
    const store = createMemoryProjectStore({ storage });
    const projects = new Projects(store);
    await projects.create('old.png', image, work());
    const old = projects.currentId ?? '';
    await projects.create('jolteon.png', image, work());
    const current = projects.currentId ?? '';

    storage.full = true;
    projects.changed(work(7));
    await vi.runAllTimersAsync();
    expect(projects.notice).toBe('full');
    expect(projects.error).toBeNull();
    expect(projects.currentId).toBe(current);

    storage.full = false;
    await projects.remove(old);
    expect(projects.notice).toBeNull();
    expect((await store.load(current)).settings.colourHeight).toBe(7);
    expect(projects.recent.map((p) => p.name)).toEqual(['jolteon.png']);
  });

  it('when storage is unavailable, says so and carries on without saving', async () => {
    const projects = new Projects(createMemoryProjectStore({ available: false }));
    expect(projects.notice).toBe('unavailable');
    await projects.create('jolteon.png', image, work());
    projects.changed(work(2));
    await vi.runAllTimersAsync();
    await projects.refresh();
    expect(projects.currentId).toBeNull();
    expect(projects.recent).toEqual([]);
    expect(projects.error).toBeNull();
  });
});
