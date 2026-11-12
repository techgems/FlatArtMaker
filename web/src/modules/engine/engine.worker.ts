import * as Comlink from 'comlink';
import init, { Engine, engineVersion } from '../../generated/flatart-wasm/flatart_wasm';
import { createEngineHost } from './engine-host';
import type { EditLog, RgbaPixels, Settings, Slicer } from './types';
import { bufferOf, meshBuffers, previewBuffers } from './utils/transfer';

// Instantiate the main WASM module once; every call awaits it. The STEP
// module is a separate chunk, fetched on the first STEP export.
const host = createEngineHost(
  init().then(() => ({ Engine, engineVersion })),
  async () => {
    const step = await import('../../generated/flatart-wasm-step/flatart_wasm_step');
    await step.default();
    return { exportStep: step.exportStep };
  },
);

// Results with big typed arrays hand their buffers over instead of copying.
const api = {
  version: () => host.version(),
  loadImage: (pixels: RgbaPixels) => host.loadImage(pixels),
  setSettings: (settings: Settings) => host.setSettings(settings),
  applyEdits: (log: EditLog) => host.applyEdits(log),
  async previews() {
    const previews = await host.previews();
    return Comlink.transfer(previews, previewBuffers(previews));
  },
  async mesh() {
    const mesh = await host.mesh();
    return Comlink.transfer(mesh, meshBuffers(mesh));
  },
  async export3mf(slicer: Slicer, name: string) {
    const bytes = await host.export3mf(slicer, name);
    return Comlink.transfer(bytes, [bufferOf(bytes)]);
  },
  async exportSvg(name: string) {
    const bytes = await host.exportSvg(name);
    return Comlink.transfer(bytes, [bufferOf(bytes)]);
  },
  async exportStep(name: string) {
    const bytes = await host.exportStep(name);
    return Comlink.transfer(bytes, [bufferOf(bytes)]);
  },
};

export type EngineWorkerApi = typeof api;

Comlink.expose(api);
