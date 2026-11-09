# FlatArtMaker: agent instructions

- **Source of truth:** `SPEC.md`. Do not edit it unless the user explicitly asks.
- **Work items:** `.claude/work-split/`. Start at `00-index.md`. Do one task file at a time; its acceptance criteria define done.

## Session output
- Precise and succinct. No fluff, no filler, no restating the request, no praise.
- Report results and blockers, not process. Use file links and short lists.

## Build
| Task | Command |
|---|---|
| Rust tests | `cargo test --workspace` |
| Rust lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Rust format | `cargo fmt --all --check` |
| Dev server (from `web/`) | `npm run dev`: builds the WASM in dev mode, then runs Vite |
| Production build (from `web/`) | `npm run build`: release WASM, then `npm run check`, then `vite build` into `web/dist/` (static, deployable as is) |
| Type check (from `web/`) | `npm run check` |
| Web lint (from `web/`) | `npm run lint`: ESLint, then Prettier `--check` |
| Web format (from `web/`) | `npm run format` |
| Web tests (from `web/`) | `npm test` (Vitest) |

- **How the builds combine:** `wasm-pack` (an npm dev dependency) compiles `crates/flatart-wasm` into `web/src/generated/flatart-wasm/`. That folder is generated and git-ignored; never edit it. `web/src/modules/engine/engine.worker.ts` imports it, and Vite bundles the worker and the `.wasm` file.
- **Prerequisites:**
  - MSVC C++ build tools (the host linker, needed for proc macros even when targeting WASM)
  - the `wasm32-unknown-unknown` target, installed automatically by `rust-toolchain.toml`
  - Node 24+

## Modularity (all code)
Modularity is imperative. Every unit has one clear job and a narrow public surface.

## Rust
- **Modularity is 100% paramount.** Single responsibility at every level: crate, module, type, function.
- **Consumers and providers are separate:**
  - **Consumers** (`flatart-pipeline`, wiring crates) depend only on **capability interfaces** in `flatart-ports`, named for what they do: `ColorQuantizer` (detect colours), `BorderTracer` (vectorise), `CurveFitter`, `Triangulator`, `Exporter<VectorModel>` (SVG, STEP), `Exporter<MeshModel>` (3MF), `SlicerDialect`.
  - **Providers** (adapter crates) implement one interface each. The third-party library is a private implementation detail of that crate.
- **Swapping a dependency touches only its adapter crate plus one line in the wiring crates.** Examples: k-means → median cut, the in-house tracer → vtracer, `truck` → `monstertruck`.
  - If a change would alter a port signature, a domain type or the pipeline flow, stop and flag it to the user.
- **No third-party types in the public API** of `flatart-domain`, `flatart-ports` or `flatart-pipeline`. Adapters convert at their edge.
- **Every port** has a documented contract and a conformance suite in `flatart-ports::testkit`. Every adapter runs that suite.
- **Only wiring crates name concrete adapters** (`flatart-cli`, `flatart-wasm*`). Everything else takes `Box<dyn Trait>`.
- **Stages:**
  - pure: input plus settings in, output out
  - deterministic: no I/O, no clocks, seeded randomness
  - exporters return bytes
- **Types:**
  - units are newtypes (`Mm`, `Px`)
  - invalid values can't be constructed (`LayerHeight`, `MinFeatureWidth`, `PrintScale`)
- **Errors:** `thiserror` per crate, mapped to `flatart_domain::Error`. Nothing panics across the WASM boundary.
- **Safety:** `unsafe_code` is forbidden everywhere except `flatart-wasm*`, where it is `deny`.

## Front end (Svelte 5 + TypeScript, Vite, no SvelteKit)
- **Skills:**
  - Before creating or editing any `.svelte`, `.svelte.ts` or `.svelte.js` file, invoke `svelte-code-writer` and `svelte-core-bestpractices`.
  - Prefer delegating that work to the `svelte-file-editor` agent.
  - Validate every changed Svelte file with `npx @sveltejs/mcp svelte-autofixer <file> --svelte-version <package.json version>` until it reports no issues.
  - For any `.ts` file, invoke `typescript-conventions`.
- **The DRY spectrum:** aim for **60–70% DRY**.
  - Extract shared logic and components where reuse is natural.
  - If a component can only be made reusable through contortions (a growing pile of props or mode flags), write separate components even if they look alike.
  - Duplication is cheaper than the wrong abstraction.
- **Structure is by module:** `src/modules/<module>/` holds `components/`, `utils/`, state (`*.svelte.ts`) and `index.ts`.
  - A whole module may have its own `utils/` folder. Utils live with the module they serve; there's no global grab-bag.
  - `src/shared/` is only for code that is truly cross-module.
  - Other modules import a module **only through its `index.ts`**.
- **State:**
  - Components display things.
  - State lives in `.svelte.ts` classes using `$state`, shared with `createContext`.
- **Boundaries:** the UI never imports WASM or three.js directly. It goes through the `EngineClient`, `MeshViewer` and `ProjectStore` interfaces.
- **Shared types:** types shared with Rust are generated (`tsify`) and never hand-copied.
