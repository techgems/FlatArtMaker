import type { Slicer } from '../../engine';

/** Under the app's `flatart:` prefix (SPEC "Saved projects"). */
const KEY = 'flatart:slicer';

export const SLICERS: ReadonlyArray<{ id: Slicer; label: string }> = [
  { id: 'prusa', label: 'PrusaSlicer' },
  { id: 'bambu', label: 'Bambu Studio / Orca' },
];

/** The slice of `Storage` the choice needs, so tests can pass a fake. */
export type SlicerStorage = Pick<Storage, 'getItem' | 'setItem'>;

/** The browser's storage, or `null` where it's blocked (private windows, settings). */
export function browserStorage(): SlicerStorage | null {
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function isSlicer(value: string | null): value is Slicer {
  return SLICERS.some((s) => s.id === value);
}

/** The last choice, or PrusaSlicer. */
export function loadSlicer(storage: SlicerStorage | null): Slicer {
  try {
    const saved = storage?.getItem(KEY) ?? null;
    return isSlicer(saved) ? saved : 'prusa';
  } catch {
    return 'prusa';
  }
}

/** Best effort: if storage is unavailable or full, the choice lasts this visit only. */
export function saveSlicer(storage: SlicerStorage | null, slicer: Slicer): void {
  try {
    storage?.setItem(KEY, slicer);
  } catch {
    // Nothing to do: the in-memory choice still applies.
  }
}
