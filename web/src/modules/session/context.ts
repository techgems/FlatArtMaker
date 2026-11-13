import { createContext } from 'svelte';
import type { Session } from './session.svelte';

export const [getSession, setSession] = createContext<Session>();
