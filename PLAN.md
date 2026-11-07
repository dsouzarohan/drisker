# Immediate plan

## State at handoff

The repository foundation is complete: one Rust binary crate, clap help/version,
3 parser unit tests, 4 CLI integration tests, a committed lockfile, MIT licensing,
and native Windows/macOS CI. No scanner, driver discovery, or maintenance
operation is implemented. The binary remains `drisker` version `0.1.0`.

The bootstrap and agent instructions are published, including
[PR #1](https://github.com/dsouzarohan/drisker/pull/1).
[Foundation CI on main](https://github.com/dsouzarohan/drisker/actions/runs/42411773487)
passed formatting, Clippy, tests, and builds on Windows x86_64 and macOS ARM64.
That verifies the CLI foundation, not future Windows filesystem or driver APIs.
The current foundation-handoff change adds licensing, corrects stale validation
notes, and establishes session continuity. Check Git and the PR for its actual
merge state and current validation results.

AGENTS.md holds standing workflow instructions. This is the only execution plan;
ROADMAP.md holds future features and completed milestone summaries. Codex's
native generated memory supplies local recall outside the codebase. Do not
create repository memory files or parallel feature plans.

## Next increment: first read-only disk scan

**Status:** Planned, not started. The next action is task 1: settle the behavior
contract before implementing traversal. No scanner implementation is included
in the foundation-handoff change.

**Goal:** `drisker disk scan <path>` reports observed logical bytes and entry
counts beneath a directory, with clear limitations and partial-result reporting.
It does not claim allocated disk usage or identify safe-to-delete data.

1. [ ] Define the initial behavior contract with small output examples: accepted
   roots, logical-size accounting, hard-link counting, links/reparse points,
   volume boundaries, changes during scanning, and fatal versus partial results.
   Set exit statuses, text/JSON fields, and diagnostic limits before coding.
2. [ ] Introduce a focused disk module with request/result types and checked
   aggregation. Keep application results separate from rendering; retain one
   crate and avoid speculative platform traits or a full tree representation.
3. [ ] Implement sequential read-only traversal with explicit failure reporting.
   Do not follow links or Windows reparse points whose traversal safety is not
   established. Verify metadata handling on native Windows. Resolve depth and
   resource-limit behavior without silently omitting subtrees.
4. [ ] Expose the working `disk scan <path>` command with text and `--json`
   rendering from the same result. Add serialization dependencies only as
   needed; keep progress, styling, and diagnostics off JSON stdout.
5. [ ] Add controlled temporary fixtures for empty/nested directories, known
   byte totals, spaces/Unicode, links, invalid roots, and partial results.
   Cover overflow and error-limit behavior deterministically. Gate native
   cases and avoid reliance on installed hardware or administrator privileges.
6. [ ] Run formatting, Clippy, tests, and builds locally and in native
   Windows/macOS CI. Document counting rules and unverified edge cases in README
   and architecture docs; record the outcome before choosing the next increment.

Largest-entry views, allocated-size accounting, interactive exploration,
categorization, and driver work remain later roadmap features. Correctness
comes before concurrency or caching; measure before optimizing.
