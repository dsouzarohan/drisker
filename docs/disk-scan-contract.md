# First disk scan: behavior contract

**Status:** Implemented in the first read-only scanner. This
contract covers the disk-scanning milestone in [ROADMAP.md](../ROADMAP.md).
Changes to these rules must update this document and the corresponding tests.

## Request and scope

The interface is `drisker disk scan <path> [--json]`. Exactly one explicit
directory root is required; there is no implicit current-directory scan. Relative
paths are resolved against the working directory captured at invocation. Accept
ordinary local directories, volume roots, and reachable network directories,
subject to OS permissions. Do not request elevation or credentials. Shell quoting
and expansion happen before Drisker receives the path.

Preserve the supplied path for reporting; do not canonicalize it into a different
reported root. Reject an empty path, a missing root, a regular-file root, and a
root whose final component is a symbolic link or Windows reparse point. The root
must have readable classification metadata and its initial directory enumeration
must open successfully. Failure at this stage is fatal, with no totals.

Ancestors of the supplied root may resolve through links or mounts. This is a
filesystem inspection tool, not a sandbox enforcing physical containment. Do not
promise that path-based checks prevent concurrent link replacement. Native
Windows metadata classification must be verified before traversal is shipped.

Only enumerate names and read metadata. Do not read file contents, hydrate files
deliberately, write files, change permissions, delete anything, or recommend
deletion. OS/filesystem side effects of reads (such as access-time changes) are
outside this read-only guarantee. Offline/reparse entries are skipped as below;
metadata access itself may involve the filesystem or network provider.

## Accounting and boundaries

The root is not included in entry counts. Every descendant for which metadata
classification succeeds contributes to exactly one count:

| Field | Meaning |
| --- | --- |
| `files` | Regular-file entries; add their observed metadata length to `logical_bytes`. |
| `directories` | Ordinary directories discovered, including ones that cannot subsequently be enumerated. |
| `skipped_links` | Symbolic links on any platform and all Windows reparse points, regardless of target or tag. |
| `skipped_special` | Other classified entries, such as sockets, devices, or FIFOs. |

An entry with failed metadata contributes to none of these counts and produces
an error. An enumeration error may conceal an unknown number of entries; do not
invent counts for them. There is no aggregate `entries` field that implies these
unknown entries were counted. Links/reparse points take precedence over file or
directory classification; they contribute zero bytes and are never traversed.
Special entries also contribute zero bytes and are never opened.

`logical_bytes` is the sum of observed regular-file lengths, in bytes. Exclude
directory metadata, link lengths, allocation units, alternate data streams,
extended attributes, and other filesystem overhead. Sparse and compressed files
contribute their logical lengths. Count hard links once per encountered name;
there is no file-identity deduplication. Two names for the same 10-byte file
contribute two files and 20 bytes. These totals are neither allocated usage nor
an estimate of reclaimable storage.

There is no same-volume restriction or volume discovery in this increment.
Traverse ordinary mounted directories even when they cross a filesystem boundary.
Windows junctions and volume mount points marked as reparse points are skipped
under the same policy as other reparse points. Thus volume behavior differs by
platform; the output must state this policy rather than claim to stay on one
disk. No flag to follow links or change boundary policy is provided initially.

## Observation, failures, and limits

Traverse sequentially. Enumeration order is unspecified; do not sort entire
directories or retain a full tree. Observe each discovered entry's metadata once
for accounting. Concurrent changes can cause missing, duplicated, or stale
observations; there is no snapshot, retry, or consistency guarantee. A detected
disappearance, access failure, or enumeration failure is reported. Undetected
changes do not automatically mark a result partial. Totals need not be a lower
bound on the filesystem's current contents.

After the root enumeration opens, metadata and enumeration failures are
recoverable: retain observations, record errors, and continue with reachable
siblings. On an iterator error, retain earlier entries and continue if the
iterator permits; never restart the directory to retry. A directory whose open
fails still counts as a directory, but its unknown descendants contribute nothing.

The root has depth 0. Enumerate directories through depth 256 inclusive. A
discovered ordinary directory at depth 257 is counted, but is not opened; record
one `depth_limit` error for it. Non-directory entries discovered at depth 257
are classified and counted normally. This bounds descent even if mount aliases
or concurrent changes create cycles. Do not silently truncate at any other
depth, handle, or queue limit; recoverable resource failures are errors.

Retain at most 100 error records, selecting the first encountered. Continue the
scan after reaching this diagnostic cap. `error_count` counts all errors, not
only retained records; `errors_omitted = error_count - errors.len()`. Store at
most 1,024 Unicode scalar values of each OS error message, adding a truncation
flag. Path fields are not truncated; there is no fixed total output-byte limit.
The cap bounds diagnostic record count, not total scan time or memory usage.

All byte, entry, and error counters are unsigned 64-bit integers with checked
addition. Any overflow aborts with fatal `counter_overflow`; do not wrap,
saturate, or return misleading totals. No configurable limits, progress UI, or
graceful cancellation are promised initially. External termination may leave no
result or incomplete output; consumers must require a complete document and
inspect the process status.

## Completion and exit statuses

| Exit | Meaning | Result |
| --- | --- | --- |
| `0` | Enumeration finished with no observed errors. | `status: "complete"` and totals. |
| `1` | Fatal root, aggregation, or output failure. | Fatal envelope when output is still possible; no totals. |
| `2` | Invalid CLI syntax, handled by clap. | Usage diagnostic on stderr; no scan document. |
| `3` | Enumeration finished with one or more recoverable errors. | `status: "partial"` and observed totals. |

Policy skips (`skipped_links` and `skipped_special`) do not make a scan partial.
“Complete” means complete under these policies with no observed errors, not a
snapshot of every byte on a disk. Output failure takes precedence over a scan's
otherwise successful or partial exit status. A write failure may leave truncated
stdout; no replacement envelope or second output is guaranteed.

## Text and JSON output

Both renderers consume the same result. Text stdout shows root, completion,
logical bytes as an exact integer, all four counts, and the accounting/boundary
limitations. For partial results it also shows total and omitted error counts;
retained diagnostics go to stderr. Fatal text errors go to stderr with empty
stdout. Human-readable unit conversions may supplement the exact bytes later.

With `--json`, stdout contains exactly one JSON object followed by a newline,
with no progress, styling, or prompts. Scan diagnostics are embedded in that
object, not duplicated on stderr. Parser and output-write errors may still use
stderr. JSON uses these fields:

| Field | Type and meaning |
| --- | --- |
| `schema_version` | Integer `1`; changes to field meaning or incompatible shape require a new version. |
| `root` | Path object for the original argument. |
| `status` | `complete`, `partial`, or `failed`. |
| `policy` | Fixed object shown below, included even for fatal scan results. |
| `totals` | Object with `logical_bytes`, `files`, `directories`, `skipped_links`, `skipped_special`; null on fatal failure. |
| `error_count` | Total recoverable errors; null on fatal failure. |
| `errors_omitted` | Recoverable errors not retained; null on fatal failure. |
| `errors` | Array of retained recoverable error records; empty on fatal failure. |
| `fatal_error` | Error record on fatal failure; otherwise null. |

Path objects contain `display` (a string) and `lossy` (a boolean). Use the
platform path's lossy Unicode conversion; set `lossy` when replacement was
needed. Diagnostic paths use the supplied root joined with encountered names.
Do not silently present these strings as round-trippable native paths. Text
paths/messages escape control characters, including newlines and terminal
escapes; JSON uses normal JSON escaping. JSON counters are exact integers;
consumers using floating-point numbers must handle values above 2^53 carefully.

Error records contain `path` (path object), `operation` (`validate_root`,
`read_metadata`, `read_directory`, or `aggregate`), `code`, `os_code` (integer or
null), `message` (string), and `message_truncated` (boolean). Stable codes are
`invalid_root`, `root_link`, `not_found`, `permission_denied`, `io_error`,
`depth_limit`, and `counter_overflow`. Use `invalid_root` for an empty or
non-directory root, `root_link` for a rejected root link/reparse point,
`not_found` and `permission_denied` for those OS error categories, and `io_error`
for other OS failures. Preserve the OS code when available; message wording is
not stable. For overflow, use the path associated with the failed addition.
Fatal results discard prior totals/diagnostics; they do not masquerade as usable
partial results. Error-array order follows observation order and is not stable.

## Small examples

For a root `sample` containing a 4-byte file, a subdirectory with a 6-byte file,
and one symbolic link, a complete text result is:

```text
Root: sample
Status: complete
Logical bytes: 10
Files: 2
Directories: 1
Skipped links/reparse points: 1
Skipped special entries: 0
Accounting: logical bytes per file name; hard links counted per name.
Boundaries: links/reparse points skipped; ordinary mounts traversed.
Limits: live observations, not a snapshot or allocated/reclaimable storage.
```

The same scan with `--json` exits 0 and emits:

```json
{
  "schema_version": 1,
  "root": {"display": "sample", "lossy": false},
  "status": "complete",
  "policy": {
    "size": "logical",
    "hard_links": "per_name",
    "links_and_reparse_points": "skip",
    "special_entries": "skip",
    "volume_boundaries": "cross_ordinary_mounts",
    "consistency": "live_observations",
    "max_directory_depth": 256,
    "max_error_records": 100,
    "max_error_message_chars": 1024
  },
  "totals": {
    "logical_bytes": 10,
    "files": 2,
    "directories": 1,
    "skipped_links": 1,
    "skipped_special": 0
  },
  "error_count": 0,
  "errors_omitted": 0,
  "errors": [],
  "fatal_error": null
}
```

If that subdirectory's enumeration instead fails with permission denied, retain
4 logical bytes, 1 file, 1 directory, and 1 skipped link. Set `status` to
`partial`, `error_count` to 1, and `errors_omitted` to 0; include a
`read_directory` / `permission_denied` record for `sample/subdirectory`.
Exit 3. Text labels the totals partial and prints the retained error on stderr.

A missing root exits 1: JSON has `status: "failed"`, null totals and error
counts, an empty `errors` array, and a `fatal_error` with operation
`validate_root` and code `not_found`. A root link uses `root_link` instead.
At 103 recoverable errors, return partial totals, retain 100 error records, and
report `error_count: 103` and `errors_omitted: 3`.

## Implementation acceptance

Controlled fixtures cover the accounting example, empty roots, root rejection,
hard links, policy skips, inaccessible/disappearing entries, iteration failures,
depth boundaries, 103 errors, and checked-counter overflow. Errors are exercised
deterministically without requiring administrator access or assuming permission
denials work under every account. A native Windows junction fixture checks
reparse classification; macOS checks alone cannot establish that behavior.
Tests cover both output modes,
stream separation, control-character escaping, lossy paths, and exit statuses.
See README.md and PLAN.md for validation evidence and remaining platform limits.
