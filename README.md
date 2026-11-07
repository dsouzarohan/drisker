# Drisker

Drisker is a Windows-first, cross-platform-minded system inspection and
maintenance CLI written in Rust. It aims to help people understand disk usage,
filesystems, hardware devices, and drivers before deciding whether to change
anything.

Repository: [dsouzarohan/drisker](https://github.com/dsouzarohan/drisker).

Inspection and explanation are useful outcomes on their own. Drisker favors
transparent, deterministic results, correctness, and clear limitations over
opaque recommendations or performance optimization claims.

## Current status

This is the repository foundation, not a working system inspector. The CLI
currently provides only:

- `drisker --help` (or `-h`).
- `drisker --version` (or `-V`).
- Help when invoked without arguments, and errors for unsupported arguments.

Disk scanning, driver discovery, JSON output, and maintenance operations are
not implemented. Proposed future commands are not yet CLI contracts.

Windows x86_64 is the initial product target. macOS ARM64 is the primary
development platform and a compilation/test target; that does not establish
macOS feature support. CI is configured for native Windows and macOS runners.
Its results must be checked in GitHub Actions before claiming either passes.
Linux support is deferred.

## Direction

The first feature increment will be a small, read-only disk scanner. Later work
will address filesystem accounting, links and special files, interactive
exploration, and Windows storage explanations. Driver work will begin with
inventory, then supported Windows update discovery and deterministic package
verification. Discovery will use supported APIs and traceable sources, without
LLMs, crawlers, or speculative device matching.

See [ROADMAP.md](ROADMAP.md) for milestones and [PLAN.md](PLAN.md) for the next
small increment and outstanding bootstrap checks.

## Development

Install the current stable Rust toolchain using [rustup](https://rustup.rs/),
including Cargo, rustfmt, and Clippy. Edition 2024 requires Rust 1.85 or later,
but dependencies may require newer versions; no minimum supported Rust version
is promised yet. On Windows, use the MSVC toolchain with Visual Studio Build
Tools and the Windows SDK. On macOS, install Xcode Command Line Tools.

From the repository root:

```sh
rustup component add rustfmt clippy
cargo build --locked
cargo run --locked -- --help
cargo run --locked -- --version
cargo test --locked
```

Run the same checks used by CI before submitting changes:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked
```

Bootstrap verification passes on native Apple Silicon with Rust/Cargo 1.99.1:
formatting, Clippy with warnings denied, 3 unit tests, 4 integration tests,
build, help, and version. Hosted CI and native Windows execution remain
unverified. `Cargo.lock` is included for this binary application; retain it in
version control. CI uses `--locked` to prevent unintended dependency changes.

Unit tests check the CLI definition and parsing. Integration tests execute the
binary and check output streams, exit status, help, version, and rejection of
unimplemented commands. They require neither elevated privileges nor access to
installed hardware. Future filesystem tests will use controlled temporary
fixtures.

### macOS and Windows workflow

Edit and run portable checks on the Mac Mini. When Windows API integration
begins, use a checkout of the same revision on the Windows PC and run native
builds and tests over SSH while that machine is booted into Windows. Configure
SSH and Rust on that machine separately; no remote execution framework is
included. A macOS build or a Windows cross-build cannot validate native Windows
behavior. GitHub Actions supplies an additional native check on each platform.

## Repository guide

```text
Cargo.toml                         Single binary crate; clap is the only direct dependency
Cargo.lock                         Resolved dependency versions for reproducible builds
src/main.rs                        CLI entry point and parser unit tests
tests/cli.rs                       Binary integration tests
.github/workflows/ci.yml            Stable Rust checks on Windows and macOS
PLAN.md                            Immediate increment and verification status
ROADMAP.md                         Feature-level milestones
docs/architecture.md               Boundaries, data flow, and testing strategy
docs/adr/0001-initial-architecture.md
                                   Initial architectural decision
```

## Safety

Read-only inspection is the default. No system mutation capabilities exist in
this foundation. An available update does not establish that it should be
installed; occupied storage does not establish that it is safe to delete.

Future maintenance must discover, inspect, explain, verify, present a concrete
proposal, obtain explicit authorization, execute, and report the result.
Automatic cleanup, automatic driver installation, background services, registry
cleaners, firmware updates, and optimization claims are outside the initial
scope.

## Contributing and following development

Start with the plan, architecture, and ADR. Keep changes small and explain their
behavior, trade-offs, and verification. Prefer idiomatic Rust and explicit error
handling, with dependencies introduced for concrete needs. Discuss significant
architectural changes before implementing them, and record accepted decisions
as focused ADRs. Use issues to track development and pull requests for changes
to `main`.

This is also a Rust and systems learning project: explain meaningful ownership,
data-model, and OS API choices without adding abstractions solely for teaching.

CLI metadata follows the [clap derive documentation](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html).
The CI runner selection follows [GitHub's native runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

A project license has not been selected yet. Open-source distribution requires
that decision and a corresponding license file before public release.
