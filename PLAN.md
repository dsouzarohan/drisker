# Immediate plan

## State at handoff

The first read-only disk scanner increment is complete in
[PR #3](https://github.com/dsouzarohan/drisker/pull/3), awaiting review and merge.
The MIT-licensed foundation remains merged on main at `3ed1da4` (PR #2).
Check Git and PR state before resuming; the scanner is on
`feat/read-only-disk-scan`, not yet on main. Version remains `0.1.0`.

`drisker disk scan <path> [--json]` reports observed logical bytes and counts,
with explicit skip policy, partial results, checked counters, and bounded
traversal/diagnostics. Domain results and rendering are separate. See the
[contract](docs/disk-scan-contract.md), [architecture](docs/architecture.md), and
[completed milestone](ROADMAP.md#2-basic-read-only-disk-scanning) for details.
Driver discovery and maintenance remain unimplemented.

## Verification evidence

- Local Apple Silicon / Rust 1.99.1: formatting, Clippy with warnings denied,
  24 unit tests, 9 binary integration tests, build, help, version, and a JSON
  scan of `src` pass with locked dependencies. The Unix socket fixture passes
  outside the local sandbox with normal user permissions; no elevation needed.
- Native Windows x86_64 and macOS ARM64
  [scanner CI](https://github.com/dsouzarohan/drisker/actions/runs/42418864283)
  passes formatting, Clippy, tests, and builds at `6158e1a`. Windows verifies
  junction reparse classification, skipped traversal, and root rejection.
- Documentation links, JSON example syntax, and whitespace pass local checks.
- Known limits: arbitrary reparse tags, cloud/network providers, real mounted
  volumes, and concurrent replacement are not comprehensively verified. Results
  are live observations, not snapshots or physical-containment guarantees.

## Next increment: select the first filesystem-accounting extension

**Status:** Scope not selected; no next feature implementation started.

1. [ ] Review the scanner diff in PR #3 and resolve any findings before merge.
2. [ ] Select one concrete requirement from roadmap milestone 3, such as
   allocated-size reporting or identity-based hard-link accounting. Define its
   behavior and native Windows fixture requirements before adding APIs.
3. [ ] Replace this selection plan with small implementation/verification tasks
   for the chosen requirement.

AGENTS.md holds standing workflow instructions. This is the only execution plan;
ROADMAP.md retains milestone outcomes and later features.
