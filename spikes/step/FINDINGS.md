# Task 03 findings: STEP

**Date:** 2026-11-09
**Status:** `truck` passes every automated check, and the samples open in **Fusion with no issues** (checked by the user, 2026-11-09). **Still open:** an Onshape sanity check (optional).

## Library
- **`truck`** from `https://github.com/ricosjp/truck`, pinned to `rev = 55548a27de0af3e87cceeb677b809aa110a87028` (master, 2026-11-02).
- **Crates:** `truck-modeling` 0.6.0, `truck-topology` 0.6.0, `truck-assembly` 0.1.0 and `truck-stepio` 0.3.0.
- **The reader is optional.** Writing only needs `truck-stepio` with `default-features = false`. Its reader feature (`in`) doesn't compile unless `derive` is also enabled, which is a bug in truck's manifest.
- **`monstertruck`** was not tried, because truck passed.

## Shapes ([src/lib.rs](src/lib.rs))
| Name | Solid | Faces |
|---|---|---|
| `Base` | 40 × 40 square, z 0 to 3.4 | 6 |
| `Black` | 30 × 15 rectangle, z 3.4 to 4.4, with a hole bounded by 4 cubic Bézier curves | 10 |
| `Black-1` | pentagon, z 3.4 to 4.4 | 7 |

All three are a planar face swept straight up (`tsweep`).
- **Curved edges** are written as cubic B-spline curves (`B_SPLINE_CURVE_WITH_KNOTS`, degree 3).
- **Side walls under curved edges** become B-spline surfaces (degree 3 × 1); straight sides become `PLANE`.
- Both are exact.

## Naming strategies
**(a) Named solids in one part: works, but only by patching the text afterwards.**
- truck always writes `MANIFOLD_SOLID_BREP('', …)`, and its API can't set that name.
- `patch_brep_names` fills the names in by file order. That's safe because truck writes entities in a fixed order.
- **Fusion ignores these names** (checked 2026-11-09), so this strategy is rejected.

**(b) Assembly with one named part per solid: works natively.**
- Built from `truck_assembly` `Assembly` and `StepDesign`. It gives a `FlatArt` root and one child `PRODUCT('Base'|'Black'|'Black-1')` per solid, with no patching.
- **Quirk:** truck writes the parent-child link in the reverse order from OpenCASCADE. This is harmless with identity placements, and Fusion opens the file fine.

Fusion shows `Base`, `Black` and `Black-1` only from the assembly file.

**Chosen: (c) grouped bodies** (user, 2026-11-09). Bodies are grouped by colour: one named component per colour, holding that colour's bodies. (d) was rejected because its sub-components add too much clutter when editing the STEP in CAD. Two grouped layouts are in [samples/](samples/). Each has a 100 × 60 base and 5 colours (`Black`, `White`, `Red`, `Yellow`, `Blue`) with 2 bodies each, and every first body has a Bézier hole.

| Sample | Layout | What Fusion should show |
|---|---|---|
| `grouped-bodies.step` | **(c)** root → `Base` plus one component per colour. Each colour's bodies sit together inside its one component, and the body names are patched in. | components `Black`, `White`, …; bodies named `Black`/`Black-1`, or `Body1`/`Body2` if Fusion ignores solid names as in (a) |
| `grouped-subcomponents.step` | **(d)** root → `Base` plus one component per colour. Each colour holds one sub-component per body (`Black` → `Black`, `Black-1`). | components `Black`, `White`, …, each with named sub-components |

Both layouts were checked automatically in the same way as above: 11 closed solids, the products and links expected, no missing references, every edge used once in each direction, and identical bytes on two runs. Both were opened in Fusion.

## Patches the adapter must apply
1. **Schema.** truck hard-codes `FILE_SCHEMA(('ISO-10303-203'))`, which isn't a valid schema name, and the field is private. Both samples are patched to `AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }` (AP214).
2. **Timestamp.** The default STEP header reads the system clock (`chrono::Utc::now`), so the adapter must set `time_stamp` explicitly to keep output deterministic.
3. **Not needed:** truck writes `FACE_SURFACE` and a plain `SHAPE_REPRESENTATION` instead of `ADVANCED_FACE` and `ADVANCED_BREP_SHAPE_REPRESENTATION`, and Fusion accepts both as written.

## Automated validation
- **Re-reading the files with truck-stepio** (`cargo run --release --bin step-check`):
  - Both files parse, each with 3 solids, each solid one closed shell, with 6, 10 and 7 faces.
  - Every edge starts and ends on its vertices, and the bounding boxes are correct.
  - The assembly's part names come back as `FlatArt`, `Base`, `Black`, `Black-1`.
  - Two runs give identical bytes.
- **Independent structural check** ([p21_lint.py](p21_lint.py)):
  - Header and trailer are present.
  - No references point at missing entities.
  - Each of the 51 edges is used exactly once in each direction.

## WASM
The same code builds for `wasm32-unknown-unknown`. Run in Node 24, it produces output identical to the native build, and generating both files takes about 5 ms.

| Stage | Bytes | gzip |
|---|---|---|
| raw `.wasm` from cargo | 1,118,742 | ~245 KB |
| after `wasm-bindgen` 0.2.130 | 744,094 | ~134 KB |
| after `wasm-opt -Oz` | **253,496** | **~104 KB** |

`chrono` and `rayon` come in through truck's own dependencies. They aren't used when writing STEP and add almost nothing after `-Oz`. This supports the spec's separate, on-demand STEP WASM module.

## Hand checks
- **Fusion:** opens with no issues.
- **Onshape:** not checked (optional).
- **Naming:** Fusion shows body names only for (b), the assembly. Of the colour-grouped layouts, (c) was chosen.

Task 19's golden tests reuse [step-check.rs](src/bin/step-check.rs) and the checks in [p21_lint.py](p21_lint.py).

## Reproduce (PowerShell, from `spikes/step/`)
    cargo run --release --bin step-check
    cargo build --lib --target wasm32-unknown-unknown --release --no-default-features
    node run-wasm.cjs   # after wasm-bindgen + wasm-opt from the wasm-pack cache; see run-wasm.cjs
