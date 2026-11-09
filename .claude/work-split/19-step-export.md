# 19: STEP export

**Depends on:** 03, 08, 14 · **Spec:** STEP library policy, Pipeline step 9 (STEP)

## Goal
Implement `Exporter<VectorModel>` for STEP, with the library and naming strategy chosen in task 03.

## Deliverables
- **`flatart-step-truck`** (or the fallback adapter task 03 chose):
  - one solid per island, plus `Base` when B > 0
  - names follow `Colour`, `Colour-1`, …, numbered by area (largest first), with ties broken top-to-bottom then left-to-right
  - flat top and bottom faces with B-spline edges, and straight vertical side walls
  - solids only: no surfaces and no mesh bodies
- **`flatart-wasm-step`:** a separate WASM module. `web/` loads it on the first "Download STEP" click, with a spinner meanwhile.
- **CLI:** a `--step out.step` flag.

## Acceptance
- The `Exporter` testkit suite passes.
- **Golden test via the saved `FreeCADCmd` script:** the shape is valid, the solid count is right, and the names match the expected list.
- Jolteon opens in Fusion with correctly named bodies and no repair needed.
- The main WASM bundle size is unchanged.
