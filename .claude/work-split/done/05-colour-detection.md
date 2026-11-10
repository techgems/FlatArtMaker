# 05: Colour detection adapter

**Completed:** 2026-11-09 · commit 2b9ef33

**Depends on:** 04 · **Spec:** Pipeline step 2 · **Crate:** `flatart-quant-kmeans`

## Goal
Implement `ColorQuantizer` with k-means in OKLab.

## Deliverables
- **Behaviour:**
  - Sample the opaque pixels (alpha threshold).
  - Run k-means up to the "Detected colours (max)" setting.
  - Honour **pinned colours** as fixed centres that k-means doesn't move.
  - Output a `DetectedPalette` (with coverage %) and a detected `LabelMap`. Outside pixels get the reserved outside label.
- **Determinism:** seeded randomness. The same input always gives the same output.
- **Isolation:** `palette` is used only inside this crate. The k-means is in-house (`kmeans_colors` doesn't build for WASM and can't fix centres).

## Acceptance
- The `ColorQuantizer` testkit suite passes.
- **Unit tests:**
  - a pinned colour survives unchanged
  - transparent pixels map to outside
  - a 2-colour synthetic image gives exactly 2 detected colours
- On the Jolteon PNG, detection runs in under 1 s natively in release mode.
