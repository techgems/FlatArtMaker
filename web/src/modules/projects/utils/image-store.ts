import { StorageFullError } from '../project-store';

/** Where project images are kept: IndexedDB in the browser, a map in tests. */
export interface ImageStore {
  put(id: string, image: Blob): Promise<void>;
  get(id: string): Promise<Blob | undefined>;
  delete(id: string): Promise<void>;
}

/** Quota errors from either store, by the names browsers use. */
export function isQuotaError(cause: unknown): boolean {
  return (
    cause instanceof DOMException &&
    (cause.name === 'QuotaExceededError' || cause.name === 'NS_ERROR_DOM_QUOTA_REACHED')
  );
}

export function memoryImageStore(): ImageStore {
  const images = new Map<string, Blob>();
  return {
    put: (id, image) => Promise.resolve(void images.set(id, image)),
    get: (id) => Promise.resolve(images.get(id)),
    delete: (id) => Promise.resolve(void images.delete(id)),
  };
}

const DATABASE = 'flatart';
const IMAGES = 'images';

function request<T>(req: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error ?? new Error('IndexedDB request failed'));
  });
}

/** Images as `Blob`s in IndexedDB (too big for `localStorage`'s ~5 MB). */
export function indexedDbImageStore(factory: IDBFactory): ImageStore {
  let opened: Promise<IDBDatabase> | null = null;
  const database = (): Promise<IDBDatabase> => {
    opened ??= new Promise((resolve, reject) => {
      const req = factory.open(DATABASE, 1);
      req.onupgradeneeded = () => req.result.createObjectStore(IMAGES);
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => {
        opened = null;
        reject(req.error ?? new Error('IndexedDB is unavailable'));
      };
    });
    return opened;
  };

  async function run<T>(
    mode: IDBTransactionMode,
    work: (store: IDBObjectStore) => IDBRequest<T>,
  ): Promise<T> {
    const tx = (await database()).transaction(IMAGES, mode);
    const done = new Promise<void>((resolve, reject) => {
      tx.oncomplete = () => resolve();
      tx.onabort = () => reject(tx.error ?? new Error('IndexedDB transaction aborted'));
    });
    try {
      const result = await request(work(tx.objectStore(IMAGES)));
      await done;
      return result;
    } catch (cause) {
      throw isQuotaError(cause) ? new StorageFullError({ cause }) : cause;
    }
  }

  return {
    put: async (id, image) => void (await run('readwrite', (s) => s.put(image, id))),
    get: (id) => run('readonly', (s) => s.get(id) as IDBRequest<Blob | undefined>),
    delete: async (id) => void (await run('readwrite', (s) => s.delete(id))),
  };
}
