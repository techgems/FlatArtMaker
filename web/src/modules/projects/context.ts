import { createContext } from 'svelte';
import type { Projects } from './projects.svelte';

export const [getProjects, setProjects] = createContext<Projects>();
