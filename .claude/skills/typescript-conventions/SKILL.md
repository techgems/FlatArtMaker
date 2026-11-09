---
name: typescript-conventions
description: FlatArtMaker TypeScript conventions. MUST be used whenever creating or editing any .ts file under web/ (including .svelte.ts modules and the <script lang="ts"> of .svelte files).
---

# TypeScript conventions (web/)

## Types
- **`strict` mode is on; never use `any`.** Use `unknown` and narrow it.
- **Don't silence the compiler:**
  - no non-null `!`, unless a comment says why it's safe
  - no `@ts-ignore`; use `@ts-expect-error` with a reason, and only if unavoidable
- **States** are discriminated unions or string-literal unions (`'loading' | 'ready' | 'failed'`), never booleans that can contradict each other.
  - Branch on them with an exhaustive `switch` whose `default` assigns to `never`.
- **No `enum`.** Use literal unions or `as const` objects.
- **Prefer `readonly`** fields and `ReadonlyArray` for data that isn't mutated.
- **Never hand-write types that mirror Rust types.** Import the generated ones from `src/generated/`.

## Modules and imports
- Use `import type` for type-only imports (`verbatimModuleSyntax`).
- Use named exports. Default exports are only for Svelte components.
- Code outside a module imports it only through its `index.ts`.
- File names are `kebab-case.ts`; reactive modules are `kebab-case.svelte.ts`; components are `PascalCase.svelte`.

## Boundaries
- **Interfaces describe capabilities** (`EngineClient`, `MeshViewer`, `ProjectStore`).
  - Implementations are created by factory functions (`createWorkerEngineClient()`).
  - Tests use fakes that implement the same interface.
- **Nothing outside `modules/engine/`** touches the WASM package or `Worker`.

## Async and errors
- **No floating promises.** `await` them, or prefix with `void` when fire-and-forget is intended.
- **Throw `Error` instances only.** Convert unknown caught values with `cause instanceof Error ? cause.message : String(cause)`.
- **Wrap browser storage** (`localStorage`, IndexedDB) in try/catch. It can be unavailable.

## Style
- Small, single-purpose functions: pure where possible, side effects at the edges.
- Comments explain *why*, not *what*.
