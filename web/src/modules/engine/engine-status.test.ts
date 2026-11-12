import { describe, expect, it } from 'vitest';
import type { EngineClient } from './engine-client';
import { EngineStatus } from './engine-status.svelte';
import { createFakeEngineClient } from './fake-engine-client';

function fakeClient(version: () => Promise<string>): EngineClient {
  return { ...createFakeEngineClient(), version };
}

describe('EngineStatus', () => {
  it('goes from loading to ready with the engine version', async () => {
    const status = new EngineStatus(fakeClient(() => Promise.resolve('9.9.9')));
    expect(status.phase).toBe('loading');

    await status.load();

    expect(status.phase).toBe('ready');
    expect(status.version).toBe('9.9.9');
    expect(status.error).toBeNull();
  });

  it('goes from loading to failed with the error message', async () => {
    const status = new EngineStatus(fakeClient(() => Promise.reject(new Error('wasm missing'))));
    expect(status.phase).toBe('loading');

    await status.load();

    expect(status.phase).toBe('failed');
    expect(status.version).toBeNull();
    expect(status.error).toBe('wasm missing');
  });
});
