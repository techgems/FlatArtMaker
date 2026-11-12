# 11: SVG exporter

**Completed:** 2026-11-11 · commit c0a55ad

**Depends on:** 08 · **Spec:** Pipeline step 9 (SVG) · **Crate:** `flatart-svg`

## Goal
Implement `Exporter<VectorModel>` for SVG.

## Deliverables
- **Content:** one `<path>` per face, built from the shared Béziers (with holes via `fill-rule="evenodd"`). Paths are grouped per output colour in a `<g id="{name}">`.
- **Units:** sized in mm from `PrintScale`, with a matching `viewBox`.
- **Dependencies:** none (a string writer).

## Acceptance
- The `Exporter` testkit suite passes: the output is deterministic and not empty.
- The output opens correctly in a browser and in Inkscape, with layers named after the output colours.
