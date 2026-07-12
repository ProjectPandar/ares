# M1: Basic slicer framework skeleton

## Goal
Establish the minimal Rust workspace and documentation foundation for a platform-neutral slicer core with a CLI adapter.

## Exit checklist
- `Cargo.toml` declares a workspace with `crates/ares-core` and `crates/ares-cli` as members.
- `crates/ares-core` exposes the first slicing API without direct filesystem access.
- `crates/ares-core` accepts model bytes and dynamic Orca-compatible options; M1 format labels are detected from bytes.
- `crates/ares-core` preserves every provided option key and value, including unknown OrcaSlicer options.
- `crates/ares-core` returns generated G-code bytes and typed errors suitable for native and WASM callers.
- `crates/ares-cli` owns command-line parsing, input file reads, output file writes, and exit behavior.
- `crates/ares-cli` calls `ares-core` instead of duplicating slicing logic.
- Tests cover dynamic option preservation and the CLI-to-core boundary.
- `AGENTS.md` documents the actual crate locations under `crates/` and states that future crates are roadmap-driven.
- `docs/architecture/ard-0001-basic-slicer-crate-boundaries.md` records the two-crate boundary, file-I/O rule, dynamic option decision, deferred splits, and OrcaSlicer evidence.
- `docs/roadmap.md` lists milestones M1 through M7 with clear sequencing.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No additional crates beyond `crates/ares-core` and `crates/ares-cli`.
- No full model parser, slicing algorithm, G-code parity engine, or WASM binding in M1.
- No narrowing of OrcaSlicer options to only the typed fields currently understood by Ares.
