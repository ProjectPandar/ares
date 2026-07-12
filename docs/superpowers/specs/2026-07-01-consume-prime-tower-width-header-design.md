# Consume Prime Tower Width Header Design

## Goal

Consume the already-registered Orca `prime_tower_width` option through Ares' existing G-code config header export path, validating malformed or out-of-range values before G-code bytes are returned and preserving the current no-placement behavior. This slice must not implement wipe-tower geometry, purge-depth planning, rib-wall width recomputation, collision checks, plate placement, or the obsolete `wipe_tower_per_color_wipe` key.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1577-1581`: `PrintConfig` declares `wipe_tower_x`, `wipe_tower_y`, `prime_tower_width`, obsolete `wipe_tower_per_color_wipe`, and `wipe_tower_rotation_angle` in this order in the repository's existing source-citation baseline.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6710-6716`: `prime_tower_width` is a `coFloat`, label `Width`, sidetext `mm`, minimum `2.0`, default `60.`, and `comSimple`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7874-7875`: legacy `wipe_tower_width` is renamed to `prime_tower_width`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8069-8074`: `wipe_tower_per_color_wipe` is explicitly ignored as an obsolete configuration key.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5574`: G-code config header serialization writes every non-banned, non-nil key using `cfg.opt_serialize(key)`, with only `wipe_tower_x`, `wipe_tower_y`, and `extruder_colour` special-cased.
- `OrcaSlicer/src/libslic3r/Print.cpp:318-325`: changing `prime_tower_width` invalidates Orca's wipe-tower processing step.
- `OrcaSlicer/src/libslic3r/Print.cpp:1002-1009`: Orca reads `config.prime_tower_width.value` when computing wipe-tower collision geometry with plate-origin-adjusted coordinates.
- `OrcaSlicer/src/libslic3r/Print.cpp:2838`: Orca uses `prime_tower_width` to compute wipe-tower cone/corner geometry.
- `OrcaSlicer/src/libslic3r/Print.cpp:3154` and `3168-3170`: Orca uses `prime_tower_width` as the divisor when estimating wipe-tower depth from purge volume.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1461-1468`: legacy `WipeTower` stores `prime_tower_width` as `m_wipe_tower_width`.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:3707-3716`: legacy `WipeTower` may recompute `m_wipe_tower_width` for rib-wall planning.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1248-1254`: `WipeTower2` stores `prime_tower_width` as `m_wipe_tower_width`.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:2024,2103,2227,2299`: `WipeTower2` uses the stored width in wipe-box and wipe-length planning.

## Current Ares Boundary

- Registry metadata for `prime_tower_width` already exists in `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs` with the current Ares source citation and default.
- `crates/ares-core/src/options/legacy.rs` already maps legacy `wipe_tower_width` to `prime_tower_width` and ignores obsolete `wipe_tower_per_color_wipe`, matching the upstream legacy/obsolete behavior.
- `crates/ares-core/src/options/filament_config_export.rs` is the current config-header snapshot boundary used by adjacent wipe-tower header options.
- `crates/ares-core/src/gcode_config_header.rs` appends the current wipe-tower config-header group.
- `crates/ares-core/src/options/filament_config_export/serialization.rs` already has scalar float validation with optional bounds.
- `crates/ares-core/src/gcode.rs:58` calls `options.filament_config_exports()?` before `crates/ares-core/src/gcode_file_start.rs:27` can skip normal header output for BTT thumbnail settings, so export validation still runs when the header is suppressed.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` are saturated at 400 LOC and must remain untouched.
- `crates/ares-core/src/tests/wipe_tower_config_header_gcode.rs` is near the 400 LOC split threshold and must remain untouched for this slice.
- Ares does not yet model Orca wipe-tower placement, fake wipe-tower state, collision checks, purge-depth planning, cone/corner geometry, legacy `WipeTower`, or `WipeTower2`.

## Design

Extend `FilamentConfigExports` with:

```rust
pub(crate) prime_tower_width: Option<String>,
```

Populate it from `SliceOptions::values().get("prime_tower_width")` using the existing scalar float bounded helper with `min = Some(2.0)` and `max = None`. The helper must:

- Accept finite JSON numeric scalar values at or above `2.0`.
- Reject missing? No; missing returns `Ok(None)` and emits no header line.
- Reject strings, arrays, objects, bools, nulls, non-finite values, and values below `2.0` with `SliceError::InvalidInput` naming `prime_tower_width`.
- Serialize valid values with existing trimmed decimal formatting, matching the current Ares config-header scalar convention and Orca's generic `opt_serialize` header path for non-special-cased scalar keys.

Append the header line immediately after `wipe_tower_y` and before `wipe_tower_rotation_angle`:

```text
; wipe_tower_x = ...
; wipe_tower_y = ...
; prime_tower_width = ...
; wipe_tower_rotation_angle = ...
```

This follows upstream declaration/key order around `wipe_tower_x`, `wipe_tower_y`, `prime_tower_width`, obsolete `wipe_tower_per_color_wipe`, and `wipe_tower_rotation_angle`, while intentionally skipping `wipe_tower_per_color_wipe` because Orca ignores it as obsolete during config normalization and Ares already mirrors that behavior.

## Alternatives Considered

- Implement `wipe_tower_per_color_wipe`: rejected because upstream ignores the key as obsolete in `PrintConfig.cpp:8069-8074`, and current Ares already ignores it in legacy normalization.
- Add a new dedicated scalar helper: rejected because existing bounded scalar float export already captures the required type, finite, min-bound, and decimal serialization behavior.
- Apply `prime_tower_width` to generated geometry now: rejected because Orca's width affects `Print`, `WipeTower`, and `WipeTower2` runtime planning that are outside this header-export consumption slice.
- Invent a maximum bound: rejected because the cited Orca definition specifies only `min = 2.0`.
- Modify the registry or legacy mapping: rejected because both already exist for this option and this slice consumes the registered option, not redefines it.

## Behavior Included

- Header export and validation for already-registered `prime_tower_width`.
- Legacy `wipe_tower_width` input reaching the same `prime_tower_width` header export and validation path through the existing Ares legacy normalization.
- Finite scalar numeric validation with Orca's `2.0` lower bound and no invented upper bound.
- Header serialization through Ares' existing scalar float config format.
- Header ordering between selected wipe-tower coordinates and `wipe_tower_rotation_angle`.
- Missing-value behavior that preserves current output and emits no `prime_tower_width` line.
- Invalid-value behavior that fails before G-code bytes are returned, including when normal header lines are skipped.

## Behavior Deferred

- Wipe-tower placement, fake wipe-tower state, and collision checks.
- Wipe-tower cone/corner geometry and rotation-aware placement.
- Wipe-tower depth planning from purge volume, layer height, and tower width.
- Legacy `WipeTower` and `WipeTower2` runtime width state.
- Rib-wall width recomputation and width/depth square balancing.
- `prime_tower_brim_width`, `prime_tower_infill_gap`, `prime_tower_skip_points`, `prime_tower_flat_ironing`, `wiping_volumes_extruders`, flush-volume behavior, and other adjacent wipe-tower runtime behavior.
- Obsolete `wipe_tower_per_color_wipe` behavior.
- UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity.

## Docs Impact

Update `docs/roadmap.md` after implementation review with a source-cited runtime-slice entry for consumed `prime_tower_width` config header validation/output and deferred geometry behavior. No CLI/API documentation changes are required.

## Safety And Rollback

This is a header-export-only consumption slice. The only intended runtime effect is that explicit malformed or below-minimum `prime_tower_width` values fail with `SliceError::InvalidInput` before G-code bytes are returned, while explicit valid values gain one config-header line. It adds no wipe-tower placement, collision checking, public API, CLI, WASM, dependency, registry, or saturated-file changes. Rollback is limited to removing the optional export field, population assignment, header append call, focused tests, and roadmap entry.

## Acceptance Criteria

- G-code config-header tests prove explicit valid values emit scalar lines such as `; prime_tower_width = 60` and `; prime_tower_width = 2`.
- G-code config-header tests prove decimal values keep Ares' scalar config formatting, such as `; prime_tower_width = 12.5`.
- Header-order tests prove `prime_tower_width` appears after `wipe_tower_y` and before `wipe_tower_rotation_angle`.
- Absence tests prove omitted `prime_tower_width` emits no header line.
- Legacy-alias tests prove `wipe_tower_width` normalizes to `prime_tower_width` and emits the same header line.
- Invalid-value tests prove strings, arrays, bools, objects, nulls, and numeric values below `2.0` return `SliceError::InvalidInput` naming `prime_tower_width`.
- Invalid values are rejected even when BTT thumbnail settings skip normal header output.
- `wipe_tower_per_color_wipe` remains untouched and ignored as obsolete.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, `crates/ares-core/src/gcode.rs`, and `crates/ares-core/src/tests/wipe_tower_config_header_gcode.rs` remain untouched.
- All touched Rust files remain at or below 400 LOC.
- `docs/roadmap.md` is updated after implementation review with the source-cited slice summary and deferred behavior.
- Verification passes with:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core prime_tower_width_header`
  - `cargo nextest run -p ares-core wipe_tower_coordinate_header`
  - `cargo nextest run -p ares-core wipe_tower_config_header`
  - `cargo nextest run --workspace`
