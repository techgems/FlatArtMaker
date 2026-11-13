import type { EngineClient, MeshBuffers } from '../engine';
import type { MeshViewer, MeshViewerFactory } from './mesh-viewer';

export type MeshPreviewPhase = 'loading' | 'ready' | 'failed';

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/** The 3D tab: the latest mesh from the engine, and the viewer showing it. */
export class MeshPreview {
  mesh = $state.raw<MeshBuffers | null>(null);
  phase = $state<MeshPreviewPhase>('loading');
  error = $state<string | null>(null);
  viewer = $state.raw<MeshViewer | null>(null);

  readonly #client: Pick<EngineClient, 'mesh'>;
  readonly #loadViewer: () => Promise<MeshViewerFactory>;
  /** Bumped by every request; an older answer is dropped. */
  #request = 0;

  /**
   * `loadViewer` fetches the viewer's code, so three.js loads only when
   * the 3D tab first opens.
   */
  constructor(client: Pick<EngineClient, 'mesh'>, loadViewer: () => Promise<MeshViewerFactory>) {
    this.#client = client;
    this.#loadViewer = loadViewer;
  }

  /** Asks the engine for the current mesh. The last one stays on screen meanwhile. */
  async refresh(): Promise<void> {
    const request = ++this.#request;
    if (this.mesh === null) this.phase = 'loading';
    try {
      const mesh = await this.#client.mesh();
      if (request !== this.#request) return;
      this.mesh = mesh;
      this.error = null;
      this.phase = 'ready';
    } catch (cause) {
      if (request !== this.#request) return;
      this.error = message(cause);
      this.phase = 'failed';
    }
  }

  /**
   * Puts a viewer in `container` once its code has loaded; the returned
   * function frees it, or cancels it if it hasn't loaded yet.
   */
  attach(container: HTMLElement): () => void {
    let viewer: MeshViewer | null = null;
    let detached = false;
    this.#loadViewer().then(
      (create) => {
        if (detached) return;
        viewer = create(container);
        this.viewer = viewer;
      },
      (cause: unknown) => {
        if (detached) return;
        this.error = `The 3D viewer couldn’t load: ${message(cause)}`;
        this.phase = 'failed';
      },
    );
    return () => {
      detached = true;
      if (viewer === null) return;
      if (this.viewer === viewer) this.viewer = null;
      viewer.dispose();
    };
  }
}
