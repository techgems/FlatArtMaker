# 20: Deploy

**Depends on:** 15 · **Spec:** Stack (Hosting), Milestones 5

## Goal
Publish `web/dist/` as a static site.

## Deliverables
- **CI (GitHub Actions):**
  - `cargo fmt`, clippy and tests
  - `npm ci`, `npm run build`, lint and tests
  - the `dist/` folder uploaded as an artifact
- **Hosting:** Cloudflare Pages (preferred), with Netlify or Amplify as alternatives.
  - The build runs in CI, where the Rust toolchain is installed. The host only serves `dist/`, so it never needs Rust.
- **Headers:**
  - `.wasm` is served as `application/wasm`
  - long cache times for hashed assets
  - no cross-origin isolation is needed unless WASM threads are added later

## Acceptance
- A push to `main` deploys.
- The site works in current Chrome, Firefox and Safari.
- A Lighthouse run shows no blocking errors.
