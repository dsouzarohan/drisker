# Roadmap

This is a revisable feature backlog, not a release schedule. Milestones describe
intended outcomes, not implemented functionality. Only the CLI source
foundation exists today; local checks pass, while native Windows and hosted CI
verification remain pending (see PLAN.md).

## 1. Repository foundation and developer experience

- **Goal:** An understandable, reproducible starting point.
- **Capabilities:** Minimal CLI, documentation, unit/integration testing,
  stable Rust formatting/lint/build checks, native Windows and macOS CI.
- **Outcome:** Contributors can build, test, and understand the project.
- **Dependencies/questions:** Observe hosted CI, retain Cargo.lock in version
  control, choose a license before public release; establish native Windows access when
  integration work begins.

## 2. Basic read-only disk scanning

- **Goal:** Explain logical file size under a supplied directory.
- **Capabilities:** Portable traversal and aggregation, explicit link policy,
  inaccessible-path reporting, useful text output and structured JSON.
- **Outcome:** A small, correct scanner with controlled fixture coverage.
- **Dependencies/questions:** Define counting rules, partial-result semantics,
  filesystem changes during scans, and safe traversal boundaries before coding.
  Benchmark before adding parallelism.

## 3. Filesystem metadata and accounting correctness

- **Goal:** Make storage totals and their limitations explainable.
- **Capabilities:** Volume discovery, logical versus allocated size, hard-link
  accounting, symbolic links, junctions, reparse points, mount boundaries,
  permissions, and relevant NTFS metadata.
- **Outcome:** Users can understand why totals differ from OS disk usage.
- **Dependencies/questions:** Native Windows API experiments and fixtures;
  define deduplication, sparse/compressed-file handling, and unavailable metadata.

## 4. Interactive disk exploration

- **Goal:** Help users navigate where space is consumed.
- **Capabilities:** Largest-file/directory views and interactive navigation
  built on scan results, with usable cancellation and progress reporting.
- **Outcome:** Practical exploration alongside scriptable inspection.
- **Dependencies/questions:** Measure large-tree memory and traversal costs;
  choose terminal libraries when interaction requirements become concrete.

## 5. Windows storage diagnostics and explanation

- **Goal:** Explain common Windows and application storage consumers.
- **Capabilities:** Evidence-based categorization of application data, caches,
  system storage, and potentially reclaimable space, with provenance and limits.
- **Outcome:** Users understand storage ownership and maintenance options.
- **Dependencies/questions:** Maintain deterministic classification rules;
  distinguish uncertain ownership from verified information. A category or
  reclaimable estimate must never imply permission to delete.

## 6. Windows hardware and driver inventory

- **Goal:** Describe installed devices and their drivers.
- **Capabilities:** Device enumeration, hardware/compatible IDs, provider,
  version, date, and INF metadata through supported Windows APIs.
- **Outcome:** Explainable inventory in text and machine-readable form.
- **Dependencies/questions:** Select supported APIs, document privilege and
  OS-version requirements, and establish fixtures plus native Windows tests.

## 7. Driver update discovery

- **Goal:** Identify candidate updates from deterministic sources.
- **Capabilities:** Windows Update discovery, device correlation, applicability
  evidence, and installed/candidate comparisons with source provenance.
- **Outcome:** Users can inspect why a candidate is applicable and its limits.
- **Dependencies/questions:** Supported update APIs, network/offline behavior,
  metadata availability, and compatibility rules. An available update is not
  an installation recommendation. No AI, crawling, or speculative matching.

## 8. Driver package verification

- **Goal:** Establish verifiable evidence before proposing installation.
- **Capabilities:** Package signature verification, hardware/OS compatibility
  checks, and explicit reporting of failed or inconclusive verification.
- **Outcome:** Auditable candidate verification with understandable risks.
- **Dependencies/questions:** Use established platform cryptography; determine
  trust-policy, revocation/offline behavior, package access, and test fixtures.

## 9. Explicitly authorized maintenance

- **Goal:** Perform narrowly scoped changes only after informed authorization.
- **Capabilities:** Reviewed cleanup proposals and driver installation, backup,
  and recovery where supported; verification and result reporting.
- **Outcome:** A discover → inspect → explain → verify → propose → authorize
  → execute → report workflow with separate mutation commands.
- **Dependencies/questions:** Separate ADRs and threat/failure analysis,
  privileges, stale proposals, recovery limitations, and robust native testing.
  Automatic cleanup or installation remains outside the initial scope.

## 10. Cross-platform expansion

- **Goal:** Extend useful inspection to macOS and Linux based on actual demand.
- **Capabilities:** Portable scan behavior plus native metadata integrations
  and explicit platform capability reporting.
- **Outcome:** Supported features documented and tested per OS.
- **Dependencies/questions:** Stabilize Windows behavior first; choose real
  integrations before adding platform abstractions. Compilation alone does not
  establish feature support.
