# 12: 3MF exporter

**Depends on:** 02, 10 · **Spec:** Pipeline step 9 (3MF) · **Crate:** `flatart-3mf`

## Goal
Implement `Exporter<MeshModel>` for 3MF, plus the `prusa` and `bambu` `SlicerDialect` adapters, following [spikes/3mf/FINDINGS.md](../../spikes/3mf/FINDINGS.md).

## Inputs from task 02
- **The aim:** separate bodies, each on its colour's extruder, in the slicer the file is made for. One file for every slicer is out of scope.
- **`prusa`:** one object with one mesh, with triangle ranges per part in `Metadata/Slic3r_PE_model.config` (`name`, `extruder`). `slic3rpe:Version3mf` is required.
- **`bambu`:** one mesh object per part under a parent `<components>` object, with `Metadata/model_settings.config` (`name`, `extruder` per part). Don't write the `Application` tag.
- **Extruders:** output colour slot *n* goes on extruder *n*, the same in both dialects.

## Deliverables
- **Model:** one part per output colour (plus the base when B > 0), each named after its colour.
- **Colours:** an `m:colorgroup` with the display colours, in both dialects.
- **Slicer data:** `SlicerDialect` adapters `prusa` and `bambu`. Each writes `3D/3dmodel.model` in its own part layout (including the `m:colorgroup`) plus its `Metadata/` config. Shared XML helpers stay private to the crate.
- **Package:** the exporter is built with a `Box<dyn SlicerDialect>` and adds `[Content_Types].xml`, `_rels`, the thumbnail and the zip.
- **Thumbnail:** `Metadata/thumbnail.png`, a top-down render of the output label map.
- **Isolation:** `zip` and `quick-xml` are used only inside this crate.

## Acceptance
- The `Exporter` and `SlicerDialect` testkit suites pass for both dialects.
- Unit tests per dialect: one part per output colour (plus the base), named after it, and output colour slot *n* on extruder *n*.
- **Jolteon in the `prusa` dialect** opens in PrusaSlicer, and **in the `bambu` dialect** opens in Bambu Studio or an Orca build. In both, it opens as separate parts on the right extruders, with no splitting and no repair warnings.
- The file is much smaller than the 10.8 MB reference.
