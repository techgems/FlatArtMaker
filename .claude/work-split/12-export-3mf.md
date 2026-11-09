# 12: 3MF exporter

**Depends on:** 02, 10 · **Spec:** Pipeline step 9 (3MF) · **Crate:** `flatart-3mf`

## Goal
Implement `Exporter<MeshModel>` for 3MF, plus the PrusaSlicer `SlicerDialect`, following `spikes/3mf/FINDINGS.md`.

## Deliverables
- **Model:** one object with one part per output colour (plus the base when B > 0), each part named after its output colour.
- **Colours:** an `m:colorgroup` with the display colours.
- **Slicer data:** `SlicerDialect` adapter `prusa` writes `Metadata/Slic3r_PE_model.config`, with the filament per part in slot order.
- **Thumbnail:** `Metadata/thumbnail.png`, a top-down render of the output label map.
- **Isolation:** `zip` and `quick-xml` are used only inside this crate.

## Acceptance
- The `Exporter` testkit suite passes.
- The Jolteon output opens in PrusaSlicer and Bambu Studio with the correct filament per part and no repair warnings.
- The file is much smaller than the 10.8 MB reference.
