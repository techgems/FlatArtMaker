# FlatArtMaker: Specification (draft 3)

Turn a flat-colour, logo-style image into a colour-separated, flat-backed 3MF ready for multi-material printing as wall art or a coaster. A STEP file of the same model is also available for editing in CAD software. Everything runs in the browser and the app is hosted as a static site.

## Scope and constraints
- **Input:** one raster image (PNG, JPG or WebP). A transparent background marks "outside the piece".
- **Supported art:** flat, vector-like art. The UI says this before upload; photos and gradients are out of scope.
- **Colour count:** the user chooses how many output colours to print, from 1 to a hard limit of **16**. Printing constraints mean most pieces use far fewer. The image can contain more colours: similar tones (two purples) are merged to reach the user's count, automatically and then by hand. The in-between tones that edge smoothing makes are folded into a neighbour and never use up the count.
- **Outputs:**
  - a `.3mf` (primary) that opens as separate bodies, one per output colour, each on its colour's extruder (output colour slot *n* on extruder *n*). Slots count only the output colours that reach the model: an empty colour takes no slot, so the extruders run 1, 2, 3, … without gaps. It comes in two formats, picked at download: one for **PrusaSlicer**, and one for **Bambu Studio and Orca-based slicers**. Exact behaviour in every other slicer is out of scope.
  - a `.step` of the same model with real curves, so it can be edited in CAD (secondary). It must open cleanly in **Fusion**; anything that passes there is expected to work in Onshape.
  - the generated colour `.svg` (bonus)
- **No server:** nothing is uploaded and all processing happens on the client.

## Stack
| Layer | Choice |
|---|---|
| Engine | A Cargo workspace of small Rust crates (see **Architecture**). It is pure Rust and also builds natively for tests and a CLI. |
| Bindings | `flatart-wasm` (main engine) and `flatart-wasm-step` (STEP only, **loaded only when a STEP download is requested**), built with `wasm-bindgen` and `wasm-pack --target web`. |
| UI | Svelte 5 + TypeScript + Vite, with **no SvelteKit**. The core runs in a Web Worker, wrapped with Comlink. |
| 3D preview | three.js (`OrbitControls`), rendering mesh buffers handed over from WASM. |
| Hosting | Static `dist/` folder on Cloudflare Pages, Netlify or Amplify. |

Third-party crates, one per job. Each one is used **only inside its own adapter crate**:

| Job (interface) | First implementation | Dependency | Swap-in candidates |
|---|---|---|---|
| Colour detection (`ColorQuantizer`) | `flatart-quant-kmeans` | in-house weighted k-means; `palette` (sRGB → OKLab). `kmeans_colors` was dropped: it needs an OS random source that doesn't build for WASM, and it can't hold pinned centres fixed. | median cut, Wu (in-house) |
| Tracing colour borders (`BorderTracer`) | `flatart-trace-graph` | in-house (Pipeline step 7) | a `vtracer`-based tracer that also removes overlaps |
| Curve fitting (`CurveFitter`) | `flatart-fit-kurbo` | `kurbo` | `visioncortex` spline fitting |
| Triangulating faces (`Triangulator`) | `flatart-tri-itriangle` | `i_triangle` | `lyon_tessellation`, `earcutr` |
| SVG output (`Exporter<VectorModel>`) | `flatart-svg` | none (string writer) | `svg` crate |
| 3MF output (`Exporter<MeshModel>`) | `flatart-3mf` | `zip`, `quick-xml` | `threemf` crate |
| STEP output (`Exporter<VectorModel>`) | `flatart-step-truck` | `truck` (`truck-modeling` + `truck-stepio` + `truck-assembly`), pinned to a git commit (see below) | 1. `flatart-step-monstertruck` (`monstertruck` 0.4.x from crates.io); 2. `flatart-step-native`, a hand-written AP214 writer |
| Image decoding (CLI only) | `flatart-cli` | `image` | – |

**STEP library policy:**
- **`truck` is the primary choice.** The last crates.io release (Sep 2024) has no `truck-assembly`, so `flatart-step-truck` depends on `ricosjp/truck` through **git, pinned to an exact commit**. That pin is moved deliberately, with the commit and the reason recorded in the changelog.
- **`monstertruck` is the first fallback.** It's a fork of the same code, so most of the adapter carries over. Switch when a bug or missing feature blocks us and `truck` can't fix it, or won't in time.
- **The hand-written writer (`flatart-step-native`) is the last resort.**
  - **What it is:** a STEP *generator* only, with no parser. It's realistic because of the permanent geometry limits (solid straight extrusions only).
  - **What it writes:** about 17 entity types:
    - geometry: points, directions, axis placements, `LINE`, `B_SPLINE_CURVE_WITH_KNOTS`, `PLANE`, `SURFACE_OF_LINEAR_EXTRUSION`
    - topology: vertices, edges, loops, faces, `CLOSED_SHELL`, `MANIFOLD_SOLID_BREP`
    - a fixed template for units, context and product entries
    - assembly entities for the named colour components
  - **What it must get right:**
    - each edge is shared by exactly two faces, in opposite directions
    - loops run in the right direction, so faces point outward
    - vertices are shared, never duplicated
  - **How it's tested:** the same conformance suite as the other adapters, plus an automated re-read of the output that checks the shells are closed, counts the solids and reads their names.
- **Each switch needs:** its own adapter crate and the same conformance tests, so changing libraries is a one-line change in the wiring.

Image decoding uses the browser (`createImageBitmap` → RGBA) to keep the WASM file small.

All of these licences are MIT or Apache. GPL and LGPL libraries are banned, which rules out `libimagequant`, `potrace` and `opencascade.js`.

## Architecture
The engine uses a **ports-and-adapters** design.
- **Core:** the *core* defines the data types and, for every job that relies on an outside library or could be done more than one way, an *interface* (a Rust trait).
- **Adapters:** each implementation of an interface lives in its **own crate**. Only that crate depends on its third-party library.
- **Swapping:** replacing a library means writing a new adapter crate and changing one line in the *wiring* (the composition root).

### Workspace layout
```
crates/
  flatart-domain/          data types only: Rgba, DetectedPalette, OutputPalette, ColourMapping, SimilarityTree, EditLog, LabelMap, BorderMap (Junction, Border, Face),
                           VectorModel, MeshModel, Settings, units (Mm, Px), errors. No third-party deps except serde.
  flatart-ports/           the interfaces (traits) and their documented contracts, plus `testkit`,
                           the shared conformance tests every adapter must pass
  flatart-pipeline/        runs the stages and caches each result; depends only on domain + ports
  flatart-raster/          in-house pixel stages (load, clean-up, paint, printability): implements RasterStages
  flatart-quant-kmeans/    adapter: ColorQuantizer
  flatart-mapping/         in-house colour mapping (merge by similarity, edit-log replay): implements ColourMapper
  flatart-trace-graph/     adapter: BorderTracer
  flatart-fit-kurbo/       adapter: CurveFitter
  flatart-tri-itriangle/   adapter: Triangulator
  flatart-mesh/            implements MeshBuilder: builds MeshModel from VectorModel through the Triangulator interface
  flatart-svg/             adapter: Exporter<VectorModel>
  flatart-3mf/             adapter: Exporter<MeshModel>, plus SlicerDialect adapters (prusa, bambu)
  flatart-step-truck/      adapter: Exporter<VectorModel> (primary)
  flatart-step-monstertruck/  adapter: Exporter<VectorModel> (fallback, created only when needed)
  flatart-cli/             wiring for native use
  flatart-wasm/            wiring for the browser (main engine)
  flatart-wasm-step/       wiring for the browser (STEP only)
web/                       Svelte app
fixtures/                  test images and golden outputs
```

### Interfaces (traits in `flatart-ports`)
| Interface | Signature (simplified) |
|---|---|
| `ColorQuantizer` | `(&RgbaImage, &QuantSettings, pins: &[Oklab]) -> Detection` (a `DetectedPalette` plus its `LabelMap`) |
| `BorderTracer` | `(&LabelMap, &TraceSettings, &dyn CurveFitter) -> BorderMap` |
| `CurveFitter` | `(&[Point], &FitSettings) -> Vec<CubicBezier>` (end points fixed) |
| `Triangulator` | `(&PolygonWithHoles) -> Vec<[u32; 3]>` |
| `Exporter<M>` | `id()`, `file_extension()`, `mime_type()`, `export(&M, &ExportOptions) -> Result<Vec<u8>>` |
| `SlicerDialect` | `id()`, `parts(&MeshModel) -> Result<Vec<PackageFile>>` (`MeshModel` carries the `OutputPalette`): every slicer-specific file inside a 3MF, i.e. the model part `3D/3dmodel.model` (part layout, part names) and the slicer config under `Metadata/` (extruder per part). The 3MF exporter owns the rest of the package: `[Content_Types].xml`, `_rels`, the thumbnail and the zip. |
| `RasterStages` | `load(&RgbaImage, &LoadSettings) -> LoadedImage`, `clean_up(&LabelMap, &CleanUpSettings) -> LabelMap`, `paint(&LabelMap, &[Paint]) -> LabelMap`, `printability(&LabelMap, &PrintabilitySettings) -> Printable` (Pipeline steps 1, 4, 5, 6) |
| `ColourMapper` | `map(&Detection, &EditLog, &MappingSettings) -> Result<MappedColours>`: the output palette, the mapping, the output label map, the paints, the `SimilarityTree` and the dropped edits (Pipeline steps 3 and 5) |
| `MeshBuilder` *(provisional)* | `(&VectorModel, &MeshSettings, &dyn Triangulator) -> MeshModel` (Pipeline step 8) |

### Interface rules
- **No third-party types cross an interface.** Signatures use only `flatart-domain` types; adapters convert at their edge.
- **In-house stages sit behind interfaces too,** so the pipeline depends only on domain + ports. The one marked *provisional* has a stub contract and no conformance suite yet; it is settled, with its suite, by the task that implements it (10 mesh builder).
- **Every interface has a written contract and a conformance suite in `testkit`.**
  - Examples: "faces are closed loops", "triangles cover the polygon exactly once", "export is deterministic for the same input".
  - Every adapter runs that suite in its own tests. A replacement adapter is accepted when it passes the suite.
- **Only the wiring crates (`flatart-cli`, `flatart-wasm*`) name concrete adapters.** Everything else receives `Box<dyn Trait>`, through `Pipeline::builder()`. Speed doesn't suffer, because each call does a whole stage of work.
- **Optional adapters sit behind Cargo features.** This lets the WASM build include only what it ships.
- **Choosing the slicer:** the 3MF exporter is built with a `Box<dyn SlicerDialect>`. The wiring crates pick `prusa` or `bambu` from the CLI's `--slicer` flag or the UI's slicer choice.
- **`SlicerDialect` contract:**
  - one part per output colour (plus the base when B > 0), named after the colour
  - output colour slot *n* on extruder *n*, counting only colours that have a part, so there are no gaps
  - a `3D/3dmodel.model` entry is always present, and no paths repeat
  - deterministic output

### Front end (`web/`)
- **Engine access:** the UI talks to the engine only through a TypeScript `EngineClient` interface. The implementation calls the worker through Comlink, and tests can use a fake.
- **3D preview:** sits behind a `MeshViewer` interface, implemented with three.js.
- **Saved projects:** handled by a `ProjectStore` interface (see **Saved projects**). The first implementation combines `localStorage` and IndexedDB, and tests can use an in-memory fake.
- **Shared types:** the types passed between Rust and TypeScript (`Settings`, `Palette`, results) are **generated from the Rust types** with `tsify`, so they are defined in one place.
- **Svelte components:**
  - Components only display things; state lives in `.svelte.ts` stores (runes).
  - Components never import WASM or three.js directly.

## Coding standards
- **Rust:**
  - Formatting and linting: `rustfmt`, plus `clippy` with warnings treated as errors (`-D warnings`).
  - `#![forbid(unsafe_code)]` in every crate except the WASM wiring.
  - Errors use `thiserror` per crate and are mapped to `flatart_domain::Error`. Nothing panics across the WASM boundary.
- **Stages are pure functions:** each takes an input plus settings and returns an output.
  - No I/O, no clocks, and seeded randomness, so results are deterministic.
  - Exporters return bytes, so the caller decides where to save them.
- **Units are explicit types:**
  - `Mm` and `Px` are newtypes, never bare floats.
  - Geometry is stored in mm as `f64`. Adapters that need whole-number coordinates (e.g. a triangulator working in µm) convert inside.
- **Tests at three levels:**
  - unit tests in each crate
  - conformance tests (`testkit`) for each adapter
  - golden tests on `fixtures/` through the CLI
  - property tests with `proptest` for the shared-border map. Every border has two different sides, faces are closed, and meshes are watertight.
- **TypeScript:**
  - `strict` mode, ESLint + Prettier, Vitest for logic.
  - One Playwright smoke test: load an image, then download a 3MF.
- **Settings:** one versioned, serde-serialisable `Settings` struct. Old saved settings are migrated, never dropped.

## Core data model: the shared-border map
Every output is generated from **one** vector model of the art:
- **Junctions:** points where 3 or more areas meet, or where 2 areas meet the image edge.
- **Borders:** a curve (a sequence of cubic Béziers) between two junctions. It records the area on its left and the area on its right. One of those may be *outside*. A closed border with no junction gets an arbitrary start point.
- **Faces:** the closed loops of borders around each area: one outer loop plus any holes. Each face has an output-colour ID.

Because the two areas on either side of a border use the **same curve**, the 3MF has no gaps or slivers, the SVG is consistent, and the STEP solids meet exactly.

## Pipeline
Each stage caches its result, so a slider change only re-runs the stages after it.

1. **Load:** read RGBA pixels. Pixels with alpha below a threshold count as outside. If the image has no transparency, the user can pick a background colour to treat as outside (an exact RGB match).
   - **Art bounds:** the bounding box of the non-outside pixels sets the art's **aspect ratio**. Transparent padding around the art is ignored, so it doesn't distort the size.
2. **Detect colours:** run k-means in OKLab on sampled opaque pixels to find the **detected colours**.
   - There can be many more of these than will be printed: up to 32, 24 by default.
   - Every pixel is assigned to its nearest detected colour, producing a *detected label map*.
   - **Eyedropper pins:** a colour picked with the eyedropper in the editor joins detection as a fixed centre that k-means can't move. This catches small details the automatic pass would blur into a neighbour, like a tongue inside a cheek.
3. **Map colours (user-editable):** every detected colour is mapped to exactly one **output colour**, a numbered filament slot. Mapping is many-to-one, and there are **16 output colours at most**.
   - **Edge tones fold first:** a detected colour with no interior pixel (none whose 8 neighbours all share it) is an *edge tone*, an in-between colour made by edge smoothing. Each edge tone joins the group it shares the longest border with, among groups that have interior pixels. One that borders none joins the closest in OKLab. Edge tones never count as colours, so fewer than the user's count may remain. If no colour has an interior pixel, nothing folds.
   - **Automatic first pass (by count only):** then the closest pair of groups in OKLab (each group at its pixel-weighted mean colour) is merged until no more than the user's **"Output colours"** count remain. There is no ΔE threshold: the user's count is the only stop. Each group becomes one output colour, slots ordered by pixel count. This is only a starting point.
   - **Similarity map:** the merges continue past the user's count down to a single group, and the full merge order (which two groups joined, and how far apart they were in ΔE) is returned as a `SimilarityTree`. The merge order starts with the edge-tone folds. From it the editor can show which tones are similar at any count, and which two groups merge next if the count drops by one. Edits never change it: it describes the detected colours only.
   - **User control:** in the colour-mapping editor (see **UI**), the user can point any detected colour at any output colour. Colours the user adds count toward the 16 limit, not toward their "Output colours" count.
   - **Result:** an *output label map*.
4. **Clean up:** on the output label map, with 4-connected areas (as in tracing):
   - Run a mode filter to remove the in-between colours that edge smoothing creates. Each pixel takes the most common colour in its 3×3 block, counting only non-outside pixels, and changes only when another colour is strictly more common than its own. Outside never changes. This rounds colour-to-colour corners by about a pixel.
   - Merge connected blobs smaller than the speckle area into the neighbouring area they share the longest border with, smallest first. A merged blob counts toward the area it joined. Blobs that border only the outside are kept, so the outline never changes. A speckle area of 0 turns merging off.
5. **Region painting (user-editable):** the user can repaint individual areas.
   - **How it works:** pick an output colour with the brush tool, then click areas in the editor. The connected area under each clicked point, in the cleaned output label map, is filled with that colour.
   - **Why this level:** only the clicked areas change, not everything mapped to their colour. It's done on pixels, so the shared-border map stays valid by construction.

   **All colour-mapping operations:**

   | Operation | Effect |
   |---|---|
   | **Map** | Sets which output colour a detected colour maps to. |
   | **Paint region** | Gives the area at a clicked point a different output colour. |
   | **Add output colour** | Adds a colour with a picker, hex input or eyedropper. Can only be added while there are fewer than 16. |
   | **Change output colour** | Changes the colour shown and exported (the 3MF colour list, SVG fills and previews). The mapping and areas stay the same. |
   | **Delete output colour** | The detected colours and painted areas that used it move to an output colour the user picks. |
   | **Rename** | Output colours get an automatic name from the nearest common colour name (e.g. `Black`, `OrangeYellow`). Names must be unique and use only letters and digits. They name the 3MF parts, the STEP bodies and the SVG layer IDs. |
   | **Set base colour** | Disabled when the base height is 0. Until the user picks one, the base is the output colour most common along the art's outer edge. |
   | **Reorder** | The order is the filament slot order (1, 2, 3, …). |
   | **Pin detected colour** | The eyedropper addition from step 2. |
   | **Reset mapping** | Brings back the automatic mapping. Painted areas are kept unless the user also clears them. |

   - **Empty output colours:** an output colour with nothing mapped or painted to it is kept in the editor, with a warning, but left out of the exports, and takes no filament slot. The same goes for a colour that clean-up or the thin-feature fix removes completely: what counts is whether the colour has a part in the model.
   - **What edits re-run:**
     - Map, pin and delete re-run from step 3.
     - Paint re-runs from step 5.
     - Changing a colour or renaming only re-runs the exports.

   **How edits are stored:** as an ordered **edit log**. It's saved with the project, replayed on load, and gives **undo/redo** for free (step back and forward through the log).
   - **Stable IDs:**
     - Detected colours are identified by their OKLab colour.
     - Output colours get a generated ID, so changing their colour never breaks a reference. Automatic colours use IDs 0–15 and colours the user adds use 16 and up, so changing the count never makes them collide.
     - Painted areas are stored as the **clicked point in source-image pixels** plus the output colour ID, never as an area number. Area numbers change whenever settings change; a point always lands in a well-defined area.
   - **If an Advanced setting changes detection:** each stored detected colour is matched to the nearest new one within 10 ΔE. Operations that no longer match are dropped, and the user is told which ones.
6. **Printability (still on pixels):** the X/Y size gives the pixels-per-mm.
   - Optionally upscale the label map (2–4×, nearest-neighbour).
   - **Thin-feature fix** (only when **"Fix thin features"** is on):
     - **How it works:** for each colour, shrink and regrow by a disc as wide as the threshold (a morphological opening). This removes any part narrower than the threshold **in any X/Y direction**, including islands that small inside another colour. The removed pixels go to the other colour that most surrounds them, filling in from the edges.
     - **The outline never changes:** a too-thin part with no other colour beside it (a whisker or dot sticking into the outside) keeps its colour and is reported as a thin feature, which feeds the warning in step 7. The few corner pixels of a wide part that the disc can't reach are rounding, not features, and aren't reported.
     - **When off:** the label map passes through unchanged. Modern printers can handle details thinner than a 0.4 mm nozzle, so this is the user's call.

   Doing this on pixels before tracing keeps the shared-border map valid by construction.
7. **Trace borders:** build the shared-border map from the label map.
   - Walk the pixel edges wherever two labels differ, splitting at junctions.
   - Simplify each border with its end points fixed, rejecting any change that would make it cross another border.
   - Fit cubic Béziers to each border within the smoothing tolerance. A turn sharper than the corner threshold stays a sharp corner.
   - Assemble each area's faces from its borders.
   - When the thin-feature fix is on, warn about any remaining feature thinner than the threshold.
8. **Build the mesh (3MF and 3D preview):** turn **each border** into straight segments **once**, at the curve tolerance, scaled to mm. Adjacent faces therefore share identical edge points.
   - **Base (only when the base height B > 0):** the outline is the borders that touch *outside*. Holes are filled as an option. The base runs from z=0 to z=B.
   - **Colour layer:** each colour's faces are filled with triangles (with holes) and raised from z=B to z=B+C, giving one mesh per colour.
   - **Base-colour faces** (only when B > 0) merge into the base at full height, which saves filament swaps.
   - **No base (B = 0):** the colour layer starts at z=0 and no base body is made in any output. There is no base colour, so every colour becomes its own colour-layer bodies. Neighbouring colours still share exact borders, so the model prints as one flat piece.
   - Each mesh must be closed and have no self-intersections.
9. **Export:**
   - **3MF:** one object with one *part* per colour, a standard `m:colorgroup`, and a thumbnail PNG.
     - The part layout and the slicer-specific files come from the chosen `SlicerDialect`. Neither family reads the other's format correctly (milestone 0):
       - **`prusa`:** one mesh holding every part, plus `Metadata/Slic3r_PE_model.config` with each part's triangle range, name and extruder.
       - **`bambu`** (Bambu Studio and Orca-based slicers): one mesh object per part under a parent object, plus `Metadata/model_settings.config` with each part's name and extruder.
     - Both dialects put output colour slot *n* on extruder *n*. Empty colours take no slot, so the extruders have no gaps.
   - **STEP (AP214):** built from the shared-border map, with no straight-segment approximation.
     - **Bodies:** one solid per island, plus the base as a solid of its own when B > 0 (one per separate piece of art). Units are mm.
     - **Names matter; colour doesn't.** Bodies are grouped and named by output colour, so anyone editing the file can recolour a whole colour at once in their own CAD tool:
       - **Structure (grouped bodies):** one root part, holding one named component per output colour (`Black`, `White`, …), plus a `Base` component when B > 0.
         - Each colour component holds all of that colour's bodies as solids in a single shape. It has no sub-component per body, because that adds too much clutter when editing.
         - In STEP terms: a root `PRODUCT`, one child `PRODUCT` per colour, and one `SHAPE_REPRESENTATION` per colour holding its `MANIFOLD_SOLID_BREP`s.
       - **Body names:** the first body of a colour is `Black`, then `Black-1`, `Black-2`, … These names are written on the solids, but CAD tools may show generic names instead (Fusion ignored solid names in testing). The colour component's name is the one that matters.
       - **Numbering order:** by area, largest first, with ties broken top-to-bottom then left-to-right. The order is deterministic, so re-exporting gives the same names.
       - **The base** (when B > 0) is the `Base` component. It holds the base slab (from 0 to B) and, as bodies of their own, the base colour's islands (from B to B + C). They are numbered together: `Base`, `Base-1`, … Recolouring the `Base` component recolours both.
     - **Colour styling** (`STYLED_ITEM` / `COLOUR_RGB`) is not required. It can be added later as a nice-to-have.
     - **Faces:** flat top and bottom faces bounded by the original curves, converted to B-splines. The side walls are straight extrusions of those curves.
     - **Geometry limits (permanent):**
       - Every body is a **solid** (`MANIFOLD_SOLID_BREP`). The file never contains loose surfaces or mesh bodies.
       - Every body is a **straight extrusion** of a flat outline: flat top and bottom, and vertical side walls.
       - No chamfers, fillets, drafts or sloped faces, in STEP or in the 3MF.
     - **Base-colour faces** (when B > 0): in the `Base` component, but **not fused** into the slab, unlike the 3MF, where they're part of the base mesh.
       - **Why:** fused, the slab's top face would touch itself wherever two base-colour islands meet only at a corner. A face boundary can't do that, and `truck` rejects it (it failed on 21 of 52 test images). Separate bodies also keep every body a straight extrusion.
       - Accepted by the user, 2026-11-12.
   - **SVG:** one `<path>` per face, built from the same Béziers.

## UI (single page)
- **Left panel:** the two **primary sliders** sit at the top, each paired with a number box for exact entry. The **X/Y size** control sits just below them, then the **Printability** group. Everything else is in a collapsed **Advanced** section.
- **Centre:** tabs for **Original**, **Mapped colours**, **Vector (SVG)** and **3D**.
- **Right:** a summary of the output colours: the numbered filament chips, with names and % coverage. An **"Edit colours"** button opens the colour-mapping editor.
- **Header:** "Download 3MF" (primary), "Download STEP" and "Download SVG". The STEP module loads on the first click and shows a spinner meanwhile.
  - Next to "Download 3MF" is a slicer choice: **PrusaSlicer**, or **Bambu Studio / Orca**. The last choice is remembered.
- **Persistence:** every piece of work is saved automatically as a project (see **Saved projects**). A "Recent projects" list on the start screen reopens them.

### Colour-mapping editor (modal, "Edit colours")
Its layout follows MakerWorld's lightbox-maker "Edit Color Details" dialog: a large canvas on the left and a colour panel on the right.

- **Header:** the title "Edit colours" and a close (×) button. Closing works like Cancel.
- **Canvas (left, most of the space):**
  - shows the **live result** (the cleaned output label map, or the traced art) on a checkerboard background, so transparent "outside" areas are visible
  - a toggle shows the original image underneath at partial opacity
  - **Overlay controls, bottom-left:** an undo/redo group and a zoom in/out group. The mouse wheel also zooms, and dragging pans.
  - Hovering a row in the Mapping list highlights that detected colour's pixels on the canvas.
- **Colour panel (right):**
  - **"Colours in use":** a row of **numbered chips**, one per output colour. The number is the filament slot, and the chip's fill and number colour match the colour. An empty colour's chip shows its warning instead of a number, and the chips after it number on without a gap.
    - **Header actions:** **+** adds an output colour (up to 16). The **brush** icon toggles paint mode.
    - **Clicking a chip** opens its details: a colour picker with hex input and an eyedropper, the name, a "base" option, and delete. Delete asks where its colours should go.
    - **Dragging chips** reorders them, and so the filament slots.
    - An "n / 16" counter sits next to the chips.
  - **"Colour count":** the user's "Output colours" count (1–16), editable here. Changing it re-runs the automatic mapping.
    - Detected colours are grouped by the similarity map, so similar tones sit together.
    - The editor previews which two groups would merge next if the count dropped by one. When the count is above the number of colours that aren't edge tones, dropping it changes nothing, so there is no preview.
  - **"Mapping":** a two-column grid with one cell per detected colour, sorted by coverage. Each cell shows the detected colour as a **circle → the output chip** it maps to.
    - Clicking the target chip opens a picker of the output colours, plus "+ New colour" (disabled at 16).
    - **"Reset mapping"** sits in the section header.
    - Below the grid, an **eyedropper "Pin a missed colour"** action handles details the detection merged away (Pipeline step 2).
  - **Paint mode (brush on):**
    - The active output colour is highlighted.
    - Hovering the canvas outlines the area under the pointer; clicking paints it, and shift-drag paints every area the pointer passes over.
    - Esc or the brush icon leaves paint mode.
- **Footer:** **Cancel** and **Confirm**.
  - Edits preview live in the modal.
  - Confirm saves them to the project's edit log; Cancel throws them away.
- **Keyboard:** Ctrl+Z / Ctrl+Shift+Z for undo/redo, B for the brush, Esc to leave paint mode or close.
- **Warnings shown inline:**
  - an output colour has nothing mapped or painted to it
  - a mapping or paint stroke would create a feature thinner than the threshold (only when the thin-feature fix is on)
  - the 16-colour limit has been reached

**Primary sliders:** both are **Z-axis** heights. The piece is printed lying flat on the bed, so in STEP each height is the **length of the extrusions**. These are the only two Z values the user controls: every colour shares the same layer height.

| Slider | Default | Range | Rule |
|---|---|---|---|
| **Colour layer height C** | 1.0 mm | 0.1–10 mm, step 0.05 | **Never 0.** The domain type `LayerHeight` can't hold 0 or a negative value, so an invalid value can't reach the engine. |
| **Base height B** | 3.4 mm | **0**–10 mm, step 0.05 | **0 means no base:** no base body, no base colour, and "Fill holes in base" is disabled. Stored as `Option<LayerHeight>`, where 0 becomes `None`. |

**X/Y size (aspect ratio always locked):**
- **The control:** one slider that scales the piece up or down in X and Y **together**, plus two linked number boxes, **X (mm)** and **Y (mm)**.
  - Dragging the slider updates both boxes.
  - Typing in either box sets the other from the aspect ratio.
- **The aspect ratio can't be changed.** It comes from the art bounds (Pipeline step 1).
- **What's stored:** a single value, **mm per source pixel**. X and Y are always derived from it, so rounding can never change the aspect ratio. In the code this is the domain type `PrintScale`, and there is deliberately no way to set X and Y independently.
- **Accuracy:** the final model's bounding box, in the 3MF and the STEP, is exactly the X × Y shown. Values are displayed to 0.01 mm.
- **Range:** 20–300 mm on the longest side, with 100 mm as the default.

**Printability:** set before generating the output files. These settings change the geometry, so the previews always show their effect.

| Control | Default | Range | Rule |
|---|---|---|---|
| **Fix thin features** | on | on / off | When off, the threshold is disabled and thin details are kept as traced. |
| **Thin-feature threshold** | 0.25 mm | **0.25**–1.2 mm, step 0.05 | The smallest width allowed in any X/Y direction. The domain type `MinFeatureWidth` can't hold less than 0.25 mm. Stored as `Option<MinFeatureWidth>`, where off becomes `None`. |

**Advanced:**

| Control | Default | Range |
|---|---|---|
| Detected colours (max) | 24 | 2–32 |
| Output colours (the user's count for the automatic mapping; also set in the colour-mapping editor) | 8 | 1–16 |
| Speckle area | 16 px | 0–500 px |
| Trace upscale | 2× | 1–4× |
| Smoothing tolerance | 0.5 px | 0.1–2 px |
| Corner threshold | 60° | 0–180° |
| Curve tolerance (mesh only) | 0.02 mm | 0.005–0.2 mm |
| Fill holes in base | on | on / off (disabled when B = 0) |

## Saved projects
Each uploaded image becomes a **project**, saved automatically in the browser so the user can close the tab and come back to it.

**What a project holds:**

| Field | Contents |
|---|---|
| `id`, `name`, `createdAt`, `updatedAt` | `name` defaults to the image file name and can be edited |
| **Original image** | the uploaded file's bytes, unchanged, plus its MIME type |
| **Colour-mapping edit log** | the ordered list of mapping, painting and output-colour operations, including the undo position (Pipeline steps 2–5) |
| **Settings** | every slider and Advanced value: colour layer height C, base height B, `PrintScale` (X/Y size) and the rest |
| `schemaVersion` | used to migrate older saved projects instead of discarding them |

Derived data is **not** stored: the label map, border map, meshes and exports. It's all recomputed from the image, edits and settings, and the results are deterministic.

**Where it's stored:**
- **Small data** (project list, names, settings, palette edits) goes in **`localStorage`**, as JSON under a `flatart:` key prefix.
- **The image** goes in **IndexedDB**, as a `Blob`. `localStorage` stores only text and is capped at roughly 5 MB per site. As base64, a single 2–3 MB PNG would use most of that, so images don't fit there reliably.
- **Behind an interface:** both stores sit behind the `ProjectStore` interface, so the storage choice can change without touching the UI.

**Behaviour:**
- **Autosave:** about 500 ms after the last change.
- **Project management:** a "Recent projects" list with rename and delete. Delete asks for confirmation.
- **Storage full:** the user is told and offered "delete old projects". Work in the current tab continues.
- **Storage unavailable** (private windows, blocked storage): the app still works, with a notice that projects won't be kept.
- **Nothing is uploaded:** saved projects never leave the browser.

**Possible v2 addition:** export and import a project as a single `.flatart` file (a zip of the image and JSON), to move projects between devices.

## Milestones
0. **Two tests, done first:**
   - **Slicers:** hand-write a two-colour square 3MF. Iterate until it opens as separate parts on filaments 1 and 2, with no splitting. The inputs needed are two-colour projects saved from PrusaSlicer and from Bambu Studio, to copy their metadata formats exactly.
     - **Outcome (2026-11-09):** each format works only in its own family, so there are two dialects.
       - The PrusaSlicer format opens in Orca as one body.
       - The Bambu format opens in PrusaSlicer as separate objects, all on extruder 1.
       - Details: `spikes/3mf/FINDINGS.md`.
   - **STEP:** build two islands of one colour, plus a base, from a square with a Bézier-curved hole. Write them as STEP with `truck` (git-pinned) and compile the same code to WASM. Confirm in **Fusion** that the file opens cleanly and shows the colour components with their bodies. Onshape is checked once as a sanity test.
     - If `truck` fails a check, retry it with `monstertruck`. If both fail, use the hand-written writer.
     - **Outcome (2026-11-09):** `truck` master passed, natively and in WASM, and the file opens cleanly in Fusion.
       - The grouped-bodies structure (one named component per colour) was chosen.
       - Fusion ignores names on solids, and one component per body was too cluttered.
       - Details: `spikes/step/FINDINGS.md`.
1. **Native engine and CLI:**
   - Set up the workspace: `flatart-domain`, `flatart-ports` with `testkit` conformance suites, and `flatart-pipeline`.
   - Write the first adapters, and `flatart in.png out.3mf --slicer prusa|bambu --output-colors 8 ...` running stages 1–9 (3MF + SVG).
   - Conformance tests run in CI from day one. Golden test: Jolteon PNG → no gaps between colours, each dialect opens in its slicer as separate parts on the right extruders, about 100 mm, no repair warnings, and much smaller than the 10.8 MB reference. A second golden test with B = 0 checks that there is no base part, and that the colours start at z=0 and still meet with no gaps.
2. WASM build and worker, plus a minimal Svelte UI with the 2D tabs and download.
3. three.js preview, the colour-mapping editor (mapping, output colours, paint mode, eyedropper pins, undo/redo), the X/Y size control and saved projects.
4. **STEP export:** `flatart-step-truck` (or the fallback adapter, if milestone 0 forced a switch), `flatart-wasm-step` loaded on demand, plus a `--step` flag on the CLI. Golden test: Jolteon STEP opens with one named component per output colour (plus `Base`), one solid per island inside it, and no repair needed.
5. Polish: the SVG download, and deploying the static site.

## Later (v2 candidates)
- colour styling in STEP (`STYLED_ITEM`), on top of the names
- a keyhole or hanger slot for wall art
- a text or logo on the base underside
- importing an existing colour SVG (skip steps 1–7, but it still needs to be converted to shared borders)
- multiple pieces per plate

## Open questions
- None at present.
