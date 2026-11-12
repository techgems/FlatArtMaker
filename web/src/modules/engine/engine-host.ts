import type { Engine } from '../../generated/flatart-wasm/flatart_wasm';
import type { EngineClient } from './engine-client';
import type {
  EditLog,
  LoadedImageInfo,
  MeshBuffers,
  Previews,
  RgbaPixels,
  Settings,
  Slicer,
} from './types';

/** The parts of the WASM package the host uses, once it is initialised. */
export interface EngineModule {
  readonly Engine: new () => Engine;
  engineVersion(): string;
}

/** The engine side of `EngineClient`: everything but `dispose`. */
export type EngineHost = Omit<EngineClient, 'dispose'>;

/**
 * Runs the WASM engine in the current thread. The worker exposes it through
 * Comlink; tests can call it directly. The module is passed in because the
 * browser and Node load the `.wasm` file differently.
 */
export function createEngineHost(module: Promise<EngineModule>): EngineHost {
  let project: Engine | null = null;

  async function open(): Promise<Engine> {
    await module;
    if (project === null) throw new Error('No image is loaded');
    return project;
  }

  return {
    async version(): Promise<string> {
      return (await module).engineVersion();
    },

    async loadImage(pixels: RgbaPixels): Promise<LoadedImageInfo> {
      const { Engine } = await module;
      const next = new Engine();
      try {
        const info = next.loadImage(pixels.data, pixels.width, pixels.height);
        project?.free();
        project = next;
        return info;
      } catch (cause) {
        next.free();
        throw cause;
      }
    },

    async setSettings(settings: Settings): Promise<void> {
      (await open()).setSettings(settings);
    },

    async applyEdits(log: EditLog): Promise<void> {
      (await open()).applyEdits(log);
    },

    async previews(): Promise<Previews> {
      const engine = await open();
      return {
        mapped: { ...engine.mapped(), labels: engine.mappedLabels() },
        vector: engine.vectorModel(),
        report: engine.report(),
      };
    },

    async mesh(): Promise<MeshBuffers> {
      const engine = await open();
      const parts = engine.meshParts().map((part, i) => ({
        ...part,
        positions: engine.meshPositions(i),
        indices: engine.meshIndices(i),
      }));
      return { parts };
    },

    async export3mf(slicer: Slicer, name: string): Promise<Uint8Array> {
      return (await open()).export3mf(slicer, name);
    },

    async exportSvg(name: string): Promise<Uint8Array> {
      return (await open()).exportSvg(name);
    },
  };
}
