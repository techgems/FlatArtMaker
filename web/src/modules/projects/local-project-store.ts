import {
  StorageFullError,
  StorageUnavailableError,
  type ProjectStore,
  type ProjectSummary,
  type ProjectWork,
  type SavedProject,
} from './project-store';
import {
  indexedDbImageStore,
  isQuotaError,
  memoryImageStore,
  type ImageStore,
} from './utils/image-store';
import { SCHEMA_VERSION, migrate, type ProjectRecord } from './utils/record';

/** The slice of `Storage` the store uses, so tests can pass their own. */
export type KeyValueStore = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;

export interface LocalStoreParts {
  /** `null` where storage is blocked. */
  readonly storage: KeyValueStore | null;
  readonly images: ImageStore | null;
  readonly now?: () => number;
  readonly newId?: () => string;
}

/** Every key sits under the app's prefix (SPEC "Saved projects"). */
const INDEX_KEY = 'flatart:projects';
const recordKey = (id: string): string => `flatart:project:${id}`;

/**
 * The project store: the list, names, settings and edits as JSON in
 * `storage`, and each image in `images`.
 */
export function createLocalProjectStore(parts: LocalStoreParts): ProjectStore {
  const { storage, images } = parts;
  const now = parts.now ?? Date.now;
  const newId = parts.newId ?? (() => crypto.randomUUID());
  const available = storage !== null && images !== null;

  function backends(): { storage: KeyValueStore; images: ImageStore } {
    if (storage === null || images === null) throw new StorageUnavailableError();
    return { storage, images };
  }

  function write(key: string, value: unknown): void {
    try {
      backends().storage.setItem(key, JSON.stringify(value));
    } catch (cause) {
      throw isQuotaError(cause) ? new StorageFullError({ cause }) : cause;
    }
  }

  function readIndex(): ProjectSummary[] {
    const raw = backends().storage.getItem(INDEX_KEY);
    if (raw === null) return [];
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as ProjectSummary[]) : [];
  }

  function writeIndex(list: readonly ProjectSummary[]): void {
    write(
      INDEX_KEY,
      [...list].sort((a, b) => b.updatedAt - a.updatedAt),
    );
  }

  function readRecord(id: string): ProjectRecord {
    const raw = backends().storage.getItem(recordKey(id));
    if (raw === null) throw new Error('That project no longer exists');
    return migrate(JSON.parse(raw));
  }

  function summary(record: ProjectRecord): ProjectSummary {
    const { id, name, createdAt, updatedAt } = record;
    return { id, name, createdAt, updatedAt };
  }

  /** Writes `record` and its place in the list. */
  function store(record: ProjectRecord): void {
    write(recordKey(record.id), record);
    writeIndex([...readIndex().filter((p) => p.id !== record.id), summary(record)]);
  }

  return {
    available,

    list: () => Promise.resolve().then(readIndex),

    async create(project) {
      const time = now();
      const record: ProjectRecord = {
        schemaVersion: SCHEMA_VERSION,
        id: newId(),
        name: project.name,
        createdAt: time,
        updatedAt: time,
        settings: project.settings,
        edits: project.edits,
      };
      const { images } = backends();
      await images.put(record.id, project.image);
      try {
        store(record);
      } catch (cause) {
        // Leave no orphaned image behind.
        await images.delete(record.id).catch(() => undefined);
        backends().storage.removeItem(recordKey(record.id));
        throw cause;
      }
      return summary(record);
    },

    async save(id: string, work: ProjectWork) {
      await Promise.resolve();
      store({ ...readRecord(id), ...work, updatedAt: now() });
    },

    async rename(id, name) {
      await Promise.resolve();
      store({ ...readRecord(id), name });
    },

    async load(id): Promise<SavedProject> {
      const record = readRecord(id);
      const image = await backends().images.get(id);
      if (image === undefined) throw new Error('That project’s image is missing');
      return { ...summary(record), settings: record.settings, edits: record.edits, image };
    },

    async remove(id) {
      const { storage, images } = backends();
      storage.removeItem(recordKey(id));
      writeIndex(readIndex().filter((p) => p.id !== id));
      await images.delete(id);
    },
  };
}

/** The browser's store, or an unavailable one where storage is blocked. */
export function createBrowserProjectStore(): ProjectStore {
  return createLocalProjectStore(browserBackends() ?? { storage: null, images: null });
}

function browserBackends(): { storage: KeyValueStore; images: ImageStore } | null {
  try {
    // Reading the property throws where site data is blocked.
    const storage = window.localStorage;
    const probe = 'flatart:probe';
    storage.setItem(probe, '1');
    storage.removeItem(probe);
    return { storage, images: indexedDbImageStore(window.indexedDB) };
  } catch {
    return null;
  }
}

/** A `Storage` in memory that can be made to run out of space. */
export class MemoryStorage implements KeyValueStore {
  readonly items = new Map<string, string>();
  full = false;

  getItem(key: string): string | null {
    return this.items.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    if (this.full) throw new DOMException('The quota has been exceeded.', 'QuotaExceededError');
    this.items.set(key, value);
  }

  removeItem(key: string): void {
    this.items.delete(key);
  }
}

/** An in-memory store for tests; `available: false` acts like blocked storage. */
export function createMemoryProjectStore(
  options: { storage?: MemoryStorage; available?: boolean; now?: () => number } = {},
): ProjectStore {
  const available = options.available ?? true;
  let counter = 0;
  return createLocalProjectStore({
    storage: available ? (options.storage ?? new MemoryStorage()) : null,
    images: available ? memoryImageStore() : null,
    now: options.now,
    newId: () => `project-${++counter}`,
  });
}
