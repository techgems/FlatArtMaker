# 18: Saved projects

**Completed:** 2026-11-13 · commit 7ed0bd8

**Depends on:** 06, 15 · **Spec:** Saved projects

## Goal
Autosave each project and reopen it later.

## Deliverables
- **Module:** `modules/projects` with the `ProjectStore` interface.
- **Storage implementation:**
  - small data (project list, names, settings, edit log) in `localStorage`, under the `flatart:` key prefix
  - the image `Blob` in IndexedDB
  - an in-memory fake for tests
- **Project fields:** `id`, `name`, `createdAt`, `updatedAt`, the original image (bytes + MIME type), the edit log with its undo position, `Settings` and `schemaVersion`. Derived data is never stored.
- **Behaviour:**
  - autosave about 500 ms after the last change
  - a "Recent projects" list with rename and delete (with confirmation)
  - migrations for older `schemaVersion`s
- **Storage failures:**
  - storage full: a notice offering "delete old projects"
  - storage unavailable: a notice, and the app keeps working

## Acceptance
- Vitest with the fake store covers: save → load gives identical settings and edits, a migration from version 1, and the storage-full and unavailable paths.
- Manual check: edit a Jolteon project, reload the page, reopen it. The sliders, mapping, painted areas and undo history are restored.

## Outcome (2026-11-13)
- **Done:** the store, autosave, Recent projects (rename, confirmed delete), and the storage-full and unavailable notices. Vitest covers each acceptance case.
- **Manual check:** done on Jolteon. Sliders, mapping, the undone paint and the undo position all came back after a reload.
  - There's no colour editor yet (task 16), so the recorded edit log (`fixtures/jolteon/edits.json`, undo position 3) was written into the saved project directly.
- **Migrations:** version 1 is the only format so far, so no real migration exists yet.
  - The test runs a version 1 record through two test-only steps, and checks that a record from a newer app is refused, not overwritten.
- **Session:** now keeps the edit log (`applyEdits`) for task 16 to use.
