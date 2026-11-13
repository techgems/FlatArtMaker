import type { EditLog, Settings } from '../../engine';

/** The current layout of a stored project. Raise it with a migration below. */
export const SCHEMA_VERSION = 1;

/** One project's small data, as JSON in `localStorage`. The image is kept apart. */
export interface ProjectRecord {
  readonly schemaVersion: number;
  readonly id: string;
  readonly name: string;
  readonly createdAt: number;
  readonly updatedAt: number;
  /** Versioned on its own; the engine migrates older settings. */
  readonly settings: Settings;
  readonly edits: EditLog;
}

/** Turns a record of one version into the next. */
export type Migration = (record: Record<string, unknown>) => Record<string, unknown>;

/**
 * `MIGRATIONS[n]` turns a version-n record into version n + 1. Old projects
 * are migrated on load, never dropped. None yet: version 1 is current.
 */
export const MIGRATIONS: Readonly<Partial<Record<number, Migration>>> = {};

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/**
 * A stored record brought up to `current`. Throws if it isn't a project,
 * comes from a newer version of the app, or a migration is missing, so
 * the caller can leave it untouched rather than overwrite it.
 */
export function migrate(
  raw: unknown,
  migrations: Readonly<Partial<Record<number, Migration>>> = MIGRATIONS,
  current: number = SCHEMA_VERSION,
): ProjectRecord {
  if (!isObject(raw) || typeof raw.schemaVersion !== 'number') {
    throw new Error('This isn’t a saved project');
  }
  let record = raw;
  let version = raw.schemaVersion;
  if (version > current) {
    throw new Error(`This project was saved by a newer version of the app (${version})`);
  }
  while (version < current) {
    const step = migrations[version];
    if (step === undefined) throw new Error(`No migration from project version ${version}`);
    record = { ...step(record), schemaVersion: version + 1 };
    version += 1;
  }
  const ok =
    typeof record.id === 'string' &&
    typeof record.name === 'string' &&
    typeof record.createdAt === 'number' &&
    typeof record.updatedAt === 'number' &&
    isObject(record.settings) &&
    isObject(record.edits);
  if (!ok) throw new Error('This saved project is incomplete');
  // The shape is checked above; settings and edits are checked by the
  // engine when the project opens.
  return record as unknown as ProjectRecord;
}
