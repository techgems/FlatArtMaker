import { decodeImage } from '../engine';
import type { ArtBounds, EditLog, EngineClient, Previews, RgbaPixels, Settings } from '../engine';
import { fileStem } from './utils/file-stem';

export type SessionPhase = 'empty' | 'loading' | 'ready' | 'failed';

/** The user's own work on an image: what a saved project brings back. */
export interface SessionWork {
  readonly settings: Settings;
  readonly edits: EditLog;
}

export interface SessionOptions {
  /** Defaults to the browser decoder; tests pass their own. */
  readonly decode?: (file: Blob) => Promise<RgbaPixels>;
  /** How long settings and edits must stay unchanged before the engine re-runs. */
  readonly delayMs?: number;
  /** Called after every change the user makes, with the work as it now stands. */
  readonly onchange?: (work: SessionWork) => void;
}

/** What the engine gives back after any change. */
interface Results {
  readonly previews: Previews;
  readonly svg: Uint8Array;
}

const NO_EDITS: EditLog = { ops: [], position: 0 };

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/**
 * The image being worked on: its settings and edits, and the engine's
 * latest results. Changes show at once; the engine catches up once they
 * stop changing, and re-runs only the stages they affect.
 */
export class Session {
  phase = $state<SessionPhase>('empty');
  /** A change is waiting or running. */
  updating = $state(false);
  /** The image file's name without its extension; names the downloads. */
  name = $state('');
  imageUrl = $state<string | null>(null);
  svgUrl = $state<string | null>(null);
  bounds = $state.raw<ArtBounds | null>(null);
  settings = $state.raw<Settings | null>(null);
  edits = $state.raw<EditLog>(NO_EDITS);
  previews = $state.raw<Previews | null>(null);
  error = $state<string | null>(null);

  readonly client: EngineClient;
  readonly #decode: (file: Blob) => Promise<RgbaPixels>;
  readonly #delayMs: number;
  readonly #onchange: (work: SessionWork) => void;
  #timer: ReturnType<typeof setTimeout> | undefined;
  /** Bumped by every change; results from an older one are dropped. */
  #generation = 0;

  constructor(client: EngineClient, options: SessionOptions = {}) {
    this.client = client;
    this.#decode = options.decode ?? decodeImage;
    this.#delayMs = options.delayMs ?? 250;
    this.#onchange = options.onchange ?? (() => undefined);
  }

  /** The work as it stands, once an image is open. */
  get work(): SessionWork | null {
    return this.settings === null ? null : { settings: this.settings, edits: this.edits };
  }

  /**
   * Opens `file`: with the default settings and no edits, or with `saved`
   * work. Saved settings the engine no longer accepts give way to the
   * defaults, and the user is told.
   */
  async open(file: File, saved?: SessionWork): Promise<void> {
    this.close();
    const generation = this.#generation;
    this.phase = 'loading';
    this.name = fileStem(file.name);
    this.imageUrl = URL.createObjectURL(file);
    try {
      const info = await this.client.loadImage(await this.#decode(file));
      let settings = info.settings;
      let warning: string | null = null;
      if (saved) {
        try {
          await this.client.setSettings(saved.settings);
          settings = saved.settings;
        } catch (cause) {
          warning = `The saved settings couldn’t be used, so the defaults are back: ${message(cause)}`;
        }
        await this.client.applyEdits(saved.edits);
      }
      const results = await this.#results();
      if (generation !== this.#generation) return;
      this.bounds = info.bounds;
      this.settings = settings;
      this.edits = saved?.edits ?? NO_EDITS;
      this.#show(results);
      this.error = warning;
      this.phase = 'ready';
    } catch (cause) {
      if (generation !== this.#generation) return;
      this.error = message(cause);
      this.phase = 'failed';
    }
  }

  update(settings: Settings): void {
    if (this.phase !== 'ready') return;
    this.settings = settings;
    this.#changed();
  }

  /** Replaces the edit log (colour-mapping edits and the undo position). */
  applyEdits(edits: EditLog): void {
    if (this.phase !== 'ready') return;
    this.edits = edits;
    this.#changed();
  }

  /** Back to the upload screen. */
  close(): void {
    clearTimeout(this.#timer);
    this.#generation++;
    this.#setUrl('imageUrl', null);
    this.#setUrl('svgUrl', null);
    this.phase = 'empty';
    this.updating = false;
    this.name = '';
    this.bounds = null;
    this.settings = null;
    this.edits = NO_EDITS;
    this.previews = null;
    this.error = null;
  }

  #changed(): void {
    this.updating = true;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.#apply(), this.#delayMs);
    const work = this.work;
    if (work) this.#onchange(work);
  }

  /** Sends the current settings and edits; the engine skips what hasn't changed. */
  async #apply(): Promise<void> {
    const generation = ++this.#generation;
    const work = this.work;
    if (work === null) return;
    try {
      await this.client.setSettings(work.settings);
      await this.client.applyEdits(work.edits);
      const results = await this.#results();
      if (generation !== this.#generation) return;
      this.#show(results);
      this.error = null;
    } catch (cause) {
      if (generation === this.#generation) this.error = message(cause);
    } finally {
      if (generation === this.#generation) this.updating = false;
    }
  }

  async #results(): Promise<Results> {
    const previews = await this.client.previews();
    const svg = await this.client.exportSvg(this.name);
    return { previews, svg };
  }

  #show({ previews, svg }: Results): void {
    this.previews = previews;
    // Copied so the bytes sit in a plain ArrayBuffer, which Blob requires.
    const blob = new Blob([new Uint8Array(svg)], { type: 'image/svg+xml' });
    this.#setUrl('svgUrl', URL.createObjectURL(blob));
  }

  #setUrl(key: 'imageUrl' | 'svgUrl', url: string | null): void {
    const old = this[key];
    if (old !== null) URL.revokeObjectURL(old);
    this[key] = url;
  }
}
