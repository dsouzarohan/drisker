# AGENTS.md

Guide for coding agents working in Drisker. Read this first, then `PLAN.md` for
the current increment and `README.md` for project status and setup.

## What this is

A Windows-first system inspection and maintenance CLI written in Rust. Help
users understand their systems before deciding whether to change anything.
Inspection and explanation are useful on their own.

The current implementation is a single binary crate with clap, help/version
output, and CLI tests. Disk scanning, driver discovery, and maintenance are
not implemented. Do not present roadmap features as available commands.

This is also a Rust and systems learning project. Explain meaningful ownership,
data-structure, and OS API choices in the context of the change. Keep code and
diffs understandable without turning every task into a tutorial.

## How work is organised

- `PLAN.md` holds the immediate increment: small, verifiable tasks. Update
  checkboxes as work completes; refresh the plan when an increment finishes.
- `ROADMAP.md` holds feature-level milestones, not detailed implementation tasks.
- `docs/architecture.md` describes boundaries and testing strategy.
- `docs/adr/` records meaningful architectural decisions.
- Docs follow code: update affected instructions and documentation in the same
  change. Distinguish local checks from hosted CI and native platform results.
- Keep shared guidance in the repository, independent of any particular agent.
  Add scripts or supporting documents when there is a concrete need.

## Implementation boundaries

- Stay with one crate and focused modules. Add traits, dependencies, or workspace
  boundaries for real requirements rather than hypothetical extensibility.
- Prefer idiomatic safe Rust, explicit errors, and deliberate ownership. The
  crate currently forbids unsafe code; revisit that only for a concrete,
  documented integration need.
- Keep application logic separate from CLI rendering. Future JSON stdout must
  contain only structured output, with no progress, styling, or prompts.
- Windows is the product target; macOS is a development and compilation target.
  Isolate Windows integrations with conditional compilation when introduced.
  A macOS build does not verify Windows API behavior.
- Prioritize correctness. Measure before introducing parallelism or caching.

## Safety and scope

- Inspection is read-only by default. No cleanup or driver installation belongs
  in the foundation; start feature work only when requested.
- Future mutation requires a concrete proposal, verification, explicit user
  authorization, execution, and result reporting. Occupied storage is not proof
  of deletion safety; an available update is not a recommendation to install it.
- Driver discovery uses supported Windows APIs and deterministic sources. Do not
  add AI decisions, crawlers, speculative matching, or custom cryptography.
- Tests use controlled fixtures and require no administrator access, live device
  inventory, or mutations to system directories, drivers, or privileged settings.

## Commands and verification

Use stable Rust. Run from the repository root:

```sh
cargo run --locked -- --help
cargo run --locked -- --version
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked
```

Run formatting, Clippy, tests, and build checks for code changes. For
documentation-only changes, check accuracy and links; no Rust test run is
needed. Keep `Cargo.lock` in version control. CI runs native Windows x86_64 and
macOS ARM64 checks; report unverified behavior honestly.

## Git and communication

- Use feature branches and pull requests for `main`; do not bypass its ruleset.
- Use Conventional Commits. No generated attribution or co-author trailers.
- Keep PR descriptions short: behavior, significant choices, and verification.
- Comments and Rust documentation describe behavior. Put design rationale and
  trade-offs in the PR description or an ADR when they warrant a durable record.
