# 03: STEP test

**Depends on:** 01 · **Spec:** STEP library policy, Pipeline step 9 (STEP), Milestones 0

## Goal
Prove that `truck` (git-pinned) can write named, solid straight extrusions that Fusion and FreeCAD import cleanly, natively **and** in WASM.

## Steps
1. In `spikes/step/`, depend on `ricosjp/truck` (`truck-modeling`, `truck-stepio`, `truck-assembly`), pinned to the current `master` commit.
2. Build the shapes:
   - a 40 × 40 mm square base (3.4 mm thick)
   - two islands of one colour, each 1 mm thick on top of it
   - one island with a cubic-Bézier hole
3. Export STEP with **both** naming strategies:
   - (a) named `MANIFOLD_SOLID_BREP`s in one part (patch the names into the output if `truck` writes `''`)
   - (b) an assembly with named `PRODUCT`s
4. Compile the same code for `wasm32-unknown-unknown`, and record the `.wasm` size after `wasm-opt -Oz`.
5. Import into FreeCAD (scripted with `FreeCADCmd`: shape valid, solid count, names) and Fusion (by hand). Sanity-check in Onshape.
6. If `truck` fails any check, repeat with `monstertruck` 0.4.x. If that also fails, write down the scope of the hand-written writer.

## Acceptance
- `spikes/step/FINDINGS.md` records:
  - the chosen library and commit
  - the chosen naming strategy
  - the WASM size
  - screenshots or notes from FreeCAD and Fusion showing the bodies `Base`, `Black`, `Black-1`
- The `FreeCADCmd` check script is saved for reuse in task 19's golden tests.
