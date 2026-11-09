# 08: Border tracer

**Depends on:** 04, 09 · **Spec:** Core data model, Pipeline step 7 · **Crate:** `flatart-trace-graph`

## Goal
Implement `BorderTracer`: turn a label map into the shared-border map.

## Deliverables
- **Tracing:**
  - Walk the pixel edges between differing labels (outside included).
  - Split at junctions (3 or more labels meet, or 2 labels meet the image edge).
  - Closed loops with no junction get an anchor point.
- **Simplification:** simplify each border with its end points fixed, rejecting any change that would make two borders cross.
- **Curve fitting:** through the injected `&dyn CurveFitter`, with the smoothing tolerance and corner threshold.
- **Faces:** assemble each label's faces (outer loop plus holes) from its borders.
- **Isolation:** no direct dependency on any curve library. Fitting goes only through the port.

## Acceptance
- The `BorderTracer` testkit suite passes:
  - every border has two different sides
  - faces are closed
  - no borders cross
  - the face count matches the connected components
- Property tests (`proptest`) on random label maps keep those invariants.
- Traces the Jolteon label map in under 1 s natively in release mode.
