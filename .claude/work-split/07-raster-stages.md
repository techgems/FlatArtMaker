# 07: Raster stages

**Depends on:** 04 · **Spec:** Pipeline steps 1, 4, 5 and 6 · **Crate:** `flatart-raster` (in-house, domain types only)

## Goal
Handle every pixel-level stage between mapping and tracing.

## Deliverables
- **Load:** apply the alpha threshold (or the chosen background colour) to mark *outside*, and compute the art bounds and aspect ratio.
- **Clean up:** a mode filter, then speckle merging into the neighbour with the longest shared border.
- **Region painting:** flood-fill the connected area under each stored point with its output colour.
- **Printability:**
  - optional nearest-neighbour upscale
  - the thin-feature fix when `MinFeatureWidth` is set: per colour, shrink then regrow by half the width, with removed pixels going to the neighbour that most surrounds them
  - pixels-per-mm comes from `PrintScale`
- **Feedback:** a "thin features remaining" report that the UI can show as warnings.
- **Finish the `RasterStages` port** (`flatart-ports`, provisional since task 04): settle its signatures and contract against this implementation, and add its testkit suite. The stage result types in `flatart-domain` (`LoadedImage`, `Printable`, `ThinFeature`) are provisional too.

## Acceptance
- **Unit tests on synthetic label maps:**
  - a 1 px line disappears at 0.25 mm when the fix is on and stays when it's off
  - painting changes only the clicked area
  - speckles are merged
  - the bounds ignore transparent padding
- No stage outputs a label that wasn't in its input (outside included).
