// Public surface of the engine module. Other modules import only from here.
export type { EngineClient } from './engine-client';
export type * from './types';
export { createWorkerEngineClient } from './worker-engine-client';
export { createFakeEngineClient } from './fake-engine-client';
export { decodeImage } from './utils/decode-image';
export { EngineStatus, type EnginePhase } from './engine-status.svelte';
export { getEngineClient, setEngineClient, getEngineStatus, setEngineStatus } from './context';
export { default as EngineBadge } from './components/EngineBadge.svelte';
// TEMPORARY (task 14): dev-only control to check the browser 3MF against the CLI's.
export { default as DebugExport } from './components/DebugExport.svelte';
