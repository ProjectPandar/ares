# Consume Filament Density Header Design

## Objective

Consume the existing OrcaSlicer `filament_density` option in Ares G-code output instead of leaving it as metadata-only registry coverage. This slice ports the narrow header-output behavior from Orca `libslic3r/GCode.cpp` while preserving the larger statistics and cost calculations for later source-cited slices.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1321` declares `((ConfigOptionFloats, filament_density))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2776-2782` defines `filament_density` as `coFloats`, label `Density`, unit `g/cm³`, minimum `0`, advanced mode, default `ConfigOptionFloats { 0. }`, and tooltip `Filament density. For statistics only.`
- `OrcaSlicer/src/libslic3r/GCode.cpp:2570-2572` writes `; filament_density: ` plus `m_config.filament_density.serialize()` into the generated G-code header.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2574-2576` writes the adjacent `filament_diameter` header line; Ares already emits `filament_diameter` in its own header format.

## Ares Destination Boundary

- `crates/ares-core/src/gcode_header.rs` owns Ares header formatting and should emit the new `; filament_density = ...` line next to `; filament_diameter = ...`.
- `crates/ares-core/src/options/parsing.rs` may widen the existing numeric-vector parser visibility from `pub(super)` to `pub(crate)` so header formatting can reuse it without duplicating parsing rules.
- `crates/ares-core/src/pipeline/tests/filament_density_header.rs` should cover concrete G-code output behavior.
- `crates/ares-core/src/pipeline/tests.rs` should register the focused pipeline test module.
- `docs/roadmap.md` should record that this slice consumed the header-output portion and explicitly deferred statistics/cost behavior.
- Do not add code or tests to `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, or `crates/ares-core/src/options/tests/core.rs`; those files are at or near the 400 LOC limit.

## Included Behavior

- Missing `filament_density` emits the Orca default value as an Ares header comment line: `; filament_density = 0`.
- Numeric vector forms accepted for `filament_density` match existing Orca-style Ares vector parsing:
  - scalar number, for example `1.24`
  - scalar numeric string, for example `"1.24"`
  - semicolon- or comma-separated string, for example `"1.24;1.27"` or `"1.24,1.27"`
  - JSON array containing numbers or numeric strings
- Values must be finite and non-negative, matching the upstream `min = 0` definition.
- Header formatting uses existing `format_decimal_list`, matching current Ares header style for `filament_diameter`.
- `filament_density` is header/statistics metadata for this slice and must not affect extrusion E values, paths, layer generation, fan behavior, speeds, temperatures, or G-code movement commands.
- BTT thumbnail header suppression continues to suppress the entire Ares header, including the new `filament_density` line.

## Deferred Behavior

- Orca print statistics weight/cost calculations from `GCode.cpp:1681`, `GCode.cpp:1926`, `GCode.cpp:2300`, `GCode.cpp:2329`, and final summary emission around `GCode.cpp:3479-3481`.
- Exact Orca `ConfigOptionFloats::serialize()` punctuation and the colon-form header line. Ares keeps its established `; key = value` header convention in this slice.
- Full upstream config block generation, wipe-tower statistics, multi-extruder material accounting, UI statistics, and post-processing placeholder replacement.

## Acceptance Criteria

- Missing `filament_density` reaches `format_gcode` as the default header line `; filament_density = 0`.
- `filament_density` accepts the same numeric vector forms as `filament_diameter` and rejects empty, non-numeric, nested, negative, and non-finite values with `SliceError::InvalidInput` from the G-code formatting boundary.
- `format_gcode` emits `; filament_density = 1.24,1.27` when the option is configured as `"1.24;1.27"`.
- `format_gcode` emits `; filament_density = 0` when the option is absent.
- Changing only `filament_density` changes header comments but does not change extrusion `E` values or movement G-code.
- `thumbnails` containing `BTT_TFT` suppresses the `filament_density` header line through the existing full-header suppression path.
- Focused and full verification use `cargo nextest run`, not `cargo test`.

## Safety And Rollback

This slice is additive and limited to header output and boundary parsing. Rollback is a single commit revert. No new dependencies, crates, file I/O, terminal behavior, UI behavior, OpenGL, or platform-specific code are introduced.

## Spec Self-Review

- No placeholders remain.
- Scope is a single source-cited Orca behavior slice.
- The runtime behavior is concrete G-code output, not new option metadata.
- Deferred statistics/cost behavior is explicitly named and excluded from this slice.
