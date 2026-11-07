# ADR 0001: Rust CLI with a Windows-first, read-only foundation

**Status:** Accepted

## Context

Drisker should help users understand storage, filesystems, hardware, and
drivers before considering maintenance. Windows is the first product target;
development primarily happens on macOS, with native Windows testing on a
separate PC and in CI. The project is also a practical Rust and systems
learning effort. It needs room to grow without making its starting structure
difficult to understand.

## Decision

- Use Rust for explicit ownership, strong domain modeling, native integration,
  and performance without requiring a managed runtime. Prefer safe code and
  deliberate error handling; isolate any necessary unsafe system bindings.
- Build a CLI first, using clap for argument parsing. Keep future inspection
  logic separate from presentation so text, JSON, and interaction can share
  results. No GUI is planned.
- Support Windows first while keeping portable domain and filesystem logic
  usable on macOS. Compile and test on both platforms, but establish feature
  support through native behavior tests. Add Windows-specific modules and
  conditional compilation when concrete integrations require them.
- Default to read-only inspection and explanation. Future mutation needs
  separate commands, verification, concrete proposals, and explicit user
  authorization. Discovery alone never authorizes a change.
- Start as one binary crate with logical modules introduced as code requires.
  Do not create a workspace, speculative OS backends, or a plugin system.
- Use supported Windows APIs and deterministic sources for driver discovery.
  Keep candidate discovery, applicability, trust, and installation distinct.

## Alternatives considered

- **A managed-language implementation:** Could speed some scripting and UI
  tasks, but does not serve the Rust learning goal or the desired native
  systems implementation as directly.
- **GUI-first or background service:** Adds lifecycle, deployment, and
  interaction complexity before inspection behavior is understood.
- **Windows-only code throughout:** Simplifies some immediate API calls but
  couples domain logic to the OS and hinders macOS development checks.
- **Full cross-platform support immediately:** Expands metadata and testing
  obligations before Windows behavior is established.
- **A multi-crate workspace or general platform abstraction:** Introduces
  boundaries without demonstrated reuse or concrete alternative backends.
- **Automatic optimization or opaque update matching:** Conflicts with
  explainability, deterministic verification, and user-controlled maintenance.

## Consequences

The foundation stays small and can be developed on macOS. Rust and Windows APIs
still require learning and careful native testing; macOS success says nothing
about Windows API correctness. Portable interfaces will emerge from real
features rather than anticipated implementations.

Read-only inspection must remain independently useful. Maintenance will need
additional architectural decisions and recovery/failure analysis. A future
workspace, library target, or shared platform interface is possible when actual
complexity justifies it. This decision does not specify scan algorithms, JSON
contracts, Windows API choices, or release automation.
