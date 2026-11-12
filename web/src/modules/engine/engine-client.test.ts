/// <reference types="node" />
import { existsSync, readFileSync } from 'node:fs';
import { describe } from 'vitest';
import { engineClientContract } from './engine-client.contract';
import { createEngineHost } from './engine-host';
import { createFakeEngineClient } from './fake-engine-client';

engineClientContract('fake', createFakeEngineClient);

// The real engines, in-process. They need the generated packages (`npm run
// wasm` or `npm run wasm:dev`), so they are skipped on a checkout that hasn't
// built them.
const wasmFile = new URL('../../generated/flatart-wasm/flatart_wasm_bg.wasm', import.meta.url);
const stepFile = new URL(
  '../../generated/flatart-wasm-step/flatart_wasm_step_bg.wasm',
  import.meta.url,
);

describe.skipIf(!existsSync(wasmFile) || !existsSync(stepFile))('the WASM engine', () => {
  engineClientContract('WASM engine host', async () => {
    const wasm = await import('../../generated/flatart-wasm/flatart_wasm');
    await wasm.default({ module_or_path: readFileSync(wasmFile) });
    const host = createEngineHost(
      Promise.resolve({ Engine: wasm.Engine, engineVersion: wasm.engineVersion }),
      async () => {
        const step = await import('../../generated/flatart-wasm-step/flatart_wasm_step');
        await step.default({ module_or_path: readFileSync(stepFile) });
        return { exportStep: step.exportStep };
      },
    );
    return { ...host, dispose: () => {} };
  });
});
