# Work split

Each file is one self-contained task, derived from `SPEC.md`. Rules:
- **One task at a time.** A task is done when every item under **Acceptance** passes.
- **Each task has its own commit (or PR).** When it's done, close it as `.claude/CLAUDE.md` describes: move the file to `done/`, then record the date and commit here and in the file.
- If a task needs a spec change, stop and ask the user. `SPEC.md` is never edited silently.

## Order and dependencies

| # | Task | Depends on | Spec milestone | Completed |
|---|---|---|---|---|
| - [x] 01 | [Toolchain, build and tooling](done/01-toolchain-and-build.md) | – | setup | 2026-11-09 · 2573bb7 |
| - [x] 02 | [Slicer 3MF test](done/02-slicer-spike.md) | 01 | 0 | 2026-11-09 · 95c7a3a |
| - [x] 03 | [STEP test](done/03-step-spike.md) | 01 | 0 | 2026-11-09 · 95c7a3a |
| - [x] 04 | [Domain types and ports](done/04-domain-and-ports.md) | 01 | 1 | 2026-11-09 · 95c7a3a |
| - [x] 05 | [Colour detection adapter](done/05-colour-detection.md) | 04 | 1 | 2026-11-09 · 2b9ef33 |
| - [x] 06 | [Colour mapping and edit log](done/06-colour-mapping.md) | 04, 05 | 1 | 2026-11-10 · 8a2c5a3 |
| - [x] 07 | [Raster stages](done/07-raster-stages.md) | 04 | 1 | 2026-11-10 · 2a50404 |
| - [x] 08 | [Border tracer](done/08-border-tracer.md) | 04, 09 | 1 | 2026-11-10 · a5a3c65 |
| - [x] 09 | [Curve fitting adapter](done/09-curve-fitting.md) | 04 | 1 | 2026-11-10 · a5a3c65 |
| - [x] 10 | [Mesh builder](done/10-mesh-builder.md) | 04, 08 | 1 | 2026-11-11 · c0a55ad |
| - [x] 11 | [SVG exporter](done/11-export-svg.md) | 08 | 1 | 2026-11-11 · c0a55ad |
| - [x] 12 | [3MF exporter](done/12-export-3mf.md) | 02, 10 | 1 | 2026-11-11 · c0a55ad |
| - [x] 13 | [CLI and golden tests](done/13-cli-and-golden-tests.md) | 05–12 | 1 | 2026-11-11 · c0a55ad |
| - [x] 14 | [WASM engine API](done/14-wasm-engine-api.md) | 13 | 2 | 2026-11-11 · 4cf4b19 |
| - [ ] 15 | [UI shell and controls](15-ui-shell-and-controls.md) | 14 | 2 | – |
| - [ ] 16 | [Colour-mapping editor](16-colour-mapping-editor.md) | 06, 15 | 3 | – |
| - [ ] 17 | [3D preview](17-3d-preview.md) | 15 | 3 | – |
| - [ ] 18 | [Saved projects](18-saved-projects.md) | 06, 15 | 3 | – |
| - [x] 19 | [STEP export](done/19-step-export.md) | 03, 08, 14 | 4 | 2026-11-12 · 8d6e5ed |
| - [ ] 20 | [Deploy](20-deploy.md) | 15 | 5 | – |
| - [ ] 21 | [Lighter STEP files](21-step-size.md) | 19 | 4 | – |

## Tasks that can run in parallel

| Once done | These can run in parallel |
|---|---|
| 01 | 02, 03 and 04 |
| 04 | 05, 07 and 09 |
| 15 | 16, 17 and 18 (each touches a different module) |

## Shared conventions
- **Architecture and style:** `.claude/CLAUDE.md`.
- **Every adapter task** adds its crate to the workspace, implements exactly one port, and runs that port's `testkit` suite.
- **Only these files may change in the wiring:** `flatart-cli`, `flatart-wasm*`.
