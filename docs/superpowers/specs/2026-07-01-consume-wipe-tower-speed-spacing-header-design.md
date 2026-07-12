# Consume Wipe Tower Speed Spacing Header Design

## Goal

Consume the already-registered Orca wipe-tower speed, spacing, flow, rotation, cone, and bridging scalar option group through Ares' existing G-code config header export path, validating malformed values before G-code bytes are returned, without implementing wipe-tower purge planning, sparse-layer generation, speed selection, cone geometry, or bridge-spacing behavior.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1581`: `PrintConfig` declares `wipe_tower_rotation_angle` as `ConfigOptionFloat`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1588-1589`: `PrintConfig` declares `wipe_tower_bridging` as `ConfigOptionFloat` and `wipe_tower_extra_flow` as `ConfigOptionPercent`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1594-1596`: `PrintConfig` declares `wipe_tower_cone_angle`, `wipe_tower_extra_spacing`, and `wipe_tower_max_purge_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6718-6723`: `wipe_tower_rotation_angle` definition, degree unit, advanced mode, and default `0`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6736-6744`: `wipe_tower_cone_angle` definition, degree unit, bounds `0..=90`, and default `30`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6746-6757`: `wipe_tower_max_purge_speed` definition, minimum `10`, and default `90`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6872-6877`: `wipe_tower_bridging` definition and default `10`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6879-6886`: `wipe_tower_extra_spacing` percent definition, bounds `100..=300`, and default `100`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6888-6896`: `wipe_tower_extra_flow` percent definition, bounds `100..=300`, and default `100`.
- `OrcaSlicer/src/libslic3r/Print.cpp:267-269`: `wipe_tower_rotation_angle` invalidates Orca skirt/brim placement.
- `OrcaSlicer/src/libslic3r/Print.cpp:337-339` and `353-355`: Orca treats the other five options as wipe-tower reprocessing dependencies.
- `OrcaSlicer/src/libslic3r/Print.cpp:1001-1004`, `2836-2844`, and `3483`: Orca uses rotation and cone angle for wipe-tower placement and fake mesh data.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1467-1472`, `1489`, and `2366-2368`: legacy wipe-tower state keeps rotation, currently hardcodes bridging to `10`, uses `prime_tower_infill_gap` for extra spacing, and uses bridging while generating sparse grid lines.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1254-1262` and `1269`: `WipeTower2` copies rotation angle, cone angle, extra flow, extra spacing, bridging, and maximum purge speed into runtime state.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1927-1940`, `2018-2019`, `2084-2089`, and `2545-2552`: Orca uses extra flow/spacing in purge-line width/depth, max purge speed in feedrate selection, bridging in sparse-grid spacing, and cone angle in cone wall geometry.

## Current Ares Boundary

- Registry metadata for these six keys already exists in `crates/ares-core/src/options/registry/definitions/table/tail_terminal_wipe.rs` with source citations back to `PrintConfig.hpp` and `PrintConfig.cpp`.
- `crates/ares-core/src/options/filament_config_export.rs` is the current config-header snapshot boundary used by adjacent wipe-tower header options.
- `crates/ares-core/src/gcode_config_header.rs` appends the current wipe-tower config-header group after `filament_stamping_distance` and before `small_area_infill_flow_compensation_model`.
- `crates/ares-core/src/options/filament_config_export/serialization.rs` already owns scalar float and bounded scalar-float helpers that can validate these scalar values without adding a new parser module.
- Ares does not yet model Orca `WipeTower`, `WipeTower2`, wipe-tower purge planning, sparse wipe-tower layers, rotation-aware placement, cone wall geometry, bridge-grid spacing, purge-line spacing/flow adjustment, or max purge speed feedrate selection.

## Design

Extend `FilamentConfigExports` with optional header-export strings for:

```rust
pub(crate) wipe_tower_rotation_angle: Option<String>,
pub(crate) wipe_tower_bridging: Option<String>,
pub(crate) wipe_tower_extra_flow: Option<String>,
pub(crate) wipe_tower_cone_angle: Option<String>,
pub(crate) wipe_tower_extra_spacing: Option<String>,
pub(crate) wipe_tower_max_purge_speed: Option<String>,
```

Populate them from the existing `SliceOptions` values map:

- `wipe_tower_rotation_angle`: accept finite scalar floats with no upstream min/max and emit decimal text.
- `wipe_tower_bridging`: accept finite scalar floats with no upstream min/max and emit decimal text.
- `wipe_tower_extra_flow`: accept finite scalar numbers in `100.0..=300.0` and emit the percent value as decimal text.
- `wipe_tower_cone_angle`: accept finite scalar floats in `0.0..=90.0` and emit decimal text.
- `wipe_tower_extra_spacing`: accept finite scalar numbers in `100.0..=300.0` and emit the percent value as decimal text.
- `wipe_tower_max_purge_speed`: accept finite scalar floats with minimum `10.0` and emit decimal text.

Use the existing `optional_scalar_float_export()` for the two unbounded float definitions and `optional_scalar_float_export_with_bounds()` for the four bounded definitions. Do not add public API, CLI flags, WASM bindings, registry entries, dependencies, or new crates.

Append the six lines in `gcode_config_header.rs` immediately after `support_multi_bed_types` and before the already-consumed rib-wall group:

```text
; support_multi_bed_types = ...
; wipe_tower_rotation_angle = ...
; wipe_tower_bridging = ...
; wipe_tower_extra_flow = ...
; wipe_tower_cone_angle = ...
; wipe_tower_extra_spacing = ...
; wipe_tower_max_purge_speed = ...
; wipe_tower_wall_type = ...
```

The header order follows the upstream `PrintConfig.hpp` declaration order for the consumed slice, then preserves the existing rib-wall group and small-area flow model order after it.

Missing options emit no config line. Invalid values return `SliceError::InvalidInput` naming the offending key. Validation must still run before BTT thumbnail header suppression because `format_gcode()` consumes config exports before output bytes are returned.

## Alternatives Considered

- Consume only `wipe_tower_max_purge_speed`: rejected because the adjacent six-option scalar group shares the same header-export path and ordering boundary, and a one-key slice would force immediate reshuffling.
- Implement Orca `WipeTower2` speed/spacing/cone behavior now: rejected because Ares lacks the `WipeTower2` runtime planning and geometry boundary needed for source-compatible behavior.
- Add a separate percent serializer: rejected because the header format preserves percent values as ordinary decimal config values, and the existing bounded scalar-float helper is enough.
- Emit default values when options are omitted: rejected because existing Ares config-header exports only emit explicitly provided values.
- Invent lower bounds for `wipe_tower_rotation_angle` or `wipe_tower_bridging`: rejected because the cited upstream option definitions do not define those bounds.

## Behavior Included

- Header export and validation for the six already-registered scalar options.
- Finite numeric validation using the upstream max/min constraints listed above.
- Percent-valued header serialization for `wipe_tower_extra_flow` and `wipe_tower_extra_spacing` as the raw configured percent number.
- Missing-value behavior that preserves current output and omits the key.
- Invalid-value behavior that fails before G-code bytes are returned, including when normal header lines are skipped.

## Behavior Deferred

- Wipe-tower rotation-aware placement and collision checks.
- Wipe-tower cone base/corner construction and cone wall geometry.
- `WipeTower2` runtime state beyond validation/export.
- Purge-line width/depth changes from extra flow and extra spacing.
- Sparse wipe-tower grid bridging distance behavior.
- Max purge speed feedrate selection and sparse-layer speed fallback.
- Legacy `WipeTower` hardcoded bridging replacement.
- UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity.

## Docs Impact

Update `docs/roadmap.md` after implementation review with a source-cited runtime-slice entry for the six consumed wipe-tower speed/spacing config header options. No CLI/API documentation changes are required.

## Safety And Rollback

This is a header-export-only consumption slice. The only intended runtime effect is that explicit malformed values for the six keys fail with `SliceError::InvalidInput` before G-code bytes are returned, while explicit valid values gain config-header lines. It adds no wipe-tower geometry, purge planning, speed selection, public API, CLI, WASM, dependency, registry, or saturated-file changes. Rollback is limited to removing the six optional export fields, their population assignments, their six header append calls, the focused tests, and the roadmap entry.

## Acceptance Criteria

- G-code config-header tests prove explicit valid values emit:
  - `; wipe_tower_rotation_angle = -15.5`
  - `; wipe_tower_bridging = -2.25`
  - `; wipe_tower_extra_flow = 125`
  - `; wipe_tower_cone_angle = 45`
  - `; wipe_tower_extra_spacing = 150`
  - `; wipe_tower_max_purge_speed = 120`
- G-code config-header tests prove valid boundary values emit:
  - `; wipe_tower_extra_flow = 100`
  - `; wipe_tower_extra_flow = 300`
  - `; wipe_tower_cone_angle = 0`
  - `; wipe_tower_cone_angle = 90`
  - `; wipe_tower_extra_spacing = 100`
  - `; wipe_tower_extra_spacing = 300`
  - `; wipe_tower_max_purge_speed = 10`
- Header-order tests prove the six new lines appear after `support_multi_bed_types`, before `wipe_tower_wall_type`, and before `small_area_infill_flow_compensation_model`.
- Absence tests prove omitted speed/spacing group options emit no header lines.
- Invalid-value tests prove non-number inputs, nulls, `wipe_tower_extra_flow` below `100` or above `300`, `wipe_tower_cone_angle` below `0` or above `90`, `wipe_tower_extra_spacing` below `100` or above `300`, and `wipe_tower_max_purge_speed` below `10` return `SliceError::InvalidInput` naming the offending key.
- Invalid values are rejected even when BTT thumbnail settings skip normal header output.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` remain untouched and at 400 LOC or less.
- All touched Rust files, including `crates/ares-core/src/options/filament_config_export.rs`, `crates/ares-core/src/gcode_config_header.rs`, `crates/ares-core/src/tests/wipe_tower_config_header_gcode.rs`, and `crates/ares-core/src/tests/small_area_flow_model_header_gcode.rs`, remain at or below 400 LOC.
- `docs/roadmap.md` is updated after implementation review with the source-cited slice summary and deferred behavior.
- Verification passes with:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core wipe_tower_config_header`
  - `cargo nextest run -p ares-core small_area_flow_model_header`
  - `cargo nextest run --workspace`
