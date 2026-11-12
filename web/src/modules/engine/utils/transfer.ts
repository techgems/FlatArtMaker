import type { MeshBuffers, Previews } from '../types';

/** A typed array's buffer, for a transfer list. */
export function bufferOf(view: ArrayBufferView): ArrayBuffer {
  // Typed arrays may sit on a SharedArrayBuffer, which can't be transferred.
  if (view.buffer instanceof ArrayBuffer) return view.buffer;
  throw new Error('A shared buffer cannot be transferred');
}

export function previewBuffers(previews: Previews): ArrayBuffer[] {
  return [bufferOf(previews.mapped.labels)];
}

export function meshBuffers(mesh: MeshBuffers): ArrayBuffer[] {
  return mesh.parts.flatMap((part) => [bufferOf(part.positions), bufferOf(part.indices)]);
}
