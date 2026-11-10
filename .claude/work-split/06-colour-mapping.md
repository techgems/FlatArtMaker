# 06: Colour mapping and edit log

**Depends on:** 04, 05 · **Spec:** Pipeline steps 3 and 5 (operations, edit log, stable IDs) · **Crate:** `flatart-mapping`

## Goal
Implement the automatic detected → output mapping, and replaying the edit log, as in-house code with no third-party dependencies, behind the `ColourMapper` port.

## Deliverables
- **Automatic mapping:**
  - Agglomerative merging in OKLab, weighted by pixel count.
  - **Count only:** keep merging the most similar pair until at most the user's "Output colours" count remains (1–16). There is no ΔE merge threshold.
  - **Similarity map:** return the full merge order, down to one group, so the editor can show which tones belong together and what merges next if the user lowers the count.
  - Name each output colour after the nearest common colour name, keeping names unique and limited to letters and digits.
- **Replaying the edit log:** apply the operations in order.
  - **Operations:** `Map`, `PaintRegion` (stores a source-pixel point plus an output ID), `AddOutput` (at most 16 output colours in all), `ChangeOutputColour`, `DeleteOutput` (with a reassignment target), `Rename`, `SetBase`, `Reorder`, `PinDetected`, `ResetMapping`.
  - **Undo and redo:** move the log position. Never mutate past entries.
- **Re-matching:** when detection changes, match each stored detected ID to the nearest new colour within ΔE. Return a report of the operations that were dropped.
- **Outputs:** the output label map (after mapping) and the list of paint operations for task 07. Painting is applied after clean-up.
- **Finish the `ColourMapper` port** (`flatart-ports`, provisional since task 04): settle its signature and contract against this implementation, and add its testkit suite.

## Acceptance
- Unit tests for each operation, undo/redo, the 16-colour limit, unique names, and re-matching with a dropped-operation report.
- A property test: replaying any valid log twice gives the same result.
