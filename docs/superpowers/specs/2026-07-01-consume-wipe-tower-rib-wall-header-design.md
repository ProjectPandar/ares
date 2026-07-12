# Consume Wipe Tower Rib Wall Header Design

## Goal

Consume the already-registered Orca wipe-tower rib-wall option group through Ares' existing G-code config header export path, validating malformed values before G-code bytes are returned, without implementing wipe-tower wall geometry or perimeter-filament selection.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:405-408`: `WipeTowerWallType` declares `wtwRectangle`, `wtwCone`, and `wtwRib`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:558-563`: `WipeTowerWallType` maps the config keys `"rectangle"`, `"cone"`, and `"rib"`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1597-1601`: `PrintConfig` declares `wipe_tower_wall_type`, `wipe_tower_extra_rib_length`, `wipe_tower_rib_width`, `wipe_tower_fillet_wall`, and `wipe_tower_filament`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6759-6808`: option definitions, labels, bounds, defaults, and enum values for the rib-wall option group.
- `OrcaSlicer/src/libslic3r/Print.cpp:353-360`: Orca treats these options as wipe-tower reprocessing dependencies.
- `OrcaSlicer/src/libslic3r/Print.cpp:3363-3364` and `3474-3478`: Orca passes wall type, rib width/length, and fillet state into wipe-tower mesh construction.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1364-1416` and `1478-1488`: Orca uses rib length, rib width, and fillet state while constructing rib-wall geometry.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1262-1274`: Orca copies the related wipe-tower runtime config into `WipeTower2` state.

## Current Ares Boundary

- Registry metadata for these five keys already exists in `crates/ares-core/src/options/registry/definitions/table/tail_terminal_wipe.rs` with source citations back to `PrintConfig.hpp` and `PrintConfig.cpp`.
- `crates/ares-core/src/options/filament_config_export.rs` is the existing config-header snapshot boundary used by adjacent wipe-tower header options.
- `crates/ares-core/src/gcode_config_header.rs` appends the current wipe-tower config-header group after `filament_stamping_distance`.
- `crates/ares-core/src/options/filament_config_export/serialization.rs` already owns scalar bool, scalar float, vector, string, and wipe-tower enum header serialization helpers.
- Ares does not yet model Orca `WipeTower`, `WipeTower2`, wipe-tower mesh construction, rib-wall geometry, wipe-tower tool ordering, or wipe-tower perimeter-filament selection.

## Design

Extend `FilamentConfigExports` with optional header-export strings for:

```rust
pub(crate) wipe_tower_wall_type: Option<String>,
pub(crate) wipe_tower_extra_rib_length: Option<String>,
pub(crate) wipe_tower_rib_width: Option<String>,
pub(crate) wipe_tower_fillet_wall: Option<String>,
pub(crate) wipe_tower_filament: Option<String>,
```

Populate them from the existing `SliceOptions` values map:

- `wipe_tower_wall_type`: accept only `"rectangle"`, `"cone"`, or `"rib"` and emit the key unchanged.
- `wipe_tower_extra_rib_length`: accept finite scalar floats with maximum `300.0`; negative values are valid because upstream sets no minimum.
- `wipe_tower_rib_width`: accept finite scalar floats in `0.0..=300.0`.
- `wipe_tower_fillet_wall`: accept a scalar bool and emit `1` or `0`.
- `wipe_tower_filament`: accept scalar integers in `0..=i32::MAX` and emit the integer as decimal text.

Add focused serialization helpers only where existing helpers do not cover the upstream type and bound:

```rust
optional_wipe_tower_wall_type_export(value)
optional_scalar_float_export_with_bounds(value, key, min: Option<f64>, max: Option<f64>)
optional_scalar_int_export_in_range(value, key, min, max)
```

Use `min: None, max: Some(300.0)` for `wipe_tower_extra_rib_length` and `min: Some(0.0), max: Some(300.0)` for `wipe_tower_rib_width`.

Keep helper visibility at `pub(super)` or private inside the existing serialization module. Do not add public API, CLI flags, WASM bindings, registry entries, dependencies, or new crates.

Append the five lines in `gcode_config_header.rs` immediately after `support_multi_bed_types` and before `small_area_infill_flow_compensation_model`:

```text
; support_multi_bed_types = ...
; wipe_tower_wall_type = ...
; wipe_tower_extra_rib_length = ...
; wipe_tower_rib_width = ...
; wipe_tower_fillet_wall = ...
; wipe_tower_filament = ...
; small_area_infill_flow_compensation_model = ...
```

Missing options emit no config line. Invalid values return `SliceError::InvalidInput` naming the offending key. Validation must still run before BTT thumbnail header suppression because `format_gcode()` consumes config exports before output bytes are returned.

## Alternatives Considered

- Consume only `wipe_tower_fillet_wall`: rejected because the upstream rib-wall group is adjacent, already registered, and would immediately force a later header-order reshuffle.
- Implement Orca rib-wall geometry now: rejected because Ares lacks the `WipeTower` and `WipeTower2` runtime boundaries needed to make geometry behavior source-compatible.
- Emit default values when options are omitted: rejected because existing Ares config-header exports only emit explicitly provided values.
- Accept out-of-range floats and defer bounds: rejected because upstream definitions encode the range constraints used by this config surface.

## Behavior Included

- Header export and validation for the five already-registered rib-wall group options.
- Orca enum key preservation for `wipe_tower_wall_type`.
- Orca-style scalar bool serialization for `wipe_tower_fillet_wall`.
- Finite numeric validation using the upstream max/min constraints listed above.
- Missing-value behavior that preserves current output and omits the key.
- Invalid-value behavior that fails before G-code bytes are returned, including when normal header lines are skipped.

## Behavior Deferred

- Wipe-tower wall shape selection in path planning.
- Rib-wall and cone geometry generation.
- Fillet application in `WipeTower::rib_section`.
- Wipe-tower mesh construction through `WipeTowerData::construct_mesh`.
- `WipeTower2` runtime state and purge generation.
- Wipe-tower perimeter-filament selection and non-soluble preference.
- Legacy `prime_tower_*` key migration beyond existing registry behavior.
- UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity.

## Docs Impact

Update `docs/roadmap.md` after implementation review with a source-cited runtime-slice entry for the five consumed rib-wall config header options. No CLI/API documentation changes are required.

## Acceptance Criteria

- G-code config-header tests prove explicit valid values emit:
  - `; wipe_tower_wall_type = rectangle`
  - `; wipe_tower_extra_rib_length = -12.5`
  - `; wipe_tower_rib_width = 8.25`
  - `; wipe_tower_fillet_wall = 1`
  - `; wipe_tower_filament = 3`
- G-code config-header tests prove `wipe_tower_fillet_wall = false` emits `; wipe_tower_fillet_wall = 0`.
- Header-order tests prove the new lines appear after `support_multi_bed_types` and before `small_area_infill_flow_compensation_model`.
- Absence tests prove omitted rib-wall group options emit no header lines.
- Invalid-value tests prove invalid enum keys, non-boolean fillet values, non-number rib float inputs, out-of-range rib float values, negative filament indexes, non-integer filament values, and null values return `SliceError::InvalidInput` naming the offending key. The finite-float guard remains in production code for internal safety, but JSON boundary tests use representable malformed values such as strings, arrays, nulls, and numeric values outside the upstream ranges.
- Invalid values are rejected even when BTT thumbnail settings skip normal header output.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` remain untouched and at 400 LOC or less.
- All touched Rust files, including `crates/ares-core/src/options/filament_config_export.rs`, `crates/ares-core/src/options/filament_config_export/serialization.rs`, `crates/ares-core/src/gcode_config_header.rs`, and `crates/ares-core/src/tests/wipe_tower_config_header_gcode.rs`, remain at or below 400 LOC.
- `docs/roadmap.md` is updated after implementation review with the source-cited slice summary and deferred behavior.
- Verification passes with:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core wipe_tower_config_header`
  - `cargo nextest run --workspace`
