import { describe, expect, it, vi } from 'vitest';
import type { MeshBuffers } from '../engine';
import { MeshPreview } from './mesh-preview.svelte';
import type { MeshViewer, MeshViewerFactory } from './mesh-viewer';

const mesh = (name: string): MeshBuffers => ({
  parts: [
    {
      kind: 'colour',
      colour: 0,
      name,
      display: { r: 0, g: 0, b: 0 },
      slot: 1,
      positions: new Float32Array(9),
      indices: new Uint32Array([0, 1, 2]),
    },
  ],
});

/** A viewer factory, loaded at once. */
function viewer(): Promise<MeshViewerFactory> {
  return Promise.resolve(() => ({ show: vi.fn(), dispose: vi.fn() }));
}

describe('MeshPreview', () => {
  it('loads the engine’s mesh', async () => {
    const preview = new MeshPreview({ mesh: () => Promise.resolve(mesh('A')) }, viewer);
    expect(preview.phase).toBe('loading');
    await preview.refresh();
    expect(preview.phase).toBe('ready');
    expect(preview.mesh?.parts[0]?.name).toBe('A');
  });

  it('keeps only the newest answer when requests overlap', async () => {
    const answers: ((m: MeshBuffers) => void)[] = [];
    const preview = new MeshPreview(
      { mesh: () => new Promise((resolve) => answers.push(resolve)) },
      viewer,
    );
    const first = preview.refresh();
    const second = preview.refresh();
    answers[1]?.(mesh('new'));
    answers[0]?.(mesh('old'));
    await Promise.all([first, second]);
    expect(preview.mesh?.parts[0]?.name).toBe('new');
  });

  it('reports a mesh the engine can’t build', async () => {
    const preview = new MeshPreview({ mesh: () => Promise.reject(new Error('no art')) }, viewer);
    await preview.refresh();
    expect(preview.phase).toBe('failed');
    expect(preview.error).toBe('no art');
  });

  it('frees the viewer when it’s detached', async () => {
    const dispose = vi.fn();
    const made: MeshViewer = { show: vi.fn(), dispose };
    const preview = new MeshPreview({ mesh: () => Promise.resolve(mesh('A')) }, () =>
      Promise.resolve(() => made),
    );
    const detach = preview.attach({} as HTMLElement);
    await Promise.resolve();
    await Promise.resolve();
    expect(preview.viewer).toBe(made);
    detach();
    expect(dispose).toHaveBeenCalledOnce();
    expect(preview.viewer).toBeNull();
  });

  it('never makes a viewer that was detached while its code loaded', async () => {
    const create = vi.fn<MeshViewerFactory>();
    let load: (factory: MeshViewerFactory) => void = () => undefined;
    const preview = new MeshPreview(
      { mesh: () => Promise.resolve(mesh('A')) },
      () => new Promise((resolve) => (load = resolve)),
    );
    const detach = preview.attach({} as HTMLElement);
    detach();
    load(create);
    await Promise.resolve();
    expect(create).not.toHaveBeenCalled();
    expect(preview.viewer).toBeNull();
  });
});
