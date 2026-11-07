# Immediate plan

## Bootstrap handoff

The source foundation includes the single binary crate, clap CLI, unit and
integration tests, documentation, and native Windows/macOS CI configuration.
It contains no scanner, driver discovery, or mutation operations.

Git is initialized on `main`. Rust/Cargo 1.99.1 is installed with the native
`stable-aarch64-apple-darwin` toolchain, rustfmt, and Clippy. Cargo.lock is
generated and CI uses locked dependencies. Formatting, Clippy with warnings
denied, 3 unit tests, 4 integration tests, build, help, and version all pass
locally. Hosted CI and native Windows execution have not been verified.
The public repository is `dsouzarohan/drisker`, linked as `origin`. The initial
commit establishes the foundation; subsequent changes should use pull requests.
A license remains to be chosen before the project can be distributed as
open-source software.

## Next increment: validate the foundation

Complete these tasks before starting feature work. This plan deliberately stays
within the bootstrap scope.

1. [x] Install and verify stable Rust/Cargo, rustfmt, and Clippy on the Mac Mini;
   confirm the native ARM64 host toolchain and pass the formatting check.
2. [x] Initialize Git, generate `Cargo.lock`, and add `--locked` to CI's Clippy,
   test, and build commands. Include the lockfile in the eventual initial commit.
3. [x] Run `cargo fmt --all -- --check` and
   `cargo clippy --all-targets --locked -- -D warnings`; resolve any findings.
4. [x] Run `cargo test --locked`, `cargo build --locked`, and manually inspect
   `cargo run --locked -- --help` and `cargo run --locked -- --version`.
5. [ ] Push to the hosted repository and observe successful native Windows
   x86_64 and macOS ARM64 CI jobs; record actual results and fix failures.
6. [ ] Select and add the project license, update the verification notes in
   this plan and README, and review the first read-only scanner increment with
   the developer. Implementation requires an explicit follow-up request.
