# 17: 3D preview

**Depends on:** 15 · **Spec:** Stack (3D preview), Front end (`MeshViewer`)

## Goal
Show the 3D tab behind a `MeshViewer` interface.

## Deliverables
- **Module:** `modules/preview-3d` with the `MeshViewer` interface and the `createThreeMeshViewer()` implementation (three.js plus `OrbitControls`).
- **Content:** one mesh per output colour, using the display colours. It updates when the mesh buffers change, without recreating the renderer.
- **Isolation:** three.js is imported only inside this module's implementation file.

## Acceptance
- All Svelte files pass `svelte-autofixer`.
- The B = 0 and B > 0 Jolteon models render correctly.
- Changing a slider updates the view in under 300 ms after the engine responds.
- No WebGL resources leak when switching tabs or projects.
