# 14: WASM engine API

**Depends on:** 13 · **Spec:** Stack, Front end (`web/`)

## Goal
Expose the full pipeline to the browser through the worker, with types generated from Rust.

## Deliverables
- **`flatart-wasm`:** wires the same adapters as the CLI and keeps one `Pipeline` per project in the worker, so stage caching works across calls.
- **Exports:**
  - `loadImage(rgba, w, h)`
  - `setSettings(settings)`
  - `applyEdits(log)`
  - `previews()` (the mapped label map and the vector model)
  - `mesh()` (buffers for three.js)
  - `export3mf()` and `exportSvg()`
- **Generated types:** `tsify` produces the `Settings`, `EditLog`, palette and report types under `web/src/generated/`.
- **Front end:**
  - `EngineClient` grows to match.
  - Large buffers are transferred, not copied (`Comlink.transfer`).
  - Image decoding stays in the browser (`createImageBitmap`).

## Acceptance
- A Vitest test with a fake `EngineClient` covers the client contract.
- A manual check: in `npm run dev`, loading the Jolteon PNG through a temporary debug button downloads a 3MF identical to the CLI's output.
- The release WASM size is recorded in `.claude/work-split/NOTES.md`.
