# 16: Colour-mapping editor

**Depends on:** 06, 15 · **Spec:** Pipeline steps 3 and 5, UI → Colour-mapping editor

## Goal
Build the "Edit colours" modal, laid out like MakerWorld's "Edit Color Details" dialog.

## Deliverables
- **Module:** `modules/colour-mapping` (own `components/`, `utils/`, state). The editor works on a draft copy of the edit log: Confirm commits it, Cancel throws it away.
- **Canvas:**
  - a checkerboard background, plus an optional overlay of the original image
  - zoom and pan
  - undo/redo and zoom groups in the bottom-left corner
  - hovering a mapping row highlights that colour's pixels
- **Colours in use:**
  - numbered chips (drag to reorder)
  - clicking a chip opens its details: picker, hex, eyedropper, name, base, delete with a reassignment target
  - **+** to add, the brush to toggle paint mode, and an n / 12 counter
- **Mapping:** a two-column grid of detected circle → output chip, a target picker with "+ New colour", "Reset mapping", and "Pin a missed colour".
- **Paint mode:** hover outlines the area, click paints it, shift-drag paints several. Esc or B toggles paint mode.
- **Keyboard:** Ctrl+Z / Ctrl+Shift+Z. **Inline warnings:** an empty output colour, thin features, the 12-colour limit.

## Acceptance
- All Svelte files pass `svelte-autofixer`.
- Vitest covers the draft log: Confirm and Cancel, and undo/redo.
- **A manual check on Jolteon:**
  - remap a detected colour
  - paint one area
  - add a colour
  - reorder slots

  The 3MF reflects every change.
