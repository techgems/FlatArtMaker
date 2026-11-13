import type { EngineClient, Slicer } from '../engine';
import { saveFile } from './utils/save-file';
import { browserStorage, loadSlicer, saveSlicer, type SlicerStorage } from './utils/slicer-choice';

export type DownloadFormat = '3mf' | 'step' | 'svg';

/** What the downloads need from the session. */
export interface DownloadSource {
  readonly client: EngineClient;
  /** Names the files, and the 3MF object and STEP root inside them. */
  readonly name: string;
}

export interface DownloadOptions {
  readonly storage?: SlicerStorage | null;
  readonly save?: (bytes: Uint8Array, fileName: string, mimeType: string) => void;
}

interface Exported {
  readonly bytes: Uint8Array;
  readonly fileName: string;
  readonly mimeType: string;
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/** The header's downloads: one at a time, and the slicer choice, remembered between visits. */
export class Downloads {
  slicer = $state<Slicer>('prusa');
  /** The download being made, if any. */
  working = $state<DownloadFormat | null>(null);
  error = $state<string | null>(null);

  readonly #source: DownloadSource;
  readonly #storage: SlicerStorage | null;
  readonly #save: (bytes: Uint8Array, fileName: string, mimeType: string) => void;

  constructor(source: DownloadSource, options: DownloadOptions = {}) {
    this.#source = source;
    this.#storage = options.storage === undefined ? browserStorage() : options.storage;
    this.#save = options.save ?? saveFile;
    this.slicer = loadSlicer(this.#storage);
  }

  setSlicer(slicer: Slicer): void {
    this.slicer = slicer;
    saveSlicer(this.#storage, slicer);
  }

  async download(format: DownloadFormat): Promise<void> {
    if (this.working !== null) return;
    this.working = format;
    this.error = null;
    try {
      const { bytes, fileName, mimeType } = await this.#export(format);
      this.#save(bytes, fileName, mimeType);
    } catch (cause) {
      this.error = `The ${format.toUpperCase()} download failed: ${message(cause)}`;
    } finally {
      this.working = null;
    }
  }

  async #export(format: DownloadFormat): Promise<Exported> {
    const { client, name } = this.#source;
    switch (format) {
      case '3mf':
        return {
          bytes: await client.export3mf(this.slicer, name),
          fileName: `${name}-${this.slicer}.3mf`,
          mimeType: 'model/3mf',
        };
      case 'step':
        return {
          bytes: await client.exportStep(name),
          fileName: `${name}.step`,
          mimeType: 'model/step',
        };
      case 'svg':
        return {
          bytes: await client.exportSvg(name),
          fileName: `${name}.svg`,
          mimeType: 'image/svg+xml',
        };
      default: {
        const unknown: never = format;
        throw new Error(`Unknown format ${String(unknown)}`);
      }
    }
  }
}
