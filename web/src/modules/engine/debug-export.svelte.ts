// TEMPORARY (task 14): dev-only check that the browser 3MF matches the CLI's.
// Remove with the debug control once the real export UI exists.
import type { EngineClient } from './engine-client';
import type { Slicer } from './types';
import { decodeImage } from './utils/decode-image';

export type DebugExportPhase = 'idle' | 'working' | 'done' | 'failed';

export interface DebugExportResult {
  readonly fileName: string;
  readonly bytes: number;
  readonly sha256: string;
}

/** Runs image → 3MF in the engine, downloads it and records its size and hash. */
export class DebugExport {
  phase = $state<DebugExportPhase>('idle');
  result = $state.raw<DebugExportResult | null>(null);
  error = $state<string | null>(null);

  readonly #client: EngineClient;

  constructor(client: EngineClient) {
    this.#client = client;
  }

  async run(file: File, slicer: Slicer): Promise<void> {
    this.phase = 'working';
    this.result = null;
    this.error = null;
    try {
      // The CLI names the 3MF object after the input's file stem.
      const stem = fileStem(file.name);
      const pixels = await decodeImage(file);
      await this.#client.loadImage(pixels);
      // Copied so the bytes sit in a plain ArrayBuffer, which digest() and Blob require.
      const bytes = new Uint8Array(await this.#client.export3mf(slicer, stem));
      const fileName = `${stem}-${slicer}.3mf`;
      const sha256 = await sha256Hex(bytes);
      download(bytes, fileName);
      this.result = { fileName, bytes: bytes.byteLength, sha256 };
      this.phase = 'done';
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
      this.phase = 'failed';
    }
  }
}

function fileStem(name: string): string {
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(0, dot) : name;
}

async function sha256Hex(bytes: Uint8Array<ArrayBuffer>): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
  return Array.from(digest, (b) => b.toString(16).padStart(2, '0')).join('');
}

function download(bytes: Uint8Array<ArrayBuffer>, fileName: string): void {
  const url = URL.createObjectURL(new Blob([bytes], { type: 'model/3mf' }));
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.click();
  // Some browsers start the download after click() returns; revoking at once can cancel it.
  setTimeout(() => URL.revokeObjectURL(url), 0);
}
