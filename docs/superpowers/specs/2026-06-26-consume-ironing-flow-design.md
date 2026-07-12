# Consume Ironing Flow Design

## Purpose

Consume the existing OrcaSlicer `ironing_flow` and `filament_ironing_flow` options into concrete Ares slicing output by making the existing `PrintPathRole::Ironing` extrusion delta use an ironing-specific flow multiplier. This continues the current ironing option chain after `ironing_speed` and `filament_ironing_speed`; it must not add new option metadata or invent a separate Ares ironing pipeline.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1137-1151` declares the PrintRegion ironing option group, including `ironing_flow`, and the filament ironing override group, including `filament_ironing_flow`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3372-3383` defines `filament_ironing_flow` as nullable percents, range `0..=100`, default `nil`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4190-4200` defines `ironing_flow` as a percent, range `0..=100`, default `10`, with `ratio_over = "layer_height"`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1584-1597` chooses a filament-specific override when present, otherwise uses `ironing_flow`, and computes ironing height as `default_layer_height * 0.01 * percent`.

## Ares Destination Boundary

- Add an independent ironing flow multiplier to `crates/ares-core/src/extrusions/options.rs`.
- Route `PrintPathRole::Ironing` through that multiplier in `crates/ares-core/src/extrusions/options/accessors.rs` instead of reusing `top_solid_infill_flow_ratio`.
- Parse `ironing_flow` and `filament_ironing_flow` from `SliceOptions` in a focused options helper, then wire it from `crates/ares-core/src/options/flow_ratios.rs`.
- Add focused pipeline tests for Ironing G-code `E` deltas in `crates/ares-core/src/pipeline/tests/ironing_flow.rs`.

## Included Behavior

- Missing `ironing_flow` uses Orca's default `10%`, so existing non-first-layer Ironing role extrusion uses a `0.10` flow multiplier.
- `ironing_flow` accepts numeric and numeric-string values in `0..=100` and maps them to `0.0..=1.0`.
- `filament_ironing_flow` accepts scalar or array numeric/numeric-string values in `0..=100`; the first value is used for the current single-filament Ares boundary.
- `filament_ironing_flow` value `"nil"` falls back to `ironing_flow`.
- Invalid values for either key return `SliceError::InvalidInput` mentioning the offending key.
- Ironing extrusion becomes independent from `top_solid_infill_flow_ratio`.
- Existing `filament_flow_ratio`, `print_flow_ratio`, and first-layer flow composition continue to apply after the role-specific ironing multiplier.

## Deferred Behavior

- Full Orca `Fill::make_ironing` path generation, including `ironing_spacing`, `ironing_inset`, `ironing_pattern`, `ironing_direction`, `ironing_angle`, and `ironing_angle_fixed`.
- Current-extruder indexed filament override selection beyond Ares' existing first single-filament boundary.
- Support ironing flow (`support_ironing_flow`) and support ironing path generation.
- Orca binary end-to-end geometric parity for generated ironing paths.

## Acceptance Criteria

- A RED focused run with `cargo nextest run -p ares-core ironing_flow` fails before implementation because Ironing `E` deltas still use the old top solid flow path.
- A GREEN focused run with `cargo nextest run -p ares-core ironing_flow` passes after implementation.
- `cargo nextest run -p ares-core ironing_flow ironing_speed` passes to show the new flow slice does not regress existing ironing speed behavior.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard, with every touched Rust file at or below 400 LOC.

## Safety

The change is confined to platform-neutral `ares-core` parsing and extrusion math. It adds no filesystem, terminal, UI, OpenGL, network, or native-only behavior. It adds no new dependencies and no compatibility fallback.
