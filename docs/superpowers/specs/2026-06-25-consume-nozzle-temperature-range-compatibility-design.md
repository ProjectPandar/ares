# Consume Nozzle Temperature Range Compatibility Design

## Goal

Consume OrcaSlicer `nozzle_temperature_range_low` and `nozzle_temperature_range_high` as concrete Ares slicing validation. Multi-filament jobs whose configured nozzle temperatures are outside each other filament's recommended range must fail before G-code is produced.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1571-1572`: `PrintConfig` owns `nozzle_temperature_range_low` and `nozzle_temperature_range_high`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6487-6501`: option definitions, integer-vector defaults, and `low < high` meaning.
- `OrcaSlicer/src/libslic3r/Print.cpp:1052-1100`: `Print::check_multi_filaments_compatibility` resolves per-filament print temperatures and ranges, rejects invalid ranges, and rejects pairs whose temperatures are not mutually inside the other filament's range.
- `OrcaSlicer/src/libslic3r/Print.cpp:1177-1234`: print validation gathers used filament types, nozzle temperatures, and nozzle-temperature ranges before slicing proceeds.

## Rust Destination Boundary

- `crates/ares-core/src/options/nozzle_temperature.rs`: parse the two integer-vector range options and expose an internal validation method on `SliceOptions`.
- `crates/ares-core/src/pipeline.rs`: call the validation before model loading and before G-code construction.
- `crates/ares-core/src/tests/nozzle_temperature_gcode.rs`: add end-to-end slice tests proving incompatible multi-filament input fails and compatible input still produces G-code.
- `crates/ares-core/src/options/tests/nozzle_temperature_runtime.rs`: add option-level tests for range defaults, parsing, invalid range order, and invalid vector values.

## Included Behavior

1. Omitted `nozzle_temperature_range_low` defaults to `[190]`; omitted `nozzle_temperature_range_high` defaults to `[240]`.
2. The existing `nozzle_temperature` parser supplies per-filament print temperatures; omitted values default to `[200]`.
3. Integer vector parsing follows the existing Orca-compatible `temperature_vector::parse_integer_vector` contract: integers, integer strings, semicolon/comma strings, and integer arrays are accepted; negative, fractional, empty, null, object, and non-integer entries are rejected.
4. Validation uses Orca vector `get_at` semantics already used elsewhere in Ares: missing per-filament entries fall back to the first configured value.
5. The effective filament count is the maximum length among `nozzle_temperature`, `nozzle_temperature_range_low`, `nozzle_temperature_range_high`, `filament_type`, `filament_diameter`, and `nozzle_diameter`, with a minimum of one. Counts below two are compatible.
6. For every effective filament index, `range_low < range_high` must hold after first-value fallback. Otherwise slicing returns `SliceError::InvalidInput` naming both `nozzle_temperature_range_low` and `nozzle_temperature_range_high`.
7. For every pair of effective filaments, each filament's configured print temperature must be inside the other filament's range, inclusive. Otherwise slicing returns `SliceError::InvalidInput` naming `nozzle_temperature`, `nozzle_temperature_range_low`, and `nozzle_temperature_range_high`.
8. Validation runs even when no machine-start placeholder references these options, so the options affect slicing behavior rather than only metadata or header output.

## Deferred Behavior

- Orca `MaterialType::get_temperature_range` fallback for zero low/high entries is deferred because Ares does not yet model the material temperature table.
- `enable_high_low_temp_mixed_printing` warning-vs-error preference handling is deferred; Ares will reject incompatible pairs in this slice.
- By-object validation using each object's exact used extruder set is deferred. Ares currently has a single active extrusion path and no per-object tool ownership.
- Full multi-extruder toolchange, wipe tower, support-specific extruder ownership, UI warnings, localization, and preset behavior are deferred.
- No new option metadata, crates, dependencies, file I/O, terminal behavior, UI behavior, or Ares-owned pipeline design is introduced.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core nozzle_temperature_range` fails before implementation because incompatible two-filament input still slices successfully.
- After implementation, `cargo nextest run -p ares-core nozzle_temperature_range` passes.
- Adjacent temperature tests pass with `cargo nextest run -p ares-core nozzle_temperature_gcode nozzle_temperature_runtime enable_high_low_temp_mix_placeholder_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- Touched Rust files remain at or below 400 LOC.

## Safety

The change is validation-only inside `ares-core`. It has no filesystem, network, terminal, UI, or platform-specific side effects and is compatible with WASM, Windows, macOS, and Linux.
