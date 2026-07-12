# M118: PrintConfig support filament registry

## Goal
Port the support/raft base filament option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6027-6034` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:959`, `PrintConfig.cpp:6027-6034`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, support filament routing behavior, raft filament routing behavior, support material selection behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_filament` with exact kind, default, and source line range.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for support/raft filament routing, support material selection, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `support_interface_not_for_body`, `support_line_width`, and following support options from `PrintConfig.cpp:6036+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
