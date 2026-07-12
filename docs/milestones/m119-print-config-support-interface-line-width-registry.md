# M119: PrintConfig support interface base avoidance and line width registry

## Goal
Port the adjacent support interface base-avoidance and support line-width option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6036-6053` into `ares-core` registry metadata, with a mechanical registry shard split to keep Rust files below 400 LOC.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:960-961`, `PrintConfig.cpp:6036-6053`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, interface-filament routing behavior, support line-width resolution behavior, nozzle-diameter ratio behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_interface_not_for_body` and `support_line_width` with exact kinds, defaults, and source line ranges.
- The existing `tail_terminal.rs` support block is mechanically split into sorted shards without changing existing metadata.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for support interface filament routing, line-width resolution, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `support_interface_loop_pattern` and following support options from `PrintConfig.cpp:6055+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
