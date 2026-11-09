# 02: Slicer 3MF test

**Depends on:** 01 · **Spec:** Pipeline step 9 (3MF), Milestones 0

## Goal
Pin down the exact 3MF structure that opens in PrusaSlicer **and** Bambu Studio with parts on filaments 1 and 2.

## Input needed from the user
- A two-colour project saved from PrusaSlicer, e.g. two cubes on extruders 1 and 2. Put it in `fixtures/slicer/`.

## Steps
1. Unzip the user's project and record which parts matter:
   - `3D/3dmodel.model`
   - `Metadata/Slic3r_PE_model.config`: volumes, triangle ranges and the `extruder` key
   - `[Content_Types].xml` and `_rels`
2. Hand-write a minimal two-colour square 3MF (one object, two parts, an `m:colorgroup`, the Prusa config). Use a throwaway script in `spikes/3mf/`.
3. Open it in PrusaSlicer and Bambu Studio. Iterate until each part shows on the right filament with its name.
4. Write down the minimal required files and keys in `spikes/3mf/FINDINGS.md`.

## Acceptance
- One `.3mf` that both slicers open with the correct filament per part and the correct part names, with no repair prompts.
- `FINDINGS.md` lists every required file, element and key. Task 12 implements it from there.
- If Bambu Studio drops the filament assignment, note it there: task 12 then adds a Bambu `SlicerDialect`.
