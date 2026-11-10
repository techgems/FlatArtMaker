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
| - [ ] 05 | [Colour detection adapter](05-colour-detection.md) | 04 | 1 | – |
| - [ ] 06 | [Colour mapping and edit log](06-colour-mapping.md) | 04, 05 | 1 | – |
| - [ ] 07 | [Raster stages](07-raster-stages.md) | 04 | 1 | – |
| - [ ] 08 | [Border tracer](08-border-tracer.md) | 04, 09 | 1 | – |
| - [ ] 09 | [Curve fitting adapter](09-curve-fitting.md) | 04 | 1 | – |
| - [ ] 10 | [Mesh builder](10-mesh-builder.md) | 04, 08 | 1 | – |
| - [ ] 11 | [SVG exporter](11-export-svg.md) | 08 | 1 | – |
| - [ ] 12 | [3MF exporter](12-export-3mf.md) | 02, 10 | 1 | – |
| - [ ] 13 | [CLI and golden tests](13-cli-and-golden-tests.md) | 05–12 | 1 | – |
| - [ ] 14 | [WASM engine API](14-wasm-engine-api.md) | 13 | 2 | – |
| - [ ] 15 | [UI shell and controls](15-ui-shell-and-controls.md) | 14 | 2 | – |
| - [ ] 16 | [Colour-mapping editor](16-colour-mapping-editor.md) | 06, 15 | 3 | – |
| - [ ] 17 | [3D preview](17-3d-preview.md) | 15 | 3 | – |
| - [ ] 18 | [Saved projects](18-saved-projects.md) | 06, 15 | 3 | – |
| - [ ] 19 | [STEP export](19-step-export.md) | 03, 08, 14 | 4 | – |
| - [ ] 20 | [Deploy](20-deploy.md) | 15 | 5 | – |

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
