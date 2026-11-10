# 10: Mesh builder

**Depends on:** 04, 08 · **Spec:** Pipeline step 8 · **Crates:** `flatart-tri-itriangle` (adapter), `flatart-mesh`

## Goal
Turn a `VectorModel` into a watertight `MeshModel`.

## Deliverables
- **`flatart-tri-itriangle`:** implements `Triangulator` using `i_triangle`.
- **`flatart-mesh`:**
  - **Flattening:** each border is converted to straight segments **once**, at the curve tolerance, scaled by `PrintScale`.
  - **Base (B > 0):** the outline is the borders that touch outside. Optionally fill holes. It runs from z=0 to z=B.
  - **Colour layer:** one mesh per output colour, from z=B to z=B+C.
  - **Base-colour faces:** when B > 0, merged into the base.
  - **No base (B = 0):** the colour layer starts at z=0, with no base body and no base colour.
  - **Isolation:** triangulation goes only through the `Triangulator` port.
- **Finish the `MeshBuilder` port** (`flatart-ports`, provisional since task 04): settle its signature and contract against this implementation, and add its testkit suite. Use `testkit::mesh::assert_watertight` for the watertight check.

## Acceptance
- The `Triangulator` testkit suite passes.
- **Meshes are watertight:** every edge is used exactly twice and normals point outward. Checked by a reusable testkit helper.
- Adjacent colours share identical edge points.
- The bounding box equals the X × Y from `PrintScale`, within 0.01 mm.
- B = 0 produces no base and colours that start at z=0.
