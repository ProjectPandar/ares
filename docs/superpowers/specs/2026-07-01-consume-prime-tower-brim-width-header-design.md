# Consume prime tower brim width header design

## Goal

Consume the already-registered Orca `prime_tower_brim_width` option through Ares' existing G-code config header export path, validating malformed or out-of-range values before G-code bytes are returned and preserving the current no-placement behavior. This slice must not implement automatic brim-width calculation, wipe-tower geometry, cone/rib wall generation, purge-depth planning, collision checks, fake wipe-tower mesh state, or adjacent prime-tower interface options.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1581-1584`: `PrintConfig` declares `wipe_tower_rotation_angle`, `prime_tower_brim_width`, `prime_tower_infill_gap`, and `prime_tower_skip_points` in this order.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6725-6734`: `prime_tower_brim_width` is a `coFloat`, GUI enum-open control, label `Brim width`, sidetext `mm`, minimum `-1`, enum value `-1` labelled `Auto`, default `3`, and describes negative values as auto calculated from prime-tower height.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7878-7879`: legacy `wipe_tower_brim_width` is renamed to `prime_tower_brim_width`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5574`: Orca's G-code path appends configured options to the generated config header.
- `OrcaSlicer/src/libslic3r/Print.cpp:318-323`: changing `prime_tower_brim_width` invalidates Orca's wipe-tower processing step.
- `OrcaSlicer/src/libslic3r/Print.cpp:3150,3177-3179`: Orca stores explicit brim width on wipe-tower data and uses `WipeTower::get_auto_brim_by_height(max_height)` when the configured brim width is negative.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1461-1468,3705-3707`: legacy `WipeTower` stores `prime_tower_brim_width` as `m_wipe_tower_brim_width` and replaces negative values with the auto height-derived brim width during tower planning.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1248-1256,2115-2119,2134-2136`: `WipeTower2` stores `prime_tower_brim_width` as runtime brim-width state, replaces negative values with `WipeTower::get_auto_brim_by_height(m_wipe_tower_height)` while generating the first-layer brim, and records the actual generated brim width for later print/preview state.

## Ares destination boundary

- Registry metadata for `prime_tower_brim_width` already exists in `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs` with default `3`.
- `crates/ares-core/src/options/legacy.rs` already maps legacy `wipe_tower_brim_width` to `prime_tower_brim_width`.
- The Rust destination is the existing `FilamentConfigExports` snapshot in `crates/ares-core/src/options/filament_config_export.rs` plus `crates/ares-core/src/gcode_config_header.rs` serialization.
- Focused tests belong in a new `crates/ares-core/src/tests/prime_tower_brim_width_header_gcode.rs` module to avoid growing saturated or near-saturated existing files.

## Design

Add one optional export field:

```rust
pub(crate) prime_tower_brim_width: Option<String>,
```

Populate it from `SliceOptions::values().get("prime_tower_brim_width")` using the existing scalar float bounded helper with `min = Some(-1.0)` and `max = None`. The helper must:

- Return `Ok(None)` when the key is absent.
- Format valid finite scalar values with Ares' existing decimal formatting.
- Accept `-1` as Orca's auto sentinel.
- Reject values below `-1`, strings, arrays, objects, bools, nulls, and non-finite values with `SliceError::InvalidInput` naming `prime_tower_brim_width`.

Append the optional header line in upstream-adjacent order:

```text
; wipe_tower_rotation_angle = ...
; prime_tower_brim_width = ...
; wipe_tower_bridging = ...
```

This follows upstream declaration/key order around `wipe_tower_rotation_angle`, `prime_tower_brim_width`, `prime_tower_infill_gap`, and `prime_tower_skip_points`, while intentionally leaving `prime_tower_infill_gap`, `prime_tower_skip_points`, and all auto-brim geometry behavior for later source-cited slices.

## Rejected alternatives

- Apply negative `prime_tower_brim_width` to generated tower geometry now: rejected because Orca's negative sentinel is resolved in `Print`, `WipeTower`, and `WipeTower2` runtime planning, which is outside this header-export consumption slice.
- Reuse the existing `wipe_tower_config_header_gcode.rs` tests: rejected because that file is already near the repository's 400 LOC split threshold and the new behavior is a separable option slice.
- Add a new serializer helper: rejected because `optional_scalar_float_export_with_bounds` already captures the exact finite scalar plus lower-bound behavior needed here.

## Included behavior

- Header export and validation for already-registered `prime_tower_brim_width`.
- Legacy `wipe_tower_brim_width` input reaching the same `prime_tower_brim_width` header export and validation path through the existing Ares legacy normalization.
- `-1` auto sentinel serialization as a literal config-header value, without resolving it to an auto width.
- Missing-value behavior that preserves current output and emits no `prime_tower_brim_width` line.
- Validation before G-code bytes are returned, including when BTT thumbnail header generation would otherwise skip the config header.

## Deferred behavior

- Automatic brim-width calculation via `WipeTower::get_auto_brim_by_height`.
- Wipe-tower placement, fake wipe-tower state, collision checks, cone/corner geometry, wall generation, mesh construction, purge-depth planning, rib-wall width recomputation, and legacy `WipeTower` / `WipeTower2` runtime brim-width behavior.
- `prime_tower_infill_gap`, `prime_tower_skip_points`, `prime_tower_flat_ironing`, `prime_tower_enable_framework`, interface feature options, flush-volume behavior, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity.

## Documentation

Update `docs/roadmap.md` after implementation review with a source-cited runtime-slice entry for consumed `prime_tower_brim_width` config header validation/output and deferred auto-brim/geometry behavior. No CLI/API documentation changes are required.

## Risk and rollback

This is a header-export-only consumption slice. The only intended runtime effect is that explicit malformed or below-minimum `prime_tower_brim_width` values fail with `SliceError::InvalidInput` before G-code bytes are returned, while explicit valid values gain one config-header line. It adds no wipe-tower placement, public API, CLI, WASM, dependency, registry, or saturated-file changes. Rollback is limited to removing the optional export field, population assignment, header append call, focused tests, and roadmap entry.

## Acceptance criteria

- G-code config-header tests prove explicit valid values emit scalar lines such as `; prime_tower_brim_width = 3` and `; prime_tower_brim_width = 4.5`.
- G-code config-header tests prove Orca's auto sentinel emits `; prime_tower_brim_width = -1`.
- Header-order tests prove `prime_tower_brim_width` appears after `wipe_tower_rotation_angle` and before `wipe_tower_bridging`.
- Absence tests prove omitted `prime_tower_brim_width` emits no header line.
- Legacy-alias tests prove `wipe_tower_brim_width` normalizes to `prime_tower_brim_width` and emits the same header line.
- Invalid-value tests prove strings, arrays, bools, objects, nulls, non-finite values, and numeric values below `-1` return `SliceError::InvalidInput` naming `prime_tower_brim_width`.
- BTT-header-skip tests prove invalid values are still rejected even when config header output is skipped.
- The saturated files `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` remain untouched at 400 LOC.
- `crates/ares-core/src/tests/wipe_tower_config_header_gcode.rs` remains untouched.
- Final verification includes:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core prime_tower_brim_width_header`
  - `cargo nextest run -p ares-core prime_tower_width_header`
  - `cargo nextest run -p ares-core wipe_tower_config_header`
  - `cargo nextest run --workspace`
