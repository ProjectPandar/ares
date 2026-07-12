# Consume Filament Ramming Parameters Header Design

## Scope

Consume the existing Ares option `filament_ramming_parameters` into concrete G-code header behavior. This is a source-cited Rust rewrite slice of OrcaSlicer configuration serialization, not a new Ares pipeline feature and not new option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1451` declares `((ConfigOptionStrings, filament_ramming_parameters))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2745-2750` defines the option as `coStrings`, label `Ramming parameters`, advanced mode, and a long default ramming parameter string edited by `RammingDialog`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes full print config into G-code comments as `; key = value`, skipping nil options and banned keys.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1380-1390` parses `filament_ramming_parameters` into SEMM ramming line-width, step multiplicator, and speed values. `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1591-1601` contains disabled legacy parsing under `#if 0`. That ramming execution behavior is outside this header-export slice.

## Current Ares State

- Ares already has source-cited metadata and registry defaults for `filament_ramming_parameters`.
- `crates/ares-core/src/options/filament_config_export.rs` exports adjacent filament header fields through `FilamentConfigExports`, including `filament_cooling_final_speed`, but does not export `filament_ramming_parameters`.
- `crates/ares-core/src/gcode_header.rs` appends adjacent filament config exports to the G-code header but does not append `filament_ramming_parameters`.

## Design

Add `filament_ramming_parameters` to the existing filament config header export path.

- Use the existing Orca-compatible string-vector serialization path in `crates/ares-core/src/options/filament_config_export.rs`.
- Preserve Ares' existing string serialization behavior: values are separated with `;`, complex values are quoted/escaped through the existing string serializer, an empty JSON array serializes as an empty header value, and a missing option emits no header line.
- Validate the configured value as a JSON array of strings; reject scalars, numbers, booleans, objects, null, and non-string array entries with `SliceError::InvalidInput` naming `filament_ramming_parameters`.
- Preserve BTT thumbnail behavior: header output is suppressed when appropriate, but invalid `filament_ramming_parameters` values are still rejected before suppression through the existing `filament_config_exports()` path.
- Do not introduce file I/O, terminal behavior, UI behavior, OpenGL, new crates, new dependencies, or a separate Ares-owned pipeline path.

Header ordering follows the source-order `PrintConfig.hpp` boundary and the existing adjacent header chain:

1. `filament_tower_interface_print_temp`
2. `filament_cooling_final_speed`
3. `filament_ramming_parameters`
4. `filament_multitool_ramming`

## Included Behavior

- Parse and serialize configured `filament_ramming_parameters` string-vector values into `; filament_ramming_parameters = ...` header comments.
- Preserve Orca-compatible `ConfigOptionStrings` formatting for ordinary strings, empty strings, whitespace, semicolons, newlines, and quotes through existing serializer behavior.
- Reject invalid values with `SliceError::InvalidInput` naming `filament_ramming_parameters`.
- Keep the new behavior platform-neutral and usable from WASM through `ares-core`.

## Deferred Behavior

- No implementation of SEMM ramming parameter parsing or ramming speed execution from `WipeTower2.cpp:1380-1390`.
- No implementation of disabled legacy `WipeTower.cpp:1591-1601` parsing.
- No implementation of neighboring `filament_multitool_ramming`, `filament_multitool_ramming_volume`, `filament_multitool_ramming_flow`, stamping, or full wipe-tower behavior.
- No new option metadata.

## Acceptance Criteria

- `cargo nextest run -p ares-core filament_ramming_parameters_gcode` fails before implementation because the header line is missing, then passes after implementation.
- Adjacent filament header tests pass with `cargo nextest run -p ares-core filament_cooling_final_speed_gcode filament_ramming_parameters_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- Independent spec, plan, and implementation reviewers return `VERDICT: APPROVE`.

## Documentation

Update `docs/roadmap.md` after implementation review approval to record that `filament_ramming_parameters` now reaches concrete Ares G-code header output and to list deferred ramming execution behavior.
