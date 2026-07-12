# Consume Filament Load/Unload Speed Header Design

## Goal

Consume the existing Orca filament load/unload speed options as concrete Ares G-code header behavior. When a caller supplies these float-vector options, Ares must emit serialized config lines in generated G-code:

- `; filament_loading_speed = ...`
- `; filament_loading_speed_start = ...`
- `; filament_unloading_speed = ...`
- `; filament_unloading_speed_start = ...`

This is a narrow `GCode::append_full_config`-style export slice. It turns already-staged options into observable G-code output without implementing wipe-tower movement generation.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2603-2634` defines the four options as `coFloats`, sets `min = 0`, and provides defaults `28`, `3`, `90`, and `100`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1436-1439` places the four keys in the `GCodeConfig` option tuple list as `ConfigOptionFloats`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`, iterating full config keys and appending each non-banned, non-nil option as `; key = cfg.opt_serialize(key)`. These four keys are not in the banned list.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1357-1362` shows the adjacent runtime consumer for single-extruder multi-material wipe-tower filament parameters. That movement/toolchange behavior is explicitly deferred in this slice.

## Current Ares State

- Registry metadata for the four keys exists under `crates/ares-core/src/options/registry`; the former source-line-only tuple modules were removed by the Option pinning cleanup.
- `crates/ares-core/src/options/filament_config_export.rs` owns the curated Orca-compatible header export path for filament options, including float-vector serialization for `filament_change_length`.
- `crates/ares-core/src/gcode_header.rs` appends selected filament config exports before note exports.

## Design

Extend the existing filament config export path:

1. Add a focused G-code test module for the four load/unload speed keys.
2. Add four `Option<String>` fields to `FilamentConfigExports`.
3. Populate each field with the existing non-negative finite float-vector export helper.
4. Append each present field in `gcode_header.rs` using the exact upstream key spelling.

No new parsing framework, public API, dependency, crate, or movement pipeline is needed. Missing values remain absent from the optional Ares header export, matching the existing curated header behavior for other filament config exports.

## Included Behavior

- A supplied single value such as `"filament_loading_speed": [28.0]` emits `; filament_loading_speed = 28`.
- Multiple supplied values such as `"filament_unloading_speed": [90.0, 80.5]` emit comma-separated `ConfigOptionFloats` formatting: `; filament_unloading_speed = 90,80.5`.
- Zero is accepted and emitted because upstream sets `min = 0`.
- Empty arrays are accepted and emitted as an empty serialized `ConfigOptionFloats` value, for example `; filament_loading_speed = `.
- Missing keys emit no optional header line for those keys.
- Invalid values are rejected with `SliceError::InvalidInput`, including scalars, strings, bool arrays, string arrays, negative numbers, objects, and null.
- Invalid values are rejected even when BTT thumbnail header suppression is requested, because Ares validates filament config exports before deciding whether to skip the header.

## Docs Impact

- Update `docs/roadmap.md` with a dated entry for this concrete header/runtime slice, citing the same upstream boundary and listing the explicitly deferred wipe-tower/toolchange behavior.
- No architecture ADR is required because this reuses the existing curated `GCode::append_full_config` header-export boundary and does not change crate boundaries, public APIs, platform constraints, or pipeline architecture.

## Deferred Behavior

- Do not implement wipe-tower loading/unloading path generation, ramming behavior, toolchange G-code, `filament_toolchange_delay`, or `filament_cooling_*` behavior.
- Do not implement `GCode/WipeTower.cpp` or `GCode/WipeTower2.cpp` movement semantics in this slice.
- Do not make `append_full_config` exhaustive for every Orca key.
- Do not add new dependencies, crates, filesystem access, terminal behavior, UI behavior, OpenGL/viewer behavior, or Ares-owned pipeline design.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core filament_load_unload_speed_gcode` fails before implementation because at least one expected header line is absent.
- After implementation, `cargo nextest run -p ares-core filament_load_unload_speed_gcode` passes.
- The focused tests include empty-array coverage for at least one of the four keys and expect an empty serialized header value.
- Adjacent filament header export tests pass with `cargo nextest run -p ares-core filament_change_length_gcode filament_adhesiveness_category_gcode filament_printable_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- All touched Rust source files remain at or below 400 LOC.
