# 01: Toolchain, build and tooling

**Completed:** 2026-11-09 · commit 2573bb7

**Depends on:** – · **Spec:** Stack, Coding standards

## Goal
The scaffold builds end to end, and the lint and test tooling exists before feature work starts.

## Current state
- **Done:** Cargo workspace (`flatart-domain`, `-ports`, `-pipeline`, `-cli`, `-wasm`), `web/` (Svelte 5 + Vite + TypeScript), the engine module (`EngineClient` → Comlink worker → WASM), npm scripts that combine both builds, and `.gitignore`.
- **Verified (2026-11-09):**
  - The C++ build tools are installed: the "Desktop development with C++" workload and Windows SDK 10.0.26100.
  - `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` and `cargo run -p flatart-cli` all pass.
  - `npm run build` passes, producing a 19 KB `.wasm` file.
  - `npm run dev` and `npm run preview` both show "Engine v0.1.0" with no console errors.
  - ESLint (type-checked; bans `enum`, `any`, floating promises), Prettier and Vitest are set up. `npm run lint` and `npm test` pass (2 tests).
  - The repo is `techgems/FlatArtMaker`, and the first commit is `2573bb7`.
- **Remaining:** none.

## Steps
1. Install the "Desktop development with C++" workload in Visual Studio (the user does this; it needs admin).
2. Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo run -p flatart-cli`.
3. From `web/`: run `npm run dev`. The header must show "Engine v0.1.0".
4. From `web/`: run `npm run build`. Then `npm run preview` must serve `dist/` with the badge showing the version.
5. Set up the web tooling: ESLint (flat config, `eslint-plugin-svelte`, `typescript-eslint`), Prettier (`prettier-plugin-svelte`), and Vitest. Add the `lint`, `format` and `test` npm scripts.
6. Write one Vitest test for `EngineStatus` using a fake `EngineClient` (loading → ready, and loading → failed).
7. `git init`, then make the first commit.

## Acceptance
- All Rust commands in step 2 pass with zero warnings.
- `npm run build`, `npm run lint` and `npm test` pass.
- The browser shows the engine version from WASM, in both dev and preview.
