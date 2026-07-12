# M180: PrintConfig legacy obsolete-key ignore list

## Goal
Port the obsolete-key ignore branch from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8069-8091` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:8069-8091` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- Every obsolete key listed in `PrintConfig.cpp:8070-8086` is dropped during `SliceOptions` deserialization.
- Dropping is key-based only; obsolete-key values of any JSON type are ignored.
- Non-obsolete unknown keys remain preserved because upstream final unknown-key validation from `PrintConfig.cpp:8093-8096` is not part of this milestone.
- Existing legacy migrations from M169-M179 continue to run before the obsolete-key ignore check.
- Later `handle_legacy` behavior from `PrintConfig.cpp:8093+`, including final unknown-key validation and composite thumbnail handling, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
