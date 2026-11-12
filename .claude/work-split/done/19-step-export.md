# 19: STEP export

**Completed:** 2026-11-12 · commit 8d6e5ed

**Depends on:** 03, 08, 14 · **Spec:** STEP library policy, Pipeline step 9 (STEP)

## Goal
Implement `Exporter<VectorModel>` for STEP with `truck`, grouping bodies into one named component per colour (the grouped-bodies layout chosen in task 03).

## Deliverables
- **Inputs from task 03** ([spikes/step/FINDINGS.md](../../spikes/step/FINDINGS.md)):
  - `truck` pinned to `rev = 55548a27de0af3e87cceeb677b809aa110a87028`
  - **grouped bodies:** a root `PRODUCT` with one child `PRODUCT` per colour, plus `Base`. Each colour's single `SHAPE_REPRESENTATION` holds all that colour's solids, as in `grouped_bodies` in `spikes/step/src/lib.rs`. No sub-component per body.
  - also patch the body names into each `MANIFOLD_SOLID_BREP` (`Black`, `Black-1`, …, in file order). Fusion may show them as `Body1`, `Body2`, … anyway.
  - patch `FILE_SCHEMA` to AP214 (`AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }`)
  - set the header `time_stamp` explicitly, so output stays deterministic
  - `truck-stepio`'s reader feature needs `derive` enabled; use it in tests only
- **`flatart-step-truck`:**
  - one solid per island, plus `Base` when B > 0
  - one component per output colour, named after the colour; body names inside follow `Colour`, `Colour-1`, …, numbered by area (largest first), with ties broken top-to-bottom then left-to-right
  - flat top and bottom faces with B-spline edges, and straight vertical side walls
  - solids only: no surfaces and no mesh bodies
- **`flatart-wasm-step`:** a separate WASM module. `web/` loads it on the first "Download STEP" click, with a spinner meanwhile.
- **CLI:** a `--step out.step` flag.

## Acceptance
- The `Exporter` testkit suite passes.
- **Golden test:** re-read the output with `truck-stepio` (as in `spikes/step/src/bin/step-check.rs`). Every shell is closed, the solid count is right, and the names match the expected list. Also run the spike's structural checks: no references to missing entities, and every edge used exactly once in each direction.
- Jolteon opens in Fusion with one correctly named component per colour (plus `Base`) and no repair needed.
- The main WASM bundle size is unchanged.

## Notes (2026-11-12)
- **Base colour:** its areas are bodies of their own in the `Base` component (`Base-1`, …), not fused into the slab: fused, the slab's top face touches itself where two of them meet at a corner, which truck rejects (21 of 52 test images). Accepted by the user; SPEC updated.
- **Browser:** `EngineClient.exportStep` fetches the STEP module on its first call. The "Download STEP" button and its spinner are part of the header in task 15. Until then the dev-only debug export control has a STEP option.
- **Fusion:** Jolteon opened cleanly (user, 2026-11-12).
- **Size:** one edge and wall face per fitted curve gives files of about 30 MB. Follow-up: [task 21](../21-step-size.md).
