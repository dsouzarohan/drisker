# Architecture

## Current implementation and boundaries

Drisker is one Rust binary crate with three focused modules:

- `src/main.rs`: clap arguments, working-directory capture, dispatch, and process
  exit handling. Missing top-level arguments show help. `disk scan` requires one
  root and optionally `--json`; an empty root reaches domain validation.
- `src/disk.rs`: request/result types, metadata classification, checked counters,
  error records, and sequential directory traversal. It returns data and never
  prints. Paths remain native `PathBuf` values.
- `src/output.rs`: text/JSON rendering and scan exit-status selection. JSON
  presentation types borrow diagnostic strings and convert native paths into
  display/lossiness objects. Serialization does not affect the domain model.

clap parses arguments; serde and serde_json serialize structured output.
`tempfile` is a development-only dependency for controlled fixtures. There is no
library target, workspace, full tree model, platform trait hierarchy, background
service, or mutation command. Unsafe code remains forbidden.

The [scanner contract](disk-scan-contract.md) defines accounting, boundaries,
limits, error codes, the JSON schema, and exit statuses. Driver inventory and
maintenance remain roadmap features.

## Scanner mechanics

```text
arguments → request with captured working directory
          → root validation → iterative depth-first traversal
          → Result<ScanSummary, ScanError> → text or JSON → exit status
```

`ScanRequest` owns the supplied root and captured working directory. Filesystem
operations use a joined path; diagnostics retain the supplied root. Trailing
separators are removed for root classification, without canonicalizing through
links or changing the reported path. Ancestor links and concurrent replacement
remain outside any containment guarantee.

`symlink_metadata` observes each entry for classification without following the
final link. A small `cfg(windows)` block checks the reparse attribute before file
or directory classification. All reparse tags are skipped. The rest of the walk
uses safe standard-library APIs on both platforms. See the official
[symlink_metadata](https://doc.rust-lang.org/std/fs/fn.symlink_metadata.html),
[Windows MetadataExt](https://doc.rust-lang.org/std/os/windows/fs/trait.MetadataExt.html),
and [file attribute constants](https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants)
references. Path checks and subsequent opens are separate observations; they do
not prevent filesystem races.

The walk owns a `Vec<Frame<I>>`. Each frame holds a directory iterator, its paths,
and its depth. A discovered directory pushes a frame; an exhausted iterator pops
one and releases its handle. There are at most 257 open directory iterators
(root plus depths 1–256), independent of directory width. Retained path lengths
still affect memory usage. There is no recursion on Rust's call stack, sorted
entry buffer, or queue of all pending paths. Ordinary mounts are crossed; any
cycles not excluded by link policy eventually hit the explicit depth limit.

`scan_with` accepts two private function parameters for metadata and enumeration.
Production uses filesystem calls; tests inject controlled results into the same
loop to exercise permission failures, disappearance, iterator errors, overflow,
and depth limits. This is a concrete test seam, not a public filesystem backend
abstraction. Its iterator is generic and owned; `use<>` on `read_directory`'s
return type specifies that the iterator does not borrow the input path.

Every total and error counter uses checked u64 addition. Recoverable errors are
counted and the first 100 retained, while traversal continues. Root failures and
counter overflow return `Err`, discarding the local summary. `Ok` contains either
complete or partial observations, distinguished by its error count. Policy skips
are successful observations, not errors. Totals count logical bytes per regular
file name and never imply allocated usage or deletion safety.

## Rendering and output failures

Both output modes borrow the same domain result. Text shows exact bytes, counts,
completion, and limitations on stdout, with retained diagnostics on stderr.
JSON contains one versioned envelope with policy, nullable totals, recoverable
errors, and a nullable fatal error. Paths carry a display string and `lossy`
flag. Control characters are escaped in text and JSON; OS error messages are
bounded by Unicode scalar count. JSON stdout has no separate progress or prompts.

The renderer uses fallible writers and flushes both streams. A write/flush error
propagates to the entry point, which exits 1. Output may already be truncated;
consumers must require valid JSON and inspect the exit status. Scan exits are
0 (complete), 1 (fatal), and 3 (partial); clap syntax errors use 2.

## Verification strategy and limits

Parser tests validate the CLI shape. Binary integration tests cover help,
version, invalid arguments, scanner output/streams, root failures, relative
paths, and a Unix depth-limit partial scan. Domain tests use empty/nested
temporary trees, known byte totals, hard links, spaces/Unicode, invalid roots,
and injected failures/limits. Renderer tests compare both example outputs with
the contract and exercise lossy paths, control characters, large integers,
partial/fatal fields, and broken writers/flushes.

Unix tests cover file/directory/dangling/cyclic links and a socket. Native Windows
has a junction fixture that verifies reparse classification, skipped traversal,
and root rejection, without requiring symlink privileges or administrator access.
Synthetic tests run on both platforms. CI runs formatting, Clippy with warnings
denied, tests, and builds on stable Rust with native Windows x86_64 and macOS
ARM64 runners. See PLAN.md for the actual validation evidence.

A junction fixture is not exhaustive validation of reparse tags, cloud files,
network providers, mounted volumes, or concurrent path replacement. Live reads
are not snapshots. Local sandbox restrictions can prevent Unix socket fixture
creation; a normal unsandboxed test run is required for that fixture. Linux
support, graceful cancellation, allocated-size accounting, identity-based
hard-link deduplication, and performance targets remain deferred. Profile before
introducing parallelism or caching.

## Safety and future boundaries

Inspection only enumerates names and reads metadata; it does not open file
contents. No mutating feature exists. OS/provider side effects of metadata reads
are outside that read-only guarantee. Windows behavior requires native tests;
a macOS build or cross-compilation cannot establish it.

Future driver inspection will keep inventory, discovery, compatibility, and
trust verification distinct, with supported OS APIs and source evidence. Add
modules and platform interfaces only when implementing concrete responsibilities.
Future mutation requires a separate design and explicit authorization:
discover, inspect, explain, verify, propose, authorize, execute, and report.
Unavailable compatibility is not verified compatibility, and storage totals do
not establish deletion safety.
