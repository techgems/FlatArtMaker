import * as Comlink from 'comlink';
import type { EngineClient } from './engine-client';
import type { EngineWorkerApi } from './engine.worker';

/** EngineClient backed by the WASM engine running in a module worker. */
export function createWorkerEngineClient(): EngineClient {
  const worker = new Worker(new URL('./engine.worker.ts', import.meta.url), { type: 'module' });
  const remote = Comlink.wrap<EngineWorkerApi>(worker);

  return {
    version: () => remote.version(),
    dispose: () => {
      remote[Comlink.releaseProxy]();
      worker.terminate();
    },
  };
}
