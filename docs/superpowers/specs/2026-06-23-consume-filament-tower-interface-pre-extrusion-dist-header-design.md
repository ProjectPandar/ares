# Consume Filament Tower Interface Pre-Extrusion Distance Header Design

## Scope

Consume the existing Ares option `filament_tower_interface_pre_extrusion_dist` into concrete G-code header behavior. This is a source-cited Rust rewrite slice of OrcaSlicer configuration serialization, not a new Ares pipeline feature and not new option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1445` declares `((ConfigOptionFloats, filament_tower_interface_pre_extrusion_dist))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2697-2703` defines the option as `coFloats`, label `Interface layer pre-extrusion distance`, `min = 0`, advanced mode, default `ConfigOptionFloats { 10. }`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes full print config into G-code comments as `; key = value`, skipping nil options and banned keys.

## Current Ares State

- Ares retains the registry definition for `filament_tower_interface_pre_extrusion_dist`; the former source-line-only tuple module was removed by the Option pinning cleanup.
- Ares already preserves the key in `SliceOptions` and registry default tests.
- `crates/ares-core/src/options/filament_config_export.rs` exports adjacent filament header fields such as `filament_cooling_before_tower` and `filament_cooling_final_speed`, but does not export `filament_tower_interface_pre_extrusion_dist`.
- `crates/ares-core/src/gcode_header.rs` appends adjacent filament header lines, but currently jumps from `filament_cooling_before_tower` to `filament_cooling_final_speed`.

## Design

Add `filament_tower_interface_pre_extrusion_dist` to Ares' existing filament config header export path.

The option is `ConfigOptionFloats`, so it uses the existing `optional_float_vector_export` parser and serializer:

- Missing value emits no header line.
- JSON arrays of finite non-negative numbers emit comma-separated decimals.
- Empty arrays emit an empty header value.
- Scalars, strings, booleans, objects, null, negative values, and non-numeric array items are rejected with `SliceError::InvalidInput` naming `filament_tower_interface_pre_extrusion_dist`.

Header ordering follows the source-order `PrintConfig.hpp` boundary and the existing adjacent header chain:

1. `filament_minimal_purge_on_wipe_tower`
2. `filament_cooling_before_tower`
3. `filament_tower_interface_pre_extrusion_dist`
4. `filament_cooling_final_speed`

This intentionally leaves the neighboring upstream options `filament_tower_interface_pre_extrusion_length`, `filament_tower_ironing_area`, `filament_tower_interface_purge_volume`, and `filament_tower_interface_print_temp` deferred for separate source-cited slices.

## Included Behavior

- Add an optional serialized field to `FilamentConfigExports`.
- Parse and serialize configured `filament_tower_interface_pre_extrusion_dist` values into `; filament_tower_interface_pre_extrusion_dist = ...` header comments.
- Preserve validation before BTT thumbnail header suppression, matching the existing header-export path.
- Add focused async G-code tests for single value, multiple values, zero, empty vector, source-adjacent ordering, missing value, invalid values, and invalid-with-header-skipped behavior.

## Deferred Behavior

- No wipe tower movement, purge, interface extrusion, temperature, or tool-change execution.
- No implementation of `filament_tower_interface_pre_extrusion_length`, `filament_tower_ironing_area`, `filament_tower_interface_purge_volume`, `filament_tower_interface_print_temp`, stamping, or ramming behavior.
- No new source-line-only option metadata.
- No exhaustive `append_full_config` parity beyond this selected header export.
- No file I/O, terminal behavior, UI, OpenGL, native viewer, or platform-specific logic in `ares-core`.

## Acceptance Criteria

- `cargo nextest run -p ares-core filament_tower_interface_pre_extrusion_dist_gcode` fails before implementation because the header line is missing, then passes after implementation.
- Adjacent filament header tests pass with `cargo nextest run -p ares-core filament_cooling_before_tower_gcode filament_cooling_final_speed_gcode filament_tower_interface_pre_extrusion_dist_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- The implementation keeps touched Rust files at or below 400 LOC.

## Documentation Impact

Update `docs/roadmap.md` after implementation review approval to record that `filament_tower_interface_pre_extrusion_dist` now reaches concrete Ares G-code header output and to list deferred adjacent wipe-tower interface behavior.
