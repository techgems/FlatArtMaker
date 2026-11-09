import * as Comlink from 'comlink';
import init, { engineVersion } from '../../generated/flatart-wasm/flatart_wasm';

// Instantiate the WASM module once; every call awaits it.
const ready = init();

const api = {
  async version(): Promise<string> {
    await ready;
    return engineVersion();
  },
};

export type EngineWorkerApi = typeof api;

Comlink.expose(api);
