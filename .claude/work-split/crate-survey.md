# Crate survey: could the hand-written adapters use existing crates?

Researched 2026-11-11. Read-only: no repo code changed, nothing committed. No dependency is added without the user's approval.

**Short answer:** no mature crate meets any of the six port contracts as it stands. The nearest hits are `quantette` (colour detection) and `vtracer` 1.0 alpha (border tracing). Both miss a contract point or are too young to take. Keep all six. Two small, optional follow-ups are listed at the end.

## How this was checked
- **Port contracts and testkit:** read `crates/flatart-ports/src/*.rs` and `testkit/`, plus each adapter's source and `Cargo.toml`.
- **Metadata:** crates.io API (versions, dates, licence, downloads, dependency trees, features).
- **Docs and source:** docs.rs, GitHub READMEs, and the downloaded crate sources (`vtracer`, `quantette`, `flo_curves`).
- **WASM builds and sizes:** probes in a scratch cargo workspace *outside the repo* (`…\scratchpad\probe`).
  - `cargo check --target wasm32-unknown-unknown`, then `cargo build --release` (`opt-level = "s"`, LTO, panic abort).
  - Sizes are raw `.wasm` bytes, no `wasm-opt`, for one small exported function that calls the crate. An empty probe is 324 B.
  - For context, `flatart-wasm` is 707,917 B raw / 299,555 B gzip ([NOTES.md](NOTES.md)).
- **Not checked:**
  - open-issue counts, except where stated
  - output quality on the Jolteon fixtures
  - threads: none of the probes uses threads

## Dependency policy recap
- **Licences:** MIT or Apache only. GPL and LGPL are banned ([SPEC.md](../../SPEC.md) line 56).
- **Banned:** `imagequant` is GPL-3.0-or-later, so it's out.
- **Already rejected in the SPEC adapter table:** `kmeans_colors`.
  - SPEC reason: it needs an OS random source, which doesn't build for WASM, and it can't hold pinned centres fixed.
  - **Re-checked:** `kmeans_colors` 0.7.1 with default features off still fails `cargo check --target wasm32-unknown-unknown`.
    - Cause: `rand` is pulled in with `thread_rng`, so `getrandom` 0.3.4 raises a `compile_error!` ("wasm32-unknown-unknown targets are not supported by default").
    - It could be forced through with the `wasm_js` cfg flag, but that adds a JS import to the module.
    - Its README lists no way to supply fixed centres. Both reasons stand.

---

## 1. Colour detection: `flatart-quant-kmeans` (`ColorQuantizer`)

The adapter is about 430 lines of Rust: histogram, weighted k-means++ with a fixed seed, and fixed centres. It already uses `palette` for sRGB → OKLab.

**Contract points that matter:** pins appear with exactly their OKLab value and stay fixed during clustering; deterministic; identical RGB means identical label; no more than `max_colours` colours.

| Candidate | Version / date | Licence | Maintenance | WASM (verified) |
|---|---|---|---|---|
| **`quantette`** | 0.6.0 · 2026-05-15 | MIT/Apache | Active: 6 releases since Nov 2025, about 2.2M recent downloads, 3 open issues, 39 stars. MSRV 1.90. | Builds for wasm32 with `default-features = false, features = ["kmeans"]` (no rayon, no `image`, no OS RNG; uses `rand` 0.10 with `alloc` plus `rand_xoshiro`). Probe: **78 KB** raw (includes `palette`, which we already ship). |
| `kmeans_colors` | 0.7.1 · 2025-09-08 | MIT/Apache | Active-ish: 5 open issues, 52k recent downloads. | **Fails** (see above). Rejected in SPEC. |
| `color_quant` (NeuQuant) | 2.0.0 · 2026-05-09 | MIT | Maintained by image-rs. | Pure Rust, wasm-safe. NeuQuant is a neural-net quantiser: no OKLab, no pins, no pixel weights. |
| `imagequant` | 4.4.1 | **GPL-3.0** | – | Banned. |
| `exoquant` | 0.2.0 · 2016 | MIT | Dead. | – |
| `linfa-clustering` | 0.8.1 · 2025-12-23 | MIT/Apache | Active, but a heavy ML stack (`ndarray`, `rand`). | Not probed. Needs an OS RNG by default. Overkill for 24 clusters. |
| `kmeans` | 2.0.2 · 2026-03-03 | Apache-2.0 | 1.6k recent downloads. | Not probed. |

**`quantette` against the contract (source read, not run):**
- **Pins: not supported.** `Kmeans::run_slice(colours, initial_centroids, KmeansOptions)` takes starting centroids, but every centroid moves. `KmeansOptions` only has `sampling_factor`, `max_samples`, `batch_size` and `seed`. This is the same pinned-centre gap SPEC cites against `kmeans_colors`.
  - An adapter workaround (cluster only the pixels not nearest a pin, then append the pins) changes the semantics: pinned colours stop pulling their neighbours' centres.
- **Algorithm:** online (MacQueen) k-means with a decaying learning rate, over sampled pixels (default cap 262,144). Our adapter does exact Lloyd iterations on the histogram of unique colours, weighted by pixel count, which is cheap and exact for flat art. Quantette has no per-colour weights, so we'd feed it one entry per pixel.
- **Determinism:** seeded (`seed`, default 0), so deterministic on one platform. Not verified is whether native and WASM agree bit for bit: `wide` SIMD is used for f32 distances, and task 14 verified byte-identical browser and CLI output.
- **Wu quantiser:** `quantette::wu` is a vetted Wu implementation. SPEC lists "Wu (in-house)" as a swap candidate, so this could back a second, simpler adapter. It also can't pin.
- **Other deltas:** every golden under `fixtures/jolteon/expected/` would change.

**Recommendation: keep in-house.** The swap saves about 175 lines (`kmeans.rs`), loses pinned centres (a SPEC requirement), and adds about 60–78 KB. If pins were ever dropped, `quantette` is the one to try.

---

## 2. Colour mapping and merging: `flatart-mapping` (`ColourMapper`)

About 1,100 lines plus a 485-line test file. It covers:
- edge-tone folding
- pixel-weighted centroid merging, with the full merge order kept as a `SimilarityTree`
- edit-log replay with stable IDs and dropped-edit reporting
- base-colour choice
- output names

| Candidate | What it offers | Why it doesn't fit |
|---|---|---|
| `kodama` 0.3.0 (2023-01, MIT; about 208k recent downloads) | Agglomerative linkage (Lance–Williams, NN-chain) | Unmaintained since 2023. No per-observation weights, so no pixel-weighted centroids. Our merge needs the edge-tone fold first, and the merge order must carry ΔE and match `ColourMapper` semantics. A hand-rolled loop over at most 32 items is shorter than the glue. |
| `linfa-hierarchical` 0.8.1 | Same idea | About 450 recent downloads. Pulls in the linfa stack. |
| `palette` 0.7.7 (already a workspace dependency) | sRGB ↔ OKLab | **Adopt, small:** `colour_space.rs` (106 lines) re-implements OKLab by hand. `flatart-quant-kmeans/src/lab.rs` already uses `palette`. |
| `deltae` 0.3.2 | CIEDE2000 and others in CIELAB | The SPEC ΔE is 100 × Euclidean distance in OKLab, so it doesn't apply. |
| `color-name` 1.2.0 | CSS colour names | Our `NAMES` table is a curated, filament-flavoured list (`Cream`, `OrangeYellow`, `Mint` …) with a chroma-weighted metric. CSS names would change the names that go on 3MF parts. |

**Recommendation: keep in-house.** Optional tidy: have `colour_space.rs` call `palette`, matching the quantiser. Behaviour is unchanged up to float rounding, and the output names and merge order need re-checking against the goldens. No new dependency.

---

## 3. Clean-up and thin-feature stages: `flatart-raster` (`RasterStages`)

The relevant parts are `clean_up.rs` (mode filter and speckle merge), `thin.rs` (opening by a disc, then reassignment) and `grid.rs`.

| Candidate | Version | Licence / maintenance | WASM (verified) |
|---|---|---|---|
| **`imageproc`** | 0.27.0 · 2026-06-02 | MIT; about 3.1M recent downloads; image-rs project | Builds for wasm32 with `default-features = false` (no rayon, fft or text). Probe using `open` and `connected_components`: **42 KB** raw. Its `Cargo.toml` depends on `getrandom` with `wasm_js`, `rand`, `nalgebra` and `image`; none of them was linked into the probe. |
| `visioncortex` 0.9.3 | MIT/Apache | Has speckle filtering inside its clustering, but it's tied to its own cluster types. | Builds. |

**`imageproc` against the contract:**
- **Mode filter:** none. It has `median_filter`, which isn't a mode, and no rank or majority filter. The spec also requires "changes only if strictly more common, outside cells ignored".
- **Speckle merging:** `region_labelling::connected_components` replaces `grid::areas` (about 20 lines). The rule "merge into the neighbour sharing the longest border, smallest first, keep areas that touch only the outside" isn't covered.
- **Opening by a disc:** `morphology::open` and `Mask::disk(radius)` exist, but:
  - they work on `GrayImage` (`image` types), so every label map needs conversion;
  - the exact disc is built from "distances rounded up" (equal to a diamond for radius ≤ 2), whereas the contract and `thin.rs` use cell centres within the circle for an arbitrary diameter in cells (`k` fits in `u8`);
  - it runs per colour, and `open` takes `u8` radii.
- **Not covered:** reassigning removed cells to the colour that most surrounds them, the outline-never-changes rule, and `thin_features` reporting.
- **Net:** at most about 90 of the roughly 600 lines (`grid.rs`, `thin.rs`, `clean_up.rs`) could go, and parity on the goldens would be at risk.

**Recommendation: keep in-house.** `imageproc` is a fine library but doesn't cover the contract's distinctive parts.

---

## 4. Border tracing: `flatart-trace-graph` (`BorderTracer`)

About 900 lines: pixel-edge graph, chains split at junctions, staircase simplification, fitting through the injected `CurveFitter`, crossing detection with tighter refits, and face assembly.

**Contract points that matter:** shared borders (each used by exactly one loop per side); borders only meet at their ends; fitting only through the injected `fitter` with `settings.fit`; refit when curves would cross; deterministic.

| Candidate | Version / date | Licence | Maintenance | WASM (verified) |
|---|---|---|---|---|
| **`vtracer` 1.0 alpha** (library crate, `mosaic` mode) | 1.0.0-alpha.4 · 2026-08-29. Stable on crates.io is still 0.6.5 (2025-10), which has no library API. | MIT/Apache | The 1.0 alphas started 2026-07-24. 7.5k stars. About 66k recent downloads. Its issue tracker showed 0 open when fetched. | Builds. Full pipeline probe (`Config` + cutout + `to_svg`): **185 KB** raw, about +26% on our 708 KB. Depends on `visioncortex` 0.9.3 and `flo_curves` 0.8 (which brings in `flo_curves` 0.3 too, via `visioncortex`). |
| `visioncortex` 0.9.3 | 2026-08-14 | MIT/Apache | Same project. | Builds. |
| `contour`, `imageproc::contours`, `geo` | – | – | – | Contour tracing per binary region: no shared borders or junctions. |
| `i_overlay` 9.1.0 (polygon booleans) | 2026-10-14 | MIT/Apache | Active, same author as `i_triangle`. | Could be used to build the topology, but needs polygon-level work we already do on the pixel lattice. |

**Why `vtracer` mosaic is the real contender.** Source read (downloaded crate):
- Its mosaic mode is the same architecture as ours: label map → `BoundaryGraph::extract` (integer lattice, junction `Node`s, `Segment`s with `left` and `right` region) → each segment fitted once → composed. Adjacent regions share the same fitted geometry, one reversed. The graph types are public and `LabelMap` takes an external label map.
- **What it would replace:** only our `grid.rs` + `chains.rs` (about 290 lines). The faces assembler (`face.rs`) is private, so `faces.rs` and the compose step wouldn't be reusable. `simplify.rs`, `curves.rs` and `crossings.rs` stay.
- **Fitter mismatch:** its `SegmentFitter` trait fits one `Segment` (integer lattice points) with no settings. Our port wants the tracer to call the injected `CurveFitter` with `settings.fit` and to refit crossing borders more tightly. An adapter would have to implement `SegmentFitter` by delegating to our fitter.
- **Crossings:** its own spline fitter doesn't promise "no crossings". The comments claim non-crossing only for the polygon simplifier. So `crossings.rs` and the refit loop stay.
- **Alpha:** 1.0 is four months old, API unstable, and `BoundaryGraph` is internal machinery of a binary-targeted app.

**Recommendation: keep in-house.** The swap would save about 290 lines and add alpha-quality code plus about 185 KB (or less if only `BoundaryGraph` is used, not measured). Revisit once `vtracer` 1.0 is stable and if the graph types are documented as public API. SPEC's swap-candidate cell ("a `vtracer`-based tracer that also removes overlaps") predates mosaic mode. It could be reworded, but I haven't touched SPEC.

---

## 5. Curve fitting: `flatart-fit-kurbo` (`CurveFitter`)

About 650 lines including tests. It splits at corners, builds a Catmull-Rom spline through each run, then calls `kurbo::fit_to_bezpath`. It works around kurbo's behaviour on exactly straight and closed sources, and checks the tolerance itself.

| Candidate | Version / date | Licence | Maintenance | WASM |
|---|---|---|---|---|
| **`kurbo`** (current) | 0.13.1 · 2026-05-13 | Apache-2.0 OR MIT | Linebender; about 29M recent downloads | Already shipping |
| **`flo_curves`** | 0.8.1 · 2026-08-25 | **Apache-2.0 only** | About 200k recent downloads | Builds (compiled as a dependency of the `vtracer` probe). Deps: `itertools`, `ouroboros` (proc macro), `roots`, `smallvec`. |
| `visioncortex` spline | 0.9.3 | MIT/Apache | Active | Builds; uses the older `flo_curves` 0.3. |
| `bezier-rs` | 0.5.0 · 2025-08 | MIT/Apache | About 45k recent downloads | Not probed. A Graphite helper crate with a broader API. |
| `lyon_geom` | 1.0.20 | MIT/Apache | Active | Geometry types only, no polyline fitter. |

**`flo_curves` is the one worth an experiment.**
- Source read: `fit_curve(points, max_error) -> Option<Vec<Curve>>` is a Schneider-style fit straight to a polyline, and there's a `fit_curve_loop` variant for closed runs. That's the shape of our port, so it might remove the Catmull-Rom pre-step and the closed-halves and straight-run workarounds in `kurbo_fit.rs` (160 lines).
- **Unknown:** whether it holds the tolerance in both directions, keeps ends exact, and keeps collinear control points on the line. `deviation.rs` and the testkit would catch those, but the fitted curves would differ, so the goldens move. Its `max_error` is also approximate, like kurbo's.

**Recommendation: keep kurbo.** The adapter is thin and working. Optional, low-risk experiment: a `flatart-fit-flo` adapter (new crate, one wiring line) to compare curve count and runtime on Jolteon. It needs the user's approval for the dependency, and the Apache-2.0-only licence is acceptable under the MIT/Apache rule.

---

## 6. Mesh extrusion: `flatart-mesh` (`MeshBuilder`)

About 850 lines. It flattens each border once, builds regions (faces plus enclosed outside areas), triangulates through the injected `Triangulator`, and stitches watertight prisms. The key contract point is that parts meeting along a border use identical edge points.

| Candidate | Version | Licence | Note |
|---|---|---|---|
| `csgrs` | 0.20.1 · 2025-07-24 | MIT | Sketch → extrude with holes, with a wasm build flag. About 5k recent downloads, last release 16 months ago. Heavy dependencies (`nalgebra`, `geo`, …). Each extrusion triangulates on its own, so neighbouring faces don't get identical edge points, and the BSP/CSG machinery is floating-point throughout. Not probed. |
| `manifold-csg` | – | – | Wraps the C++ Manifold library: not pure Rust. |
| `ifc-lite-geometry`, `fontmesh`, `boolmesh` | – | – | Building-model or font-specific extruders. Not general. |
| `lyon_tessellation`, `earcut`, `i_triangle` | – | – | Triangulation only. `i_triangle` is already behind our `Triangulator` port, and the SPEC lists `lyon_tessellation` and `earcutr` as swaps for that port. |

**Recommendation: keep in-house.** The triangulation, the part that has good crates, is already outsourced. What remains is topology-specific stitching (shared border points, base merging, enclosed-hole filling) that no crate provides.

---

## Ranked recommendations

| # | Adapter | Verdict | Next best option |
|---|---|---|---|
| 1 | `flatart-quant-kmeans` | **Keep** | `quantette` 0.6.0, only if pinned centres are dropped from the SPEC |
| 2 | `flatart-mapping` | **Keep** | none; optionally use `palette` (already a dependency) for OKLab |
| 3 | `flatart-raster` | **Keep** | `imageproc` for connected components only, not worth it |
| 4 | `flatart-trace-graph` | **Keep** | `vtracer` 1.0 `mosaic`, once stable |
| 5 | `flatart-fit-kurbo` | **Keep** | `flo_curves::fit_curve` as an experiment (a new adapter, so it can run alongside) |
| 6 | `flatart-mesh` | **Keep** | none |

## Follow-ups that need the user's say-so
1. **Use `palette` in `flatart-mapping`.** No new dependency. Removes about 40 hand-written lines; the goldens need a re-check.
2. **`flo_curves` fitter experiment.** New dependency (Apache-2.0). Only worth it if the 160-line workaround file in `kurbo_fit.rs` is a maintenance burden.
3. **Update the SPEC adapter table.** The `vtracer` row predates mosaic mode, and the `kmeans_colors` row could cite `quantette` as the checked alternative. SPEC is not edited here.
