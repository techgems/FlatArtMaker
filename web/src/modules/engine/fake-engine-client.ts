import type { EngineClient } from './engine-client';
import type {
  ArtBounds,
  DroppedEdit,
  EditLog,
  EditOp,
  LoadedImageInfo,
  MeshBuffers,
  MeshPartBuffers,
  OutputColour,
  OutputPalette,
  Previews,
  RgbaPixels,
  Settings,
} from './types';

const OUTSIDE = 0xffff;
const ALPHA_THRESHOLD = 128;

interface Project {
  readonly width: number;
  readonly height: number;
  readonly labels: Uint16Array;
  readonly palette: OutputPalette;
  readonly bounds: ArtBounds;
  settings: Settings;
  edits: EditLog;
}

function defaultSettings(bounds: ArtBounds): Settings {
  return {
    version: 1,
    colourHeight: 1,
    baseHeight: 3.4,
    scale: 100 / Math.max(bounds.width, bounds.height),
    minFeature: 0.25,
    load: { alphaThreshold: ALPHA_THRESHOLD, background: undefined },
    advanced: {
      detectedColours: 24,
      outputColours: 8,
      speckleArea: 16,
      traceUpscale: 2,
      smoothing: 0.5,
      corner: 60,
      curveTolerance: 0.02,
      fillBaseHoles: true,
    },
  };
}

/** Each distinct opaque colour becomes an output colour; the rest is outside. */
function analyse(pixels: RgbaPixels): Omit<Project, 'settings' | 'edits'> {
  const { width, height, data } = pixels;
  if (data.length !== width * height * 4) throw new Error('The pixels do not match the size');
  const colours: OutputColour[] = [];
  const labels = new Uint16Array(width * height).fill(OUTSIDE);
  let [minX, minY, maxX, maxY] = [width, height, -1, -1];
  for (let i = 0; i < width * height; i++) {
    const [r = 0, g = 0, b = 0, a = 0] = data.subarray(i * 4, i * 4 + 4);
    if (a < ALPHA_THRESHOLD) continue;
    let colour = colours.find((c) => c.display.r === r && c.display.g === g && c.display.b === b);
    if (colour === undefined) {
      colour = { id: colours.length, name: `Colour${colours.length + 1}`, display: { r, g, b } };
      colours.push(colour);
    }
    labels[i] = colour.id;
    const [x, y] = [i % width, Math.floor(i / width)];
    [minX, minY, maxX, maxY] = [
      Math.min(minX, x),
      Math.min(minY, y),
      Math.max(maxX, x),
      Math.max(maxY, y),
    ];
  }
  if (maxX < 0) throw new Error('The image has nothing opaque');
  const bounds = { x: minX, y: minY, width: maxX - minX + 1, height: maxY - minY + 1 };
  return { width, height, labels, bounds, palette: { colours, base: colours[0]?.id } };
}

/** The output colour an operation names, for those that must name an existing one. */
function namedColour(op: EditOp): number | undefined {
  switch (op.op) {
    case 'map':
      return op.output;
    case 'changeOutputColour':
    case 'deleteOutput':
    case 'rename':
    case 'setBase':
      return op.id;
    case 'paintRegion':
      return op.colour;
    case 'addOutput':
    case 'reorder':
    case 'pinDetected':
    case 'resetMapping':
      return undefined;
    default: {
      const unknown: never = op;
      return unknown;
    }
  }
}

function droppedEdits(project: Project): DroppedEdit[] {
  const ids = new Set(project.palette.colours.map((c) => c.id));
  return project.edits.ops.slice(0, project.edits.position).flatMap((op, position) => {
    const named = namedColour(op);
    return named === undefined || ids.has(named)
      ? []
      : [{ position, reason: 'unknownOutputColour' as const }];
  });
}

/** One triangle per part: enough to exercise the buffers. */
function part(project: Project, colour: OutputColour, kind: 'base' | 'colour'): MeshPartBuffers {
  const slot = project.palette.colours.indexOf(colour) + 1;
  return {
    kind,
    colour: colour.id,
    name: kind === 'base' ? 'Base' : colour.name,
    display: colour.display,
    slot,
    positions: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
    indices: new Uint32Array([0, 1, 2]),
  };
}

/** Runs `fn` as a promise, so a throw becomes a rejection, as in the real client. */
function attempt<T>(fn: () => T): Promise<T> {
  return new Promise((resolve) => resolve(fn()));
}

function checkSettings(settings: Settings): void {
  const height = settings.colourHeight;
  if (!(height > 0 && height <= 10)) throw new Error(`Invalid layer height: ${height}`);
  if (!(settings.scale > 0)) throw new Error(`Invalid print scale: ${settings.scale}`);
}

/**
 * An in-memory EngineClient for tests. It follows the client contract with
 * canned geometry: every distinct opaque colour is an output colour, and
 * meshes and files are placeholders.
 */
export function createFakeEngineClient(): EngineClient {
  let project: Project | null = null;

  function open(): Project {
    if (project === null) throw new Error('No image is loaded');
    return project;
  }

  return {
    version: () => Promise.resolve('0.0.0-fake'),

    loadImage: (pixels): Promise<LoadedImageInfo> =>
      attempt(() => {
        const loaded = analyse(pixels);
        const settings = defaultSettings(loaded.bounds);
        project = { ...loaded, settings, edits: { ops: [], position: 0 } };
        return { bounds: loaded.bounds, settings };
      }),

    setSettings: (settings) =>
      attempt(() => {
        checkSettings(settings);
        open().settings = settings;
      }),

    applyEdits: (log) =>
      attempt(() => {
        open().edits = log;
      }),

    previews: (): Promise<Previews> =>
      attempt(() => {
        const p = open();
        const { width, height, palette, settings } = p;
        return {
          mapped: {
            width,
            height,
            outside: OUTSIDE,
            palette,
            detected: { colours: [] },
            mapping: { targets: [] },
            similarity: { colours: 0, edgeTones: 0, merges: [] },
            labels: p.labels.slice(),
          },
          vector: {
            borders: { width, height, junctions: [], borders: [], faces: [] },
            palette,
            scale: settings.scale,
            extrusion: {
              colour: settings.colourHeight,
              base: settings.baseHeight,
              fillBaseHoles: settings.advanced.fillBaseHoles,
            },
          },
          report: { thinFeatures: [], droppedEdits: droppedEdits(p) },
        };
      }),

    mesh: (): Promise<MeshBuffers> =>
      attempt(() => {
        const p = open();
        const base = p.palette.colours.find((c) => c.id === p.palette.base);
        if (p.settings.baseHeight === undefined || base === undefined) {
          return { parts: p.palette.colours.map((c) => part(p, c, 'colour')) };
        }
        // The base colour prints as part of the base.
        const others = p.palette.colours.filter((c) => c !== base);
        return { parts: [part(p, base, 'base'), ...others.map((c) => part(p, c, 'colour'))] };
      }),

    export3mf: (slicer, name) =>
      attempt(() => {
        open();
        return new TextEncoder().encode(`PK\u0003\u0004${slicer}:${name}`);
      }),

    exportSvg: (name) =>
      attempt(() => {
        open();
        return new TextEncoder().encode(`<svg><title>${name}</title></svg>`);
      }),

    dispose: () => {
      project = null;
    },
  };
}
