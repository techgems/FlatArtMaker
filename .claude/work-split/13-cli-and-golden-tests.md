# 13: CLI and golden tests

**Depends on:** 05–12 · **Spec:** Milestones 1, Coding standards (tests)

## Goal
Wire every adapter natively and lock the behaviour down with golden tests.

## Deliverables
- **`flatart-cli`:**
  - **Argument parsing:** uses `clap`.
  - **Wiring:** the only place that names the concrete adapters.
  - **Usage:** `flatart in.png out.3mf [--svg out.svg] --output-colors 8 --base 3.4 --layer 1.0 --size-x 100 [--no-thin-fix | --min-width 0.25] [--edits edits.json]`.
- **Fixtures:** `fixtures/jolteon/` holds the input PNG and the expected outputs.
- **Golden tests:**
  - Jolteon with the defaults
  - Jolteon with B = 0 (no base part, colours from z=0, no gaps)
  - a recorded edit log replayed (mapping + painting)
- **Property tests:** across the whole pipeline.

## Acceptance
- `cargo test --workspace` runs the golden tests and passes.
- Running the CLI on Jolteon produces a 3MF that meets the task 12 acceptance criteria.
