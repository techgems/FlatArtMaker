import { decodeImage } from '../engine';
import type { ArtBounds, EngineClient, Previews, RgbaPixels, Settings } from '../engine';
import { fileStem } from './utils/file-stem';

export type SessionPhase = 'empty' | 'loading' | 'ready' | 'failed';

export interface SessionOptions {
  /** Defaults to the browser decoder; tests pass their own. */
  readonly decode?: (file: Blob) => Promise<RgbaPixels>;
  /** How long settings must stay unchanged before the engine re-runs. */
  readonly delayMs?: number;
}

/** What the engine gives back after any change. */
interface Results {
  readonly previews: Previews;
  readonly svg: Uint8Array;
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/**
 * The image being worked on: its settings and the engine's latest results.
 * Settings show at once; the engine catches up once they stop changing,
 * and re-runs only the stages they affect.
 */
export class Session {
  phase = $state<SessionPhase>('empty');
  /** A settings change is waiting or running. */
  updating = $state(false);
  /** The image file's name without its extension; names the downloads. */
  name = $state('');
  imageUrl = $state<string | null>(null);
  svgUrl = $state<string | null>(null);
  bounds = $state.raw<ArtBounds | null>(null);
  settings = $state.raw<Settings | null>(null);
  previews = $state.raw<Previews | null>(null);
  error = $state<string | null>(null);

  readonly client: EngineClient;
  readonly #decode: (file: Blob) => Promise<RgbaPixels>;
  readonly #delayMs: number;
  #timer: ReturnType<typeof setTimeout> | undefined;
  /** Bumped by every change; results from an older one are dropped. */
  #generation = 0;

  constructor(client: EngineClient, options: SessionOptions = {}) {
    this.client = client;
    this.#decode = options.decode ?? decodeImage;
    this.#delayMs = options.delayMs ?? 250;
  }

  async open(file: File): Promise<void> {
    this.close();
    const generation = this.#generation;
    this.phase = 'loading';
    this.name = fileStem(file.name);
    this.imageUrl = URL.createObjectURL(file);
    try {
      const info = await this.client.loadImage(await this.#decode(file));
      const results = await this.#results();
      if (generation !== this.#generation) return;
      this.bounds = info.bounds;
      this.settings = info.settings;
      this.#show(results);
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
    this.updating = true;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.#apply(settings), this.#delayMs);
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
    this.previews = null;
    this.error = null;
  }

  async #apply(settings: Settings): Promise<void> {
    const generation = ++this.#generation;
    try {
      await this.client.setSettings(settings);
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
