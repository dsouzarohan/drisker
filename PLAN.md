# Immediate plan

## State at handoff

The MIT-licensed foundation is merged at `3ed1da4` (PR #2). The feature branch
implements `drisker disk scan <path> [--json]` in one binary crate, version
`0.1.0`. Driver discovery and maintenance remain unimplemented.

AGENTS.md holds standing workflow instructions. This is the only execution plan;
ROADMAP.md holds future features and completed milestone summaries. Codex's
native generated memory supplies local recall outside the codebase. Do not
create repository memory files or parallel feature plans.

## Next increment: first read-only disk scan

**Status:** Contract, model, traversal, output, and fixtures implemented.
Local checks pass. Native Windows/macOS CI is the remaining verification step;
do not confuse earlier foundation CI with scanner validation.

**Goal:** `drisker disk scan <path>` reports observed logical bytes and entry
counts beneath a directory, with clear limitations and partial-result reporting.
It does not claim allocated disk usage or identify safe-to-delete data.

1. [x] Define the initial behavior contract with small output examples: accepted
   roots, logical-size accounting, hard-link counting, links/reparse points,
   volume boundaries, changes during scanning, and fatal versus partial results.
   Set exit statuses, text/JSON fields, and diagnostic limits before coding.
2. [x] Introduce a focused disk module with request/result types and checked
   aggregation. Keep application results separate from rendering; retain one
   crate and avoid speculative platform traits or a full tree representation.
3. [ ] Implement sequential read-only traversal with explicit failure reporting.
   Do not follow links or Windows reparse points whose traversal safety is not
   established. Verify metadata handling on native Windows. Resolve depth and
   resource-limit behavior without silently omitting subtrees.
4. [x] Expose the working `disk scan <path>` command with text and `--json`
   rendering from the same result. Add serialization dependencies only as
   needed; keep progress, styling, and diagnostics off JSON stdout.
5. [x] Add controlled temporary fixtures for empty/nested directories, known
   byte totals, spaces/Unicode, links, invalid roots, and partial results.
   Cover overflow and error-limit behavior deterministically. Gate native
   cases and avoid reliance on installed hardware or administrator privileges.
6. [ ] Run formatting, Clippy, tests, and builds locally and in native
   Windows/macOS CI. Document counting rules and unverified edge cases in README
   and architecture docs; record the outcome before choosing the next increment.

Largest-entry views, allocated-size accounting, interactive exploration,
categorization, and driver work remain later roadmap features. Correctness
comes before concurrency or caching; measure before optimizing.

## Validation and next action

- Local Apple Silicon / Rust 1.99.1: formatting, Clippy with warnings denied,
  24 unit tests, 9 binary integration tests, build, help, version, and a JSON
  scan of `src` pass with locked dependencies.
- The Unix socket fixture requires an unsandboxed test run; it passes with
  normal user permissions. No administrator access is needed.
- Unit tests inject metadata, enumeration, depth, and overflow failures;
  the same traversal loop is used in production. Native Windows includes a
  junction fixture; native CI results are pending.
- [PR #3](https://github.com/dsouzarohan/drisker/pull/3) contains the scanner diff.
  Initial hosted macOS checks pass; native Windows junction classification and
  traversal tests pass. Windows found a CRLF-sensitive documentation-example
  test, now normalized before extraction. Next: verify the corrected revision
  in native CI and record the outcome before refreshing this plan.
- Unverified scope: arbitrary reparse tags, cloud/network providers, real
  mounted-volume boundaries, and concurrent path replacement. The contract
  explicitly describes live observations and no physical containment guarantee.
