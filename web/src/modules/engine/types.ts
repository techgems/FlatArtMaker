// Types shared with the engine. The Rust-side ones are generated (tsify) and
// only re-exported here; the rest add the typed-array buffers that cross the
// worker boundary separately, so they can be transferred instead of copied.
import type {
  MappedInfo,
  MeshPartInfo,
  Report,
  VectorModel,
} from '../../generated/flatart-wasm/flatart_wasm';

export type {
  Advanced,
  ArtBounds,
  Border,
  BorderMap,
  CubicBezier,
  DetectedColour,
  DetectedPalette,
  DropReason,
  DroppedEdit,
  EditLog,
  EditOp,
  Face,
  LoadedImageInfo,
  MappedInfo,
  MeshPartInfo,
  Oklab,
  OutputColour,
  OutputColourId,
  OutputPalette,
  PartKind,
  PxPoint,
  Report,
  Rgb,
  Settings,
  SimilarityTree,
  Slicer,
  ThinFeature,
  VectorModel,
} from '../../generated/flatart-wasm/flatart_wasm';

/** A decoded image: 4 bytes per pixel (RGBA, straight alpha), row by row. */
export interface RgbaPixels {
  readonly width: number;
  readonly height: number;
  readonly data: Uint8Array;
}

/** The colour-mapping result with its output label map. */
export interface MappedPreview extends MappedInfo {
  /** One cell per source pixel: an `OutputColourId`, or `outside`. */
  readonly labels: Uint16Array;
}

/** What the 2D previews and warnings need after any change. */
export interface Previews {
  readonly mapped: MappedPreview;
  readonly vector: VectorModel;
  readonly report: Report;
}

/** One printable part, ready for a three.js `BufferGeometry`. */
export interface MeshPartBuffers extends MeshPartInfo {
  /** x, y, z per vertex, in mm, z up. */
  readonly positions: Float32Array;
  /** Vertex-index triples, anticlockwise seen from outside. */
  readonly indices: Uint32Array;
}

export interface MeshBuffers {
  readonly parts: readonly MeshPartBuffers[];
}
