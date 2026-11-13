import type { MeshBuffers } from '../engine';

/**
 * Shows the printable model in 3D inside one container. The UI uses only
 * this; `createThreeMeshViewer` is the three.js implementation.
 */
export interface MeshViewer {
  /**
   * Replaces the model with `mesh`, one part per colour, keeping the
   * renderer and, after the first model, the camera.
   */
  show(mesh: MeshBuffers): void;
  /** Frees every GPU resource and removes the canvas. */
  dispose(): void;
}

/** Makes a viewer that draws into `container`, sized to it. */
export type MeshViewerFactory = (container: HTMLElement) => MeshViewer;
