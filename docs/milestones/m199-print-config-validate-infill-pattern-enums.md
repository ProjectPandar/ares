# M199: PrintConfig validate infill pattern enum values

## Goal
Port OrcaSlicer's fill-pattern validation slice into Ares as an explicit `SliceOptions::validate_infill_pattern_options()` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10152-10170`, with option enum-value context from `PrintConfig.cpp:1986-2025` for `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern`, `PrintConfig.cpp:2928-2985` for `sparse_infill_pattern`, and serialization enum context from `PrintConfig.cpp:225-255` / `PrintConfig.hpp:87-98`. It covers only `has_enum_value` validation and resulting errors for these four pattern options. No skirt-height, bridge-flow, later validation, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_infill_pattern_options()` returns a key-to-message map like Orca validation.
- Missing pattern options use source-cited registry defaults and pass.
- `sparse_infill_pattern` accepts exactly active enum values from `PrintConfig.cpp:2928-2985`: `rectilinear`, `alignedrectilinear`, `zigzag`, `crosszag`, `lockedzag`, `line`, `grid`, `triangles`, `tri-hexagon`, `cubic`, `adaptivecubic`, `quartercubic`, `supportcubic`, `lightning`, `honeycomb`, `3dhoneycomb`, `lateral-honeycomb`, `lateral-lattice`, `crosshatch`, `tpmsd`, `tpmsfk`, `gyroid`, `concentric`, `hilbertcurve`, `archimedeanchords`, and `octagramspiral`.
- `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` accept exactly active enum values from `PrintConfig.cpp:1986-2025`: `monotonic`, `monotonicline`, `rectilinear`, `alignedrectilinear`, `concentric`, `hilbertcurve`, `archimedeanchords`, and `octagramspiral`.
- Unknown or inactive strings report `invalid value {value}` under their own option key.
- JSON boundary type errors for non-string pattern values return `SliceError::InvalidInput`.
- Existing M196, M197, and M198 validation behavior remains intact.
- Existing `crates/ares-core/src/options/tests/validation.rs` is split into submodules before adding M199 tests so modified Rust files remain under 400 LOC.
- `PrintConfig.cpp:10172+` validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
