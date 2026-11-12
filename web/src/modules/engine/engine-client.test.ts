/// <reference types="node" />
import { existsSync, readFileSync } from 'node:fs';
import { describe } from 'vitest';
import { engineClientContract } from './engine-client.contract';
import { createEngineHost } from './engine-host';
import { createFakeEngineClient } from './fake-engine-client';

engineClientContract('fake', createFakeEngineClient);

// The real engine, in-process. It needs the generated package (`npm run wasm`
// or `npm run wasm:dev`), so it is skipped on a checkout that hasn't built it.
const wasmFile = new URL('../../generated/flatart-wasm/flatart_wasm_bg.wasm', import.meta.url);

describe.skipIf(!existsSync(wasmFile))('the WASM engine', () => {
  engineClientContract('WASM engine host', async () => {
    const wasm = await import('../../generated/flatart-wasm/flatart_wasm');
    await wasm.default({ module_or_path: readFileSync(wasmFile) });
    const host = createEngineHost(
      Promise.resolve({ Engine: wasm.Engine, engineVersion: wasm.engineVersion }),
    );
    return { ...host, dispose: () => {} };
  });
});
