# 02: Slicer 3MF test

**Completed:** 2026-11-09 · commit pending

**Depends on:** 01 · **Spec:** Pipeline step 9 (3MF), Milestones 0

## Goal
Find the minimal 3MF structure that a slicer opens as **separate bodies**, one part per output colour, with **each body on its colour's extruder** (output colour slot *n* on extruder *n*), so the user doesn't have to split parts or reassign extruders.

Out of scope: one file that works perfectly in every slicer.

## Steps
1. Unzip the user's projects in `fixtures/slicer/` and record which files and keys matter.
2. Hand-write two-colour square 3MFs with a throwaway script in `spikes/3mf/`.
3. Open them in PrusaSlicer and in Bambu Studio or an Orca build, and iterate.
4. Write the result in `spikes/3mf/FINDINGS.md`.

## Acceptance
- For each slicer family, one sample in `spikes/3mf/samples/` opens as separate parts on the right extruders, with no splitting and no repair prompts.
- `FINDINGS.md` lists every required file and key per dialect. Task 12 implements it from there.

## Result (2026-11-09)
- **Two dialects are needed:** `prusa-dialect.3mf` meets the goal in PrusaSlicer, and `bambu-dialect.3mf` meets it in Bambu Studio and Orca builds.
- **Neither works in the other family.** The Prusa format opens there as one body; the Bambu format opens in PrusaSlicer as separate objects, all on extruder 1.
- Checked by the user in the PrusaSlicer and Snapmaker Orca GUIs, and by the PrusaSlicer and OrcaSlicer CLIs. See [FINDINGS.md](../../spikes/3mf/FINDINGS.md).
