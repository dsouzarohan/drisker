# Architecture

## Current implementation and boundaries

Drisker is one Rust binary crate. `src/main.rs` defines a clap parser and prints
help when invoked without arguments. clap handles help/version flags and
invalid arguments. Version and description come from Cargo package metadata.
Output failures when printing default help propagate as `io::Result` rather
than panicking. There is no application domain implementation yet.

clap is the only direct dependency. The standard library supplies process-based
integration tests. There are no empty domain modules, platform implementations,
plugin infrastructure, databases, background services, or mutation commands.
The crate currently forbids unsafe code; any future Windows integration that
requires it must deliberately revisit that restriction and isolate documented
unsafe boundaries.

## Responsibilities as features arrive

Add modules only when implementing their responsibilities:

- **CLI/presentation:** Parse arguments, invoke operations, select text or JSON
  rendering, manage diagnostics and exit status, and handle interaction.
- **Disk inspection:** Traverse filesystems, interpret metadata, aggregate
  sizes, and report results plus limitations. Implement basic traversal and
  aggregation ourselves for clarity and learning, using portable APIs first.
- **Driver inspection:** Represent devices, installed drivers, update
  candidates, provenance, and verification evidence. Start with inventory and
  supported Windows APIs; keep discovery distinct from compatibility and trust
  verification.
- **Platform integration:** Isolate Windows APIs and metadata behind small
  interfaces and `cfg(windows)` when needed. Keep platform-independent models
  and calculations free of Windows handles and bindings. Introduce traits only
  for a demonstrated need, not hypothetical OS backends.

No library crate or additional packages are needed now. When application logic
appears, move it out of the CLI entry point into focused modules. A library
target can be added later if consumers or integration tests need one.

## Conceptual inspection data flow

```text
arguments → validated request → filesystem / supported OS APIs
          → observations and explicit failures → analysis / verification
          → result model with evidence and limitations → text or JSON
```

Application logic should return data and errors rather than printing. Future
structured output must contain no styling, progress indicators, or prompts.
Keep diagnostics and progress off JSON stdout. Establish JSON schemas, partial
result behavior, and exit-code policy when the first inspection command lands;
they are not specified by this skeleton.

Disk totals must identify their accounting policy: logical versus allocated
size, handling of links and volume boundaries, inaccessible entries, and files
changing during observation. An incomplete scan must not silently present a
total as complete. Driver candidates must retain source and applicability
evidence; unknown compatibility is not verified compatibility.

## Safety boundary

Inspection is read-only and useful independently of maintenance. Storage usage
does not establish deletion safety; update availability does not establish
installation suitability. No mutating feature exists in the bootstrap.

Future mutation commands require a separate design and explicit user command.
Their flow is discover, inspect, explain, verify, present a concrete proposal,
obtain authorization, execute, and report. Do not combine observation with
automatic cleanup or installation. Privileged and unsafe integrations must be
small, documented, and reviewable; custom cryptography is excluded.

## Verification strategy

The built-in Rust test framework is sufficient initially. Parser unit tests
validate the clap definition and argument policy. Integration tests invoke the
compiled binary and verify help/version, no-argument behavior, invalid input,
stdout/stderr, and exit status without depending on exact help formatting.

Future algorithm tests should use controlled temporary directory trees: empty
and nested directories, regular files, spaces and Unicode in paths, links,
permission failures, and changes during scans. Gate platform-specific cases;
permission tests must account for privilege differences. Use fixtures or small
adapters for device/update data rather than relying on installed hardware,
live update catalogs, administrator access, or system mutations.

CI runs formatting, Clippy with warnings denied, unit and integration tests,
and builds on stable Rust using native Windows x86_64 and macOS ARM64 runners.
Native Windows API behavior needs native Windows tests. Local macOS checks and
cross-compilation are useful but cannot substitute for them. Remote Windows
SSH tooling is deferred until integration work requires it.

## Current limitations and undecided details

No scan model, tree representation, Windows bindings, error taxonomy, JSON
schema, cancellation design, or performance target is implemented. Choose them
with the first concrete requirements. Prioritize correctness; use profiling
and benchmarks before parallelism or caching. No minimum Rust version beyond
using current stable is promised. The license remains undecided. Local checks
pass with locked dependencies; native Windows and hosted CI verification
remain pending in PLAN.md.
