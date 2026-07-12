# Consume Filament Tower Ironing Area Header Design

## Scope

Consume the existing Ares option `filament_tower_ironing_area` into concrete G-code header behavior. This is a source-cited Rust rewrite slice of OrcaSlicer configuration serialization, not a new Ares pipeline feature and not new option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1447` declares `((ConfigOptionFloats, filament_tower_ironing_area))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2713-2719` defines the option as `coFloats`, label `Tower ironing area`, `min = 0`, advanced mode, default `ConfigOptionFloats { 4. }`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes full print config into G-code comments as `; key = value`, skipping nil options and banned keys.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1352` and `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1561` consume the option for wipe-tower filament parameters, but that ironing movement behavior is outside this header-export slice.

## Current Ares State

- Ares already has source-cited metadata and registry defaults for `filament_tower_ironing_area`.
- `crates/ares-core/src/options/filament_config_export.rs` exports adjacent filament header fields through `FilamentConfigExports`, including `filament_tower_interface_pre_extrusion_length`, but does not export `filament_tower_ironing_area`.
- `crates/ares-core/src/gcode_header.rs` appends adjacent filament header lines in upstream order, but currently jumps from `filament_tower_interface_pre_extrusion_length` to `filament_cooling_final_speed`.
- `crates/ares-core/src/gcode.rs` already calls `options.filament_config_exports()?` before BTT thumbnail header suppression, so any new field wired into that export path is validated before suppressed headers are skipped.

## Design

Add `filament_tower_ironing_area` to the existing filament config header export path.

The option is `ConfigOptionFloats`, so it uses the existing `optional_float_vector_export` parser and serializer:

- Missing value emits no header line.
- JSON arrays of finite non-negative numbers emit comma-separated decimals.
- Empty arrays emit an empty header value.
- Scalars, strings, booleans, objects, null, negative values, and non-numeric array items are rejected with `SliceError::InvalidInput` naming `filament_tower_ironing_area`.

Header ordering follows the source-order `PrintConfig.hpp` boundary and the existing adjacent header chain:

1. `filament_tower_interface_pre_extrusion_dist`
2. `filament_tower_interface_pre_extrusion_length`
3. `filament_tower_ironing_area`
4. `filament_cooling_final_speed`

The neighboring upstream options `filament_tower_interface_purge_volume` and `filament_tower_interface_print_temp` remain deferred for separate source-cited slices.

## Included Behavior

- Add an optional serialized field to `FilamentConfigExports`.
- Parse and serialize configured `filament_tower_ironing_area` values into `; filament_tower_ironing_area = ...` header comments.
- Preserve validation before BTT thumbnail header suppression by using the existing pre-header `filament_config_exports()` validation path.
- Add focused async G-code tests for single value, multiple values, zero, empty vector, source-adjacent ordering, missing value, invalid values, and invalid-with-header-skipped behavior.

## Deferred Behavior

- No wipe tower movement, purge, interface extrusion, ironing path generation, temperature, or tool-change execution.
- No implementation of `filament_tower_interface_purge_volume`, `filament_tower_interface_print_temp`, stamping, or ramming behavior.
- No new option metadata and no changes to existing metadata modules.
- No exhaustive `append_full_config` parity beyond this selected header export.
- No file I/O, terminal behavior, UI, OpenGL, native viewer, or platform-specific logic in `ares-core`.

## Acceptance Criteria

- `cargo nextest run -p ares-core filament_tower_ironing_area_gcode` fails before implementation because the header line is missing, then passes after implementation.
- Adjacent filament header tests pass with `cargo nextest run -p ares-core filament_tower_interface_pre_extrusion_length_gcode filament_tower_ironing_area_gcode filament_cooling_final_speed_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- The implementation keeps touched Rust files at or below 400 LOC.

## Documentation Impact

Update `docs/roadmap.md` after implementation review approval to record that `filament_tower_ironing_area` now reaches concrete Ares G-code header output and to list deferred adjacent wipe-tower interface behavior.
