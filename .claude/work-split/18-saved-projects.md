# 18: Saved projects

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
