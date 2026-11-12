import type {
  EditLog,
  LoadedImageInfo,
  MeshBuffers,
  Previews,
  RgbaPixels,
  Settings,
  Slicer,
} from './types';

/**
 * The UI's only way into the engine. It holds one open project: loading an
 * image starts a new one. Every call except `version` rejects until an
 * image is loaded.
 *
 * Implementations: `createWorkerEngineClient` (WASM in a worker), and
 * `createFakeEngineClient` for tests. `engineClientContract` checks both.
 */
export interface EngineClient {
  version(): Promise<string>;
  /**
   * Starts over on `pixels`: no edits, and the default settings sized to
   * 100 mm on the art's longest side, which it returns. `pixels.data` may be
   * transferred to the engine, so don't use it afterwards.
   */
  loadImage(pixels: RgbaPixels): Promise<LoadedImageInfo>;
  /** Rejects settings out of range, leaving the current ones. */
  setSettings(settings: Settings): Promise<void>;
  /** Replaces the edit log. Operations that can't apply are listed in the report. */
  applyEdits(log: EditLog): Promise<void>;
  previews(): Promise<Previews>;
  mesh(): Promise<MeshBuffers>;
  /** The 3MF, byte for byte what the CLI writes for the same input. */
  export3mf(slicer: Slicer, name: string): Promise<Uint8Array>;
  exportSvg(name: string): Promise<Uint8Array>;
  /**
   * The STEP file, root part named `name`. The first call also loads the
   * STEP module, which the main engine leaves out to stay small, so it can
   * take a moment longer: show progress while it runs.
   */
  exportStep(name: string): Promise<Uint8Array>;
  dispose(): void;
}
