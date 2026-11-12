# 21: Lighter STEP files

**Depends on:** 19 · **Spec:** Pipeline step 9 (STEP)

## Goal
Make the STEP files much smaller without changing the shape: today Jolteon's is 34 MB, and the 52 test images average 32 MB. Fusion opens them, but slowly, and the browser has to build and hand over a file that size.

## Where the size comes from (Jolteon, 2026-11-12)
- **One edge per fitted curve.** Every Bézier from the fitter becomes its own edge at the bottom and at the top, plus a vertical edge at each end and a wall face: 14,324 faces, 43,494 edges and 531,149 entities in all.

| Entity | Count | MB | Share |
|---|---|---|---|
| `CARTESIAN_POINT` | 197,168 | 14.1 | 42% |
| `ORIENTED_EDGE` | 86,988 | 4.2 | 12% |
| `EDGE_CURVE` | 43,494 | 2.5 | 7% |
| `B_SPLINE_CURVE_WITH_KNOTS` | 16,924 | 2.4 | 7% |
| `DIRECTION` | 38,990 | 2.4 | 7% |
| `B_SPLINE_SURFACE_WITH_KNOTS` | 8,462 | 1.8 | 5% |

- **Repeated points.** A curve's control points are written once for its bottom edge, again for its top edge, and again (both heights) for its wall surface. Vertices repeat the curve ends.
- **Text overhead.** truck writes about 3.6 MB of indentation and line breaks, and full-precision numbers (about 10 characters each, 800k numbers).

## Approaches to evaluate
Measure each one on Jolteon (size, export time) before choosing. They combine.
1. **Fewer, longer edges.** Join each border's smooth stretch (between corners and junctions) into one B-spline edge and one wall face, keeping the shape exact (knots repeated at the joins). This cuts faces and edges the most, but the joins are only tangent-continuous, not curvature-continuous, so it **needs a new Fusion check** (and Onshape if possible).
2. **Share entities.** Write each point, direction and curve once and refer to it from every user, e.g. a wall surface reusing its edges' control points.
3. **Compact text.** Drop the indentation and write numbers with only the precision needed (for example 1e-6 mm), keeping vertices exactly on their curves.
4. **Fewer curves from the fitter.** The visioncortex fitter gives about 40% fewer curves than kurbo. This is a fitter choice, not a STEP change, so it's for comparison only.

Approaches 2 and 3 can be done as a post-processing pass on truck's text, or by writing the entities ourselves; record which and why. Anything that changes how truck builds the geometry stays inside `flatart-step-truck`.

## Acceptance
- Jolteon's STEP is at most a quarter of its current size (≤ 8.5 MB), or the task records why that isn't reachable and what was reached.
- The shape is unchanged: every body, edge and face matches the current output to within 1e-6 mm (for example by sampling points on each face of the old and new files).
- Every check from task 19 still passes on the 52 test images, with and without a base: each body reads back as one closed solid with the expected names, no reference is missing, and every edge is used once in each direction.
- Output stays deterministic; the golden tests pass after re-recording.
- Jolteon opens in Fusion with no repair, with the same components and bodies (user check).
- The main WASM bundle is unchanged, and the STEP module doesn't grow by more than about 10%.
