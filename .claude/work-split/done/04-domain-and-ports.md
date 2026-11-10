# 04: Domain types and ports

**Completed:** 2026-11-09 · commit pending

**Depends on:** 01 · **Spec:** Architecture, Core data model, Pipeline, Controls

## Goal
Define the stable backbone: data types, capability interfaces and the pipeline skeleton. Later tasks only add implementations.

## Deliverables
- **`flatart-domain`:**
  - **Pixels and colours:** `RgbaImage`, `Oklab`, `DetectedPalette` (stable IDs = detected OKLab), `OutputPalette` (generated IDs, name, display colour, slot order, base flag), `ColourMapping`, `LabelMap`.
  - **Vector and mesh models:** `BorderMap` (`Junction`, `Border` with left/right labels, `Face`), `VectorModel`, `MeshModel`.
  - **Editing:** `EditLog` with the operation enum from Pipeline step 5 and the undo position.
  - **Settings:** a versioned `Settings` struct (serde) containing `LayerHeight` (> 0), `Option<LayerHeight>` for the base, `PrintScale` (mm per source pixel), `Option<MinFeatureWidth>` (≥ 0.25 mm) and the Advanced values.
  - **Rule:** constructors reject invalid values, so these types can't hold them.
- **`flatart-ports`:** traits with their contract docs:
  - `ColorQuantizer`
  - `BorderTracer`
  - `CurveFitter`
  - `Triangulator`
  - `Exporter<M>`
  - `SlicerDialect`: `id()` plus `parts(&MeshModel)` (the model carries its `OutputPalette`), which returns every slicer-specific file of a 3MF: the model part `3D/3dmodel.model` (part layout and names) and the slicer config under `Metadata/` (extruder per part). See SPEC "Interfaces".
  - **Naming:** each trait is named for its capability, never for a library. Signatures use domain types only.
- **`flatart-ports::testkit`** (behind a feature flag): one conformance suite per trait, generic over any implementation.
  - The `SlicerDialect` suite checks what's format-independent: a `3D/3dmodel.model` entry, no repeated paths, identical output on two runs. Each dialect's own tests (task 12) check part names and extruders.
- **`flatart-pipeline`:**
  - `Pipeline::builder()` takes `Box<dyn Trait>` per port.
  - The stage graph matches Pipeline steps 1–9.
  - Each stage caches its result, keyed by a hash of its inputs and settings. Downstream stages are invalidated on change.
  - Stages without an adapter yet return `Error::NotImplemented(stage)`.

## Acceptance
- `cargo test`, clippy and fmt pass.
- Unit tests show that invalid settings can't be constructed: C = 0, a min width below 0.25 mm, a negative base height.
- `flatart-pipeline` depends only on `flatart-domain` and `flatart-ports` (check with `cargo tree`).
- A short doc comment on each trait states its contract. The testkit suites compile against a trivial dummy implementation.
