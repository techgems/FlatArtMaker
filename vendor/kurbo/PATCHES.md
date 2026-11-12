# kurbo 0.13.1, patched for FlatArtMaker

This is the crates.io release of kurbo 0.13.1 (upstream commit
`838b69ef35f3381cefc007d86aa8364f5a6dc673`). The workspace uses it in place
of the registry crate through `[patch.crates-io]` in the root `Cargo.toml`.
Only `flatart-fit-kurbo` depends on kurbo.

Drop this copy, and the `[patch]` entry, once a kurbo release fixes the bug.

## The bug

`fit_to_bezpath` hung forever on some traced borders (first seen on a
750 × 750 px flat-colour image).

1. For some ranges, the cubic fitter proposes a candidate curve with a control
   point far away (about 2e13 px).
2. On "spicy" sources it measures each candidate with `inv_arclen`.
3. `inv_arclen` asks the ITP root finder for a bracket narrower than
   `accuracy / total_arclen`. With `accuracy = 1e-9` and a length of about
   2e13, that is 5e-23, finer than f64 can resolve in `0..1`.
4. The root finder's loop could never end.

The regression tests are in `crates/flatart-fit-kurbo/src/tests.rs`
(`kurbo_fit_finishes_on_a_long_candidate` and the others next to it).

## Changes

- `src/param_curve.rs`, `inv_arclen`: the bracket width is floored at
  `f64::EPSILON`. This also keeps the root finder's `1u64 << nmax` in range.
- `src/common.rs`, `solve_itp_fallible`: the loop stops once the next probe
  isn't strictly inside the bracket, since the bracket can't shrink any
  further.

Both are marked `FlatArtMaker patch` in the source. Neither changes a result
that the unpatched code would have returned.
