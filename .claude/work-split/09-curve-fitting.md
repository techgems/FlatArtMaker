# 09: Curve fitting adapter

**Depends on:** 04 · **Spec:** Pipeline step 7 · **Crate:** `flatart-fit-kurbo`

## Goal
Implement `CurveFitter`: fit cubic Béziers to a polyline with its end points fixed.

## Deliverables
- **Fitting:** use `kurbo`, within the tolerance. Turns sharper than the corner threshold stay sharp corners.
- **Output:** `Vec<CubicBezier>` (a domain type). `kurbo` types never leave the crate.

## Acceptance
- The `CurveFitter` testkit suite passes:
  - the end points are exact
  - the maximum deviation is within tolerance
  - a straight input gives straight segments
- A unit test shows a square's corners stay sharp.
