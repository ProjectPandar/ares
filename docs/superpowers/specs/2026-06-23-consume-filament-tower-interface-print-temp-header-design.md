# Consume Filament Tower Interface Print Temperature Header Design

## Scope

Consume the existing Ares option `filament_tower_interface_print_temp` into concrete G-code header behavior. This is a source-cited Rust rewrite slice of OrcaSlicer configuration serialization, not a new Ares pipeline feature and not new option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1449` declares `((ConfigOptionInts, filament_tower_interface_print_temp))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2729-2735` defines the option as `coInts`, label `Interface layer print temperature`, `min = -1`, advanced mode, default `ConfigOptionInts { -1 }`, and documents that `-1` means using the max recommended nozzle temperature.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes full print config into G-code comments as `; key = value`, skipping nil options and banned keys.
- `OrcaSlicer/src/libslic3r/GCode.cpp:900`, `GCode.cpp:954`, `GCode.cpp:1200`, and `GCode.cpp:7829-7832` copy or choose the interface print temperature for dynamic wipe-tower/tool-change config, while `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1345` and `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1554` consume it for wipe-tower filament parameters. That print-temperature execution behavior is outside this header-export slice.

## Current Ares State

- Ares already has source-cited metadata and registry defaults for `filament_tower_interface_print_temp`.
- `crates/ares-core/src/options/filament_config_export.rs` exports adjacent filament header fields through `FilamentConfigExports`, including `filament_tower_interface_purge_volume` and `filament_cooling_final_speed`, but does not export `filament_tower_interface_print_temp`.
- `crates/ares-core/src/gcode_header.rs` appends adjacent filament config exports to the G-code header but does not append `filament_tower_interface_print_temp`.

## Design

Add `filament_tower_interface_print_temp` to the existing filament config header export path.

- Use the existing integer-vector header serialization path in `crates/ares-core/src/options/filament_config_export.rs`.
- Validate values with the Orca option minimum: integers must be `>= -1`.
- Preserve Ares' existing header behavior: missing option means no header line, empty vector serializes as an empty value, and BTT thumbnail header suppression still suppresses header output while validation happens before suppression.
- Do not introduce file I/O, terminal behavior, UI behavior, OpenGL, new crates, new dependencies, or a separate Ares-owned pipeline path.

Header ordering follows the source-order `PrintConfig.hpp` boundary and the existing adjacent header chain:

1. `filament_tower_ironing_area`
2. `filament_tower_interface_purge_volume`
3. `filament_tower_interface_print_temp`
4. `filament_cooling_final_speed`

## Included Behavior

- Parse and serialize configured `filament_tower_interface_print_temp` values into `; filament_tower_interface_print_temp = ...` header comments.
- Accept `-1`, `0`, and positive integer vector values.
- Reject scalars, strings, booleans, objects, null, values below `-1`, and non-integer array items with `SliceError::InvalidInput` naming `filament_tower_interface_print_temp`.
- Keep the new behavior platform-neutral and usable from WASM through `ares-core`.

## Deferred Behavior

- No implementation of wipe-tower interface print temperature execution in `GCode.cpp:900`, `GCode.cpp:954`, `GCode.cpp:1200`, `GCode.cpp:7829-7832`, `WipeTower2.cpp:1345`, or `WipeTower.cpp:1554`.
- No implementation of max recommended nozzle temperature fallback semantics beyond preserving `-1` as a serialized configured value.
- No neighboring `filament_cooling_final_speed`, `filament_ramming_parameters`, stamping, ramming, or full wipe-tower behavior changes.
- No new option metadata.

## Acceptance Criteria

- `cargo nextest run -p ares-core filament_tower_interface_print_temp_gcode` fails before implementation because the header line is missing, then passes after implementation.
- Adjacent filament header tests pass with `cargo nextest run -p ares-core filament_tower_interface_purge_volume_gcode filament_tower_interface_print_temp_gcode filament_cooling_final_speed_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- Independent spec, plan, and implementation reviewers return `VERDICT: APPROVE`.

## Documentation

Update `docs/roadmap.md` after implementation review approval to record that `filament_tower_interface_print_temp` now reaches concrete Ares G-code header output and to list deferred adjacent wipe-tower interface temperature behavior.
