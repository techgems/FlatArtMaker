import * as Comlink from 'comlink';
import type { EngineClient } from './engine-client';
import type { EngineWorkerApi } from './engine.worker';
import { bufferOf } from './utils/transfer';

/** EngineClient backed by the WASM engine running in a module worker. */
export function createWorkerEngineClient(): EngineClient {
  const worker = new Worker(new URL('./engine.worker.ts', import.meta.url), { type: 'module' });
  const remote = Comlink.wrap<EngineWorkerApi>(worker);

  return {
    version: () => remote.version(),
    loadImage: (pixels) => remote.loadImage(Comlink.transfer(pixels, [bufferOf(pixels.data)])),
    setSettings: (settings) => remote.setSettings(settings),
    applyEdits: (log) => remote.applyEdits(log),
    previews: () => remote.previews(),
    mesh: () => remote.mesh(),
    export3mf: (slicer, name) => remote.export3mf(slicer, name),
    exportSvg: (name) => remote.exportSvg(name),
    exportStep: (name) => remote.exportStep(name),
    dispose: () => {
      remote[Comlink.releaseProxy]();
      worker.terminate();
    },
  };
}
