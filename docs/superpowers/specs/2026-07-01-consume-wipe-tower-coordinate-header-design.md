# Consume Wipe Tower Coordinate Header Design

## Goal

Consume the already-registered Orca wipe-tower coordinate vector options through Ares' existing G-code config header export path, validating malformed values before G-code bytes are returned and preserving Orca's selected-plate fixed-coordinate header branch, without implementing wipe-tower placement, plate selection, duplicate generic coordinate serialization, collision checks, purge planning, or part-plate logic.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1576-1578`: `PrintConfig` documents the BBS change to `wipe_tower_x` and `wipe_tower_y` as `ConfigOptionFloats` for part-plate logic.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6694-6708`: `wipe_tower_x` and `wipe_tower_y` are `coFloats`, `comDevelop`, with defaults `{ 15. }` and `{ 220. }`, and no explicit min/max bounds.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5558-5574`: G-code config header serialization special-cases `wipe_tower_x` and `wipe_tower_y`, first writing `get_at(print.get_plate_index())` as a fixed three-decimal value, then falling through the following generic `cfg.opt_serialize(key)` branch because the coordinate special case is not an `else if`.
- `OrcaSlicer/src/libslic3r/Config.hpp:845-862` and `910-919`: `ConfigOptionFloats` generic serialization writes the whole vector with ordinary stream formatting, which differs from the fixed selected-coordinate line.
- `OrcaSlicer/src/libslic3r/Print.cpp:267-269`: coordinate changes invalidate Orca skirt/brim placement.
- `OrcaSlicer/src/libslic3r/Print.cpp:1001-1004`: Orca combines the selected coordinate values with the plate origin for wipe-tower collision checks.
- `OrcaSlicer/src/libslic3r/Print.cpp:2388` and `2545`: Orca sets fake wipe-tower position from selected coordinate values.
- `OrcaSlicer/src/libslic3r/Print.cpp:2846`: Orca offsets wipe-tower mesh/corner points by the selected coordinates and plate origin.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1463`: legacy `WipeTower` stores the selected coordinate pair as runtime position.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1252`: `WipeTower2` stores the selected coordinate pair as runtime position.

## Current Ares Boundary

- Registry metadata for `wipe_tower_x` and `wipe_tower_y` already exists in `crates/ares-core/src/options/registry/definitions/table/tail_terminal_wipe.rs` with source citations to `PrintConfig.hpp` and `PrintConfig.cpp`.
- `crates/ares-core/src/options/filament_config_export.rs` is the current config-header snapshot boundary used by adjacent wipe-tower header options.
- `crates/ares-core/src/gcode_config_header.rs` appends the current wipe-tower config-header group.
- `crates/ares-core/src/options/filament_config_export/serialization.rs` owns float-vector validation and serialization helpers.
- `crates/ares-core/src/gcode.rs:58` calls `options.filament_config_exports()?` before `crates/ares-core/src/gcode_file_start.rs:27` can skip normal header output for BTT thumbnail settings, so export validation still runs when the header is suppressed.
- `crates/ares-core/src/options/filament_config_export.rs`, `crates/ares-core/src/options/filament_config_export/serialization.rs`, and `crates/ares-core/src/gcode_config_header.rs` are below the 400 LOC split threshold but must stay below it after this slice; split into a focused submodule only if the implementation would exceed that threshold.
- Ares does not yet model Orca plate index selection, part-plate coordinate logic, wipe-tower placement, fake wipe-tower state, collision checks, mesh/corner offsets, `WipeTower`, or `WipeTower2`.

## Design

Extend `FilamentConfigExports` with optional header-export strings for:

```rust
pub(crate) wipe_tower_x: Option<String>,
pub(crate) wipe_tower_y: Option<String>,
```

Populate them from the existing `SliceOptions` values map using a coordinate-specific helper:

- Accept only JSON arrays of finite numeric values.
- Reject scalar, string, bool, object, null, non-number array entries, non-finite values, and empty arrays with `SliceError::InvalidInput` naming the offending key.
- Use no min/max bound because the cited Orca definitions do not define coordinate bounds.
- Serialize the first value as a single fixed three-decimal string, matching Orca's selected-plate `std::fixed << std::setprecision(3)` header branch for Ares' current single/default plate boundary.

Do not use `optional_float_vector_export()` for these two header fields because that helper serializes the whole vector with Ares' trimmed decimal format. This slice consumes only Orca's selected-coordinate fixed branch and intentionally defers the apparent duplicate generic `cfg.opt_serialize(key)` line from `GCode.cpp:5569-5572` until Ares has a broader config-header parity boundary.

Append the two lines immediately before `prime_tower_width` and before the existing `wipe_tower_rotation_angle` line:

```text
; wipe_tower_x = ...
; wipe_tower_y = ...
; wipe_tower_rotation_angle = ...
```

This follows the upstream `PrintConfig.hpp` declaration order around `wipe_tower_x`, `wipe_tower_y`, `prime_tower_width`, `wipe_tower_per_color_wipe`, and `wipe_tower_rotation_angle` while keeping this slice limited to the two already-registered coordinate options. `prime_tower_width` and `wipe_tower_per_color_wipe` remain deferred.

Missing options emit no config line. Invalid values return `SliceError::InvalidInput` before G-code bytes are returned, including when BTT thumbnail settings skip normal header output.

## Alternatives Considered

- Serialize only the full coordinate vector as `15,220`: rejected because Orca's config-header path first special-cases these keys and writes the selected plate's scalar value with three decimals.
- Emit both Orca coordinate header lines now: rejected because the second line is an apparent generic serialization fallthrough and the current Ares boundary has no full config-key iteration parity; this slice records that duplicate-line behavior as deferred rather than introducing two same-key lines in the focused header-export shell.
- Add a general selected-plate API now: rejected because Ares does not yet model Orca plate index/part-plate behavior and this slice is only a config-header consumption boundary.
- Use `format_decimal()` or trimmed vector formatting: rejected because Orca uses fixed three-decimal formatting for the header branch.
- Invent coordinate min/max bounds: rejected because the cited `PrintConfig.cpp` definitions do not define bounds.
- Implement wipe-tower placement/collision behavior now: rejected because that requires separate `Print`, fake wipe-tower, `WipeTower`, and `WipeTower2` runtime boundaries.

## Behavior Included

- Header export and validation for the two already-registered coordinate vector options.
- Finite numeric array validation with no invented bounds.
- Single selected-coordinate header serialization using the first vector value and fixed three-decimal formatting.
- Exactly one header line per coordinate key in Ares' current boundary.
- Missing-value behavior that preserves current output and omits the key.
- Invalid-value behavior that fails before G-code bytes are returned, including when normal header lines are skipped.

## Behavior Deferred

- Orca plate index selection beyond Ares' current first/default plate.
- Generic duplicate `cfg.opt_serialize(key)` coordinate line parity from `GCode.cpp:5569-5572`.
- Part-plate coordinate logic.
- Wipe-tower placement, fake wipe-tower state, and collision checks.
- Wipe-tower mesh/corner offsets and rotation-aware placement.
- Legacy `WipeTower` and `WipeTower2` runtime position state.
- `prime_tower_width`, `wipe_tower_per_color_wipe`, and other adjacent wipe-tower behavior.
- UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity.

## Docs Impact

Update `docs/roadmap.md` after implementation review with a source-cited runtime-slice entry for the consumed wipe-tower coordinate config header options. No CLI/API documentation changes are required.

## Safety And Rollback

This is a header-export-only consumption slice. The only intended runtime effect is that explicit malformed coordinate values fail with `SliceError::InvalidInput` before G-code bytes are returned, while explicit valid coordinate vectors gain one selected-coordinate config-header line per key. It adds no wipe-tower placement, collision checking, part-plate selection, public API, CLI, WASM, dependency, registry, or saturated-file changes. Rollback is limited to removing the two optional export fields, the coordinate export helper, their population assignments, the two header append calls, the focused tests, and the roadmap entry.

## Acceptance Criteria

- G-code config-header tests prove explicit valid arrays emit fixed three-decimal selected-coordinate lines:
  - `; wipe_tower_x = 12.346`
  - `; wipe_tower_y = 220.000`
- G-code config-header tests prove additional vector entries are accepted but not emitted in the current single/default plate boundary.
- G-code config-header tests prove Ares emits exactly one `; wipe_tower_x = ...` line and exactly one `; wipe_tower_y = ...` line, documenting that the apparent generic duplicate `cfg.opt_serialize(key)` line from `GCode.cpp:5569-5572` remains deferred.
- G-code config-header tests prove zero and negative coordinate values are accepted and fixed to three decimals because upstream defines no bounds.
- Header-order tests prove the coordinate lines appear before `wipe_tower_rotation_angle`, `wipe_tower_bridging`, and the existing rib-wall/small-area header group.
- Absence tests prove omitted coordinate options emit no header lines.
- Invalid-value tests prove scalar inputs, empty arrays, non-number array entries, nulls, and non-finite values return `SliceError::InvalidInput` naming the offending key.
- Invalid values are rejected even when BTT thumbnail settings skip normal header output.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` remain untouched and at 400 LOC or less.
- `crates/ares-core/src/tests/wipe_tower_config_header_gcode.rs` remains untouched because it is already near the 400 LOC split threshold; use a new focused test module for coordinate coverage.
- All touched Rust files remain at or below 400 LOC.
- `docs/roadmap.md` is updated after implementation review with the source-cited slice summary and deferred behavior.
- Verification passes with:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core wipe_tower_coordinate_header`
  - `cargo nextest run -p ares-core wipe_tower_config_header`
  - `cargo nextest run -p ares-core small_area_flow_model_header`
  - `cargo nextest run --workspace`
