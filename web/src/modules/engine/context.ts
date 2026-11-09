import { createContext } from 'svelte';
import type { EngineClient } from './engine-client';
import type { EngineStatus } from './engine-status.svelte';

export const [getEngineClient, setEngineClient] = createContext<EngineClient>();
export const [getEngineStatus, setEngineStatus] = createContext<EngineStatus>();
