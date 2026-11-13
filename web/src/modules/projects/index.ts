// Public surface of the projects module. Other modules import only from here.
export type {
  NewProject,
  ProjectStore,
  ProjectSummary,
  ProjectWork,
  SavedProject,
} from './project-store';
export { StorageFullError, StorageUnavailableError } from './project-store';
export { createBrowserProjectStore, createMemoryProjectStore } from './local-project-store';
export { Projects } from './projects.svelte';
export { getProjects, setProjects } from './context';
export { default as RecentProjects } from './components/RecentProjects.svelte';
export { default as StorageNotice } from './components/StorageNotice.svelte';
