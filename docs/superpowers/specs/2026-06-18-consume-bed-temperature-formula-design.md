# Consume Bed Temperature Formula Design

## Scope

Implement the next concrete Orca rewrite slice for `bed_temperature_formula` in generated bed-temperature G-code. This consumes the existing Ares option metadata instead of adding new options or milestone-only scaffolding.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:118-122` defines `enum class BedTempFormula` with `btfFirstFilament` and `btfHighestTemp`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1340` declares `((ConfigOptionEnum<BedTempFormula>, bed_temperature_formula))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:178-182` maps serialized values `"by_first_filament"` and `"by_highest_temp"`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2503-2512` registers `bed_temperature_formula`, lists both enum values, and defaults to `BedTempFormula::btfHighestTemp`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2991`, `3975`, and `4682` branch on `btfHighestTemp` to use the highest bed temperature instead of the first printing extruder's bed temperature.

## Ares Boundary

- Runtime behavior lives in `crates/ares-core/src/options/bed_temperature.rs`.
- G-code regression tests live in `crates/ares-core/src/tests/bed_temperature_gcode.rs` and `crates/ares-core/src/tests/other_layer_temperature_gcode.rs`.
- Existing option registry metadata in `crates/ares-core/src/options/registry/definitions/table/early.rs` already exposes `bed_temperature_formula` with default `"by_highest_temp"` and must not be expanded in this slice.

## Behavior

- Parse `bed_temperature_formula` from `SliceOptions` as an internal enum.
- Missing `bed_temperature_formula` uses Orca's default `"by_highest_temp"`.
- `"by_highest_temp"` selects the maximum value from the selected bed-temperature vector.
- `"by_first_filament"` preserves the existing first-entry behavior.
- The formula applies to both first-layer bed temperature and other-layer bed temperature.
- When the other-layer bed-temperature key is missing, fallback to the computed first-layer bed temperature using the same formula.
- Invalid values return `SliceError::InvalidInput` and include `bed_temperature_formula` in the message.

## Out Of Scope

- No new option metadata or milestone registry modules.
- No multi-tool, multi-extruder scheduling beyond choosing a scalar from existing temperature vectors.
- No nozzle-temperature formula changes.
- No custom placeholder expansion changes.
- No UI, CLI, WASM, or file I/O changes.

## Acceptance Criteria

- A first-layer test with `cool_plate_temp_initial_layer: [35, 65, 45]` and default formula emits `M190 S65`.
- A first-layer test with `"bed_temperature_formula": "by_first_filament"` and the same vector emits `M190 S35`.
- A second-layer transition test with `hot_plate_temp: [60, 72, 68]` and default formula emits `M140 S72`.
- A fallback test with missing other-layer bed temperature and `cool_plate_temp_initial_layer: [35, 65]` emits no redundant `M140`, proving both first and other layer values resolve through the same highest-temperature fallback.
- An invalid formula test returns `SliceError::InvalidInput` containing `bed_temperature_formula`.
- Verification passes with targeted tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the Rust LOC gate.
