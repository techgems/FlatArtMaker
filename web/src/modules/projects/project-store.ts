import type { EditLog, Settings } from '../engine';

/** What the Recent projects list shows. Times are milliseconds since 1970. */
export interface ProjectSummary {
  readonly id: string;
  readonly name: string;
  readonly createdAt: number;
  readonly updatedAt: number;
}

/** The user's work: everything else is recomputed from it and the image. */
export interface ProjectWork {
  readonly settings: Settings;
  /** The colour-mapping edits, with the undo position. */
  readonly edits: EditLog;
}

export interface NewProject extends ProjectWork {
  readonly name: string;
  /** The uploaded file, unchanged; its `type` is the MIME type. */
  readonly image: Blob;
}

export interface SavedProject extends ProjectSummary, ProjectWork {
  readonly image: Blob;
}

/**
 * Where projects are kept (SPEC "Saved projects"). Derived data (label
 * maps, meshes, exports) is never stored.
 *
 * Every call may reject with `StorageFullError`, or with
 * `StorageUnavailableError` when `available` is false.
 */
export interface ProjectStore {
  /** False where the browser blocks storage (private windows, settings). */
  readonly available: boolean;
  /** Newest change first. */
  list(): Promise<ProjectSummary[]>;
  create(project: NewProject): Promise<ProjectSummary>;
  /** Saves the work and moves the project to the top of the list. */
  save(id: string, work: ProjectWork): Promise<void>;
  rename(id: string, name: string): Promise<void>;
  load(id: string): Promise<SavedProject>;
  remove(id: string): Promise<void>;
}

/** The browser's storage quota is used up. */
export class StorageFullError extends Error {
  constructor(options?: ErrorOptions) {
    super('Browser storage is full', options);
    this.name = 'StorageFullError';
  }
}

/** The browser won't let the app store anything. */
export class StorageUnavailableError extends Error {
  constructor(options?: ErrorOptions) {
    super('Browser storage is unavailable', options);
    this.name = 'StorageUnavailableError';
  }
}
