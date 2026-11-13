# 15: UI shell and controls

**Completed:** 2026-11-12 · commit ba322ef

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
- **Header:** Download 3MF, STEP and SVG. STEP calls `EngineClient.exportStep`, whose first call also fetches the STEP module (task 19), so the button shows a spinner while it runs. Next to Download 3MF is a slicer choice (PrusaSlicer, or Bambu Studio / Orca), remembered between visits.
- **Settings changes:** debounced. Only affected stages re-run.

## Acceptance
- All Svelte files pass `svelte-autofixer`. `npm run check`, `lint` and `test` pass.
- **Vitest covers:**
  - the X/Y linking: typing X sets Y from the aspect ratio
  - the slider limits: C can't be 0, the threshold can't go below 0.25
  - B = 0 disables the base-only controls
- A Playwright smoke test: upload the Jolteon PNG → download a 3MF.

## Outcome (2026-11-12)
- **Done:** the modules above plus `modules/session` (the open image, its settings and results) and `modules/colours` (the summary chips). Check, lint, the autofixer and the Vitest cases pass, and the app was checked by hand on Jolteon.
- **Not done:**
  - **The Playwright smoke test.** It needs `@playwright/test` as a new dependency; the user closed the task without it for now.
  - **"Edit colours"** is disabled until task 16's modal exists.
- **Found on the way:** the 3MF failed at 280 mm and up (looped fitted curves); fixed in c29f5d9.
