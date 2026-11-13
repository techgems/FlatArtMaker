import {
  StorageFullError,
  StorageUnavailableError,
  type ProjectStore,
  type ProjectSummary,
  type ProjectWork,
  type SavedProject,
} from './project-store';

/** Why projects aren't being kept, if they aren't. */
export type StorageNotice = 'full' | 'unavailable' | null;

export interface ProjectsOptions {
  /** How long work must stay unchanged before it's saved. */
  readonly delayMs?: number;
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/**
 * Saved projects: the Recent list, the project being worked on, and its
 * autosave. Storage trouble becomes a notice; the work itself carries on.
 */
export class Projects {
  recent = $state.raw<readonly ProjectSummary[]>([]);
  /** The open project, which changes are saved to. */
  currentId = $state<string | null>(null);
  notice = $state<StorageNotice>(null);
  error = $state<string | null>(null);
  /** The "Delete old projects" dialog is open. */
  managing = $state(false);

  readonly #store: ProjectStore;
  readonly #delayMs: number;
  #timer: ReturnType<typeof setTimeout> | undefined;
  /** The latest work not yet saved. */
  #pending: ProjectWork | null = null;

  constructor(store: ProjectStore, options: ProjectsOptions = {}) {
    this.#store = store;
    this.#delayMs = options.delayMs ?? 500;
    if (!store.available) this.notice = 'unavailable';
  }

  async refresh(): Promise<void> {
    if (!this.#store.available) return;
    await this.#attempt(async () => {
      this.recent = await this.#store.list();
    });
  }

  /** Starts a project for a newly opened image, named after its file. */
  async create(name: string, image: Blob, work: ProjectWork): Promise<void> {
    this.#forget();
    if (!this.#store.available) return;
    await this.#attempt(async () => {
      const created = await this.#store.create({ name, image, ...work });
      this.currentId = created.id;
      await this.refresh();
    });
  }

  /** A saved project to reopen, or `null` if it can't be read. */
  async load(id: string): Promise<SavedProject | null> {
    this.#forget();
    let project: SavedProject | null = null;
    await this.#attempt(async () => {
      project = await this.#store.load(id);
    });
    return project;
  }

  /** Makes `id` the open project, once its work is back on screen. */
  resume(id: string): void {
    this.currentId = id;
  }

  /** The work changed: save it once it settles. */
  changed(work: ProjectWork): void {
    if (this.currentId === null || !this.#store.available) return;
    this.#pending = work;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.flush(), this.#delayMs);
  }

  /** Saves pending work now. */
  async flush(): Promise<void> {
    clearTimeout(this.#timer);
    const [id, work] = [this.currentId, this.#pending];
    if (id === null || work === null) return;
    const saved = await this.#attempt(() => this.#store.save(id, work));
    if (saved) {
      if (this.#pending === work) this.#pending = null;
      if (this.notice === 'full') this.notice = null;
      await this.refresh();
    }
  }

  /** Saves what's pending and stops saving: the image was closed. */
  async close(): Promise<void> {
    await this.flush();
    this.#forget();
    await this.refresh();
  }

  async rename(id: string, name: string): Promise<void> {
    const trimmed = name.trim();
    if (trimmed === '') return;
    await this.#attempt(() => this.#store.rename(id, trimmed));
    await this.refresh();
  }

  /** Deletes a project. Space freed while storage was full retries the save. */
  async remove(id: string): Promise<void> {
    if (id === this.currentId) this.#forget();
    const removed = await this.#attempt(() => this.#store.remove(id));
    await this.refresh();
    if (removed && this.notice === 'full') await this.flush();
  }

  #forget(): void {
    clearTimeout(this.#timer);
    this.#pending = null;
    this.currentId = null;
  }

  /** Runs `work`, turning storage errors into notices. True if it succeeded. */
  async #attempt(work: () => Promise<void>): Promise<boolean> {
    try {
      await work();
      this.error = null;
      return true;
    } catch (cause) {
      if (cause instanceof StorageFullError) this.notice = 'full';
      else if (cause instanceof StorageUnavailableError) this.notice = 'unavailable';
      else this.error = message(cause);
      return false;
    }
  }
}
