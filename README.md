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

The CLI provides:

- `drisker --help` (or `-h`) and `drisker --version` (or `-V`).
- Help when invoked without arguments, and errors for unsupported arguments.
- `drisker disk scan <path>` with text output or `--json`.

Driver discovery and maintenance operations are not implemented. Windows x86_64
is the product target; macOS ARM64 is the development and portable-scanner test
platform. Linux support is deferred.

## Read-only disk scan

```sh
cargo run --locked -- disk scan ./src
cargo run --locked -- disk scan ./src --json
```

Supply exactly one directory. Quote paths containing spaces. Relative roots use
the invocation's working directory; reported paths retain the supplied root.
File roots and root links/reparse points are rejected. No administrator access
is requested.

The scanner counts regular-file names and sums their observed logical lengths.
The root is excluded from directory counts. Hard links count once per name;
sparse/compressed files contribute logical lengths. This does not measure
allocated or reclaimable storage. Links, all Windows reparse points (including
junctions), and special entries are counted as skips and never traversed.
Ordinary mounts are traversed, even across volumes; there is no same-volume
option. These are live observations, with no snapshot guarantee or protection
against concurrent path replacement.

Unreadable or disappearing entries produce partial results while reachable
siblings continue. Directories through depth 256 are enumerated; deeper
ordinary directories are counted but produce a depth-limit error. At most 100
error records are retained, with all errors counted and omissions reported.
Counter overflow is fatal and returns no totals.

| Exit | Meaning |
| --- | --- |
| `0` | Complete under the counting/skip policies, with no observed errors. |
| `1` | Fatal scan or output failure. |
| `2` | Invalid CLI syntax. |
| `3` | Partial scan with observed totals and recoverable errors. |

Text diagnostics go to stderr. JSON stdout contains one versioned object with
policy, totals, and diagnostic fields; fatal scan results have null totals.
Parser/output failures may use stderr. Paths use a display string plus a `lossy`
flag when native names cannot be represented as Unicode. JSON integers preserve
u64 values; consumers must avoid loss of precision above 2^53. Consumers must
check both JSON validity and process exit status. The full schema, examples, and
counting rules live in the [behavior contract](docs/disk-scan-contract.md).

## Direction

The first scanner provides logical totals and explicit limitations. Later work
will address filesystem accounting, links and special files, interactive
exploration, and Windows storage explanations. Driver work will begin with
inventory, then supported Windows update discovery and deterministic package
verification. Discovery will use supported APIs and traceable sources, without
LLMs, crawlers, or speculative device matching.

See [ROADMAP.md](ROADMAP.md) for milestones and [PLAN.md](PLAN.md) for the next
small increment.

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

Local Apple Silicon checks pass with Rust/Cargo 1.99.1: formatting, Clippy
with warnings denied, 24 unit tests, 9 binary integration tests, build, help,
version, and a JSON scan. Native Windows x86_64 and macOS ARM64
[scanner CI](https://github.com/dsouzarohan/drisker/actions/runs/42418864283)
passes formatting, Clippy, tests, and builds at `6158e1a`, including the native
Windows junction fixture. Platform-gated tests make test counts differ by OS.
`Cargo.lock` is committed and CI uses `--locked` to prevent unintended dependency
changes.

Tests use controlled temporary directories and deterministic metadata/enumeration
failures. They cover counting, hard links, partial results, limits, overflow,
output streams, JSON, and root rejection. Unix fixtures cover links/cycles and a
socket; the Windows fixture creates a directory junction without elevation and
checks reparse classification and root rejection. This does not validate every
reparse tag, cloud provider, network filesystem, or mounted-volume scenario.
No live hardware inventory or system-directory mutation is needed. The local
sandbox blocks binding the temporary Unix socket; that test requires a normal
unsandboxed test run, not administrator access.

### macOS and Windows workflow

Edit and run portable checks on the Mac Mini. For additional Windows API work,
use a checkout of the same revision on the Windows PC and run native
builds and tests over SSH while that machine is booted into Windows. Configure
SSH and Rust on that machine separately; no remote execution framework is
included. A macOS build or a Windows cross-build cannot validate native Windows
behavior. GitHub Actions supplies an additional native check on each platform.

## Repository guide

```text
Cargo.toml                         Single binary crate; clap, serde, and serde_json
Cargo.lock                         Resolved dependency versions for reproducible builds
LICENSE                            MIT license
src/main.rs                        CLI parsing, dispatch, and exit handling
src/disk.rs                        Scan model, accounting, and traversal
src/disk/tests.rs                  Scanner fixtures and injected failure tests
src/output.rs                      Text and JSON rendering
src/output/tests.rs                Output-contract and write-failure tests
tests/cli.rs                       Foundation CLI integration tests
tests/disk_scan.rs                 Scanner binary integration tests
.github/workflows/ci.yml            Stable Rust checks on Windows and macOS
PLAN.md                            Immediate increment and verification status
ROADMAP.md                         Feature-level milestones
AGENTS.md                          Agent workflow and session continuity
docs/architecture.md               Boundaries, data flow, and testing strategy
docs/disk-scan-contract.md          Scanner behavior and output contract
docs/adr/0001-initial-architecture.md
                                   Initial architectural decision
```

## Safety

Read-only inspection is the default. No system mutation capabilities exist.
An available update does not establish that it should be
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

Drisker is licensed under the [MIT license](LICENSE).
