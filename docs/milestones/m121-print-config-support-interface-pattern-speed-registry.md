# M121: PrintConfig support bottom interface spacing, interface speed, and patterns registry

## Goal
Port the adjacent support bottom-interface spacing, support-interface speed, support base pattern, and support interface pattern option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6114-6176` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:968-970`, `PrintConfig.hpp:1019`, `PrintConfig.hpp:172-177`, `PrintConfig.hpp:190-192`, `PrintConfig.cpp:312-320`, `PrintConfig.cpp:333-340`, `PrintConfig.cpp:6114-6176`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, support bottom-interface spacing behavior, support interface speed behavior, support base/interface pattern selection behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_bottom_interface_spacing`, `support_interface_speed`, `support_base_pattern`, and `support_interface_pattern` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for support bottom-interface spacing, interface speed, base/interface pattern selection, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `support_base_pattern_spacing`, `support_expansion`, `support_speed`, `support_style`, and following support options from `PrintConfig.cpp:6178+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
