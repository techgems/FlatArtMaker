# 15: UI shell and controls

**Depends on:** 14 · **Spec:** UI, controls tables

## Goal
Build the single-page app: upload, controls, 2D previews and downloads.

## Deliverables
- **Modules:**
  - `modules/upload`
  - `modules/controls` (its own `utils/` for unit formatting and clamping)
  - `modules/preview-2d`
  - `modules/downloads`
- **Upload:** a notice that only logo or vector-like art is supported, shown before upload.
- **Left panel:**
  - **Primary sliders:** C (never 0) and B (0 means no base), each with a number box.
  - **X/Y size:** one slider plus linked X and Y boxes, with the aspect ratio locked.
  - **Printability:** the toggle plus the threshold (0.25–1.2 mm).
  - **Advanced:** a collapsed section.
- **Centre:** tabs for Original, Mapped colours and Vector (SVG). The 3D tab is a placeholder until task 17.
- **Right:** the output-colour summary chips and an "Edit colours" button (opens task 16's modal).
- **Header:** Download 3MF, STEP (disabled until task 19) and SVG. Next to Download 3MF is a slicer choice (PrusaSlicer, or Bambu Studio / Orca), remembered between visits.
- **Settings changes:** debounced. Only affected stages re-run.

## Acceptance
- All Svelte files pass `svelte-autofixer`. `npm run check`, `lint` and `test` pass.
- **Vitest covers:**
  - the X/Y linking: typing X sets Y from the aspect ratio
  - the slider limits: C can't be 0, the threshold can't go below 0.25
  - B = 0 disables the base-only controls
- A Playwright smoke test: upload the Jolteon PNG → download a 3MF.
