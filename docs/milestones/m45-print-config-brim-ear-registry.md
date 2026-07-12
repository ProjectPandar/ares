# M45: PrintConfig brim ear option registry

## Goal
Port the FFF `brim_ears`, `brim_ears_max_angle`, and `brim_ears_detection_length` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1665-1693` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:925-926` plus the upstream `brim_ears` option definition in `PrintConfig.cpp:1665-1670` and `PrintConfig.cpp:1672-1693`; no new Ares pipeline, crate, brim-ear geometry detection, sharp-angle analysis, brim generation behavior, extrusion behavior, G-code behavior, filesystem, network, UI, preset behavior, or object override behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `brim_ears`, `brim_ears_detection_length`, and `brim_ears_max_angle` with exact defaults and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/tooltip/sidetext/min/max/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Brim-ear sharp-edge detection, detection-radius decimation, max-angle behavior, brim generation, extrusion behavior, and downstream G-code behavior remain deferred.
- Following compatible profile options remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
