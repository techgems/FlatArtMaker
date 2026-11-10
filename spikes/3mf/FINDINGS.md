# Task 02 findings: slicer 3MF

**Date:** 2026-11-09
**Checked in:** PrusaSlicer 2.9.7 (GUI and CLI), Snapmaker Orca (GUI; reads Bambu Studio and OrcaSlicer files) and OrcaSlicer (CLI).

## Goal
The 3MF only needs to do two things, in the slicer it's made for:
1. **Load as separate bodies**, one part per output colour, with no "split to parts" needed.
2. **Put every body on its colour's extruder.** Output colour slot *n* goes on extruder *n* (for example, every `Yellow` body on extruder 3).

Matching every slicer is out of scope. What matters is that the user doesn't have to reassign extruders by hand, which is what happens with a 3MF from Fusion: random colours in PrusaSlicer, and everything on extruder 1 in Orca.

## Result
Each format meets the goal only in its own slicer family, so we need **two dialects**:

| Sample | PrusaSlicer | Bambu Studio / Orca |
|---|---|---|
| [prusa-dialect.3mf](samples/prusa-dialect.3mf) | ✅ separate parts, correct extruders | ❌ one body with one colour; only "split to parts" reveals the rest |
| [bambu-dialect.3mf](samples/bambu-dialect.3mf) | ❌ opens, but as separate objects all on extruder 1 | ✅ separate parts, correct extruders |

## Prusa dialect
| File | Content |
|---|---|
| `[Content_Types].xml`, `_rels/.rels` | standard 3MF boilerplate |
| `3D/3dmodel.model` | **one** `<object>` with **one** `<mesh>` holding every part's triangles in part order, plus `<metadata name="slic3rpe:Version3mf">1</metadata>` (**required**: without it the parts are placed wrongly) |
| `Metadata/Slic3r_PE_model.config` | the object's `name`, plus one `<volume firstid lastid>` per part (an inclusive triangle range) with keys `name` and `extruder` (1-based) |

Verified by slicing on the Core One INDX profile: the tool switches T0 → T1 where the second colour starts.

## Bambu dialect
| File | Content |
|---|---|
| `[Content_Types].xml`, `_rels/.rels` | standard 3MF boilerplate |
| `3D/3dmodel.model` | one mesh `<object>` per part, plus a parent `<object>` whose `<components>` reference them, with the `BambuStudio` namespace and `BambuStudio:3mfVersion` |
| `Metadata/model_settings.config` | the parent `<object>` with `name` and `extruder`, plus one `<part id objectid subtype="normal_part">` per component with keys `name` and `extruder` |

**Leave out the `Application` metadata tag.** Without `Metadata/project_settings.config`, it crashes the OrcaSlicer CLI.

## For task 12
- Two `SlicerDialect` adapters built from the same mesh model: `prusa` and `bambu`.
- Both use the same mapping from colour to extruder: output colour slot *n* goes on extruder *n*.
- The UI needs a slicer choice when downloading the 3MF.

## Reproduce
`python -I make_3mf.py out` writes every variant tried, including the ones that turned out not to be needed (colour groups, external object files, a hybrid of both formats).
