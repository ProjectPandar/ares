# Consume Filament Ironing Inset Design

## Goal

Consume OrcaSlicer's `filament_ironing_inset` option in Ares' existing ordinary-ironing path generation so the filament-specific nullable inset override changes generated Ironing print paths and downstream G-code coordinates.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1147-1151` declares the filament-specific ironing override group, including `filament_ironing_inset`, in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3397-3407` defines `filament_ironing_inset` as a nullable float-vector millimeter option with default `nil`, min `0`, max `100`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4212-4220` defines the ordinary `ironing_inset` fallback as a millimeter option where `0` means half the nozzle diameter.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1584-1591` selects `filament_ironing_inset[extruder_idx]` when non-nil and falls back to ordinary `ironing_inset` when nil.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1687-1689` applies the effective ironing inset by offsetting top-surface polygons inward by `ironing_inset`, or by `0.5 * nozzle_diameter` when the effective value is zero.

## Current Ares Boundary

- `crates/ares-core/src/options/ironing_type.rs` parses `ironing_type` plus ordinary `ironing_inset` into `OrdinaryIroningConfig`.
- `crates/ares-core/src/print_paths/ironing.rs` duplicates eligible Ares paths as `PrintPathRole::Ironing`, shortens two-point duplicates by the effective inset, insets closed four-corner rectangle loops, drops collapsed duplicates, and keeps non-eligible shapes unchanged.
- `crates/ares-core/src/print_paths/generate.rs` calls ordinary ironing before support-interface ironing.
- `crates/ares-core/src/options/ironing_flow.rs` already demonstrates Ares' first single-active-filament nullable override pattern for `filament_ironing_flow`.
- `crates/ares-core/src/options/speed.rs` already demonstrates Ares' first single-active-filament nullable override pattern for `filament_ironing_speed`.

## Included Behavior

- Add private parsing for `filament_ironing_inset` alongside ordinary `ironing_inset`.
- Accept scalar and array forms, using only the first value because Ares currently has a single active filament/extruder path in ordinary ironing.
- Treat missing `filament_ironing_inset` and first-value `nil` as fallback to ordinary `ironing_inset`.
- Validate non-nil `filament_ironing_inset` values as finite millimeters in Orca's `0.0..=100.0` range.
- Resolve the effective ordinary Ironing inset as:
  - first non-nil `filament_ironing_inset` value when present;
  - otherwise ordinary `ironing_inset`;
  - then `0.5 * first nozzle_diameter` when the selected value is zero.
- Apply the selected effective inset through Ares' existing ordinary Ironing line/rectangle coordinate behavior.
- Preserve `ironing_type` gating, support-interface ironing behavior, and the existing ordinary `ironing_inset` fallback behavior.
- Keep `ares-core` platform-neutral and WASM-compatible.

## Deferred Behavior

- Multi-extruder current-filament selection beyond Ares' current first-value path.
- Filament-specific `filament_ironing_spacing` and full ironing spacing/pattern path generation.
- Full Orca `Layer::make_ironing` polygon collection, union, `intersection_ex`, and `Fill::fill_surface` generation.
- `ironing_pattern`, `ironing_spacing`, `ironing_angle`, `ironing_angle_fixed`, and `ironing_expansion`.
- Non-rectangular polygon offsetting, holes, expolygons, region grouping, and Orca binary E2E geometry parity.
- Support ironing inset behavior.

## Acceptance Criteria

- With `filament_ironing_inset = [0.1]` and ordinary `ironing_inset = 0.4`, ordinary top-surface Ironing line duplicates are inset by `0.1` mm, not `0.4` mm.
- With scalar string `filament_ironing_inset = "0.2"`, ordinary top-surface Ironing line duplicates are inset by `0.2` mm.
- With `filament_ironing_inset = ["nil", 0.1]` and ordinary `ironing_inset = 0.4`, ordinary Ironing duplicates use the ordinary `0.4` mm fallback.
- With `filament_ironing_inset = 0` and first nozzle diameter `0.6`, ordinary Ironing duplicates use `0.3` mm.
- Invalid non-nil `filament_ironing_inset` values outside `0.0..=100.0`, non-numeric values, non-finite values, empty arrays, and non-scalar containers return `SliceError::InvalidInput` before G-code formatting succeeds.
- Support-interface ironing duplicates are unchanged by `filament_ironing_inset`.

## Verification

- Use TDD with `cargo nextest run -p ares-core filament_ironing_inset` for the new focused tests.
- Run regression coverage for adjacent ordinary ironing with `cargo nextest run -p ares-core ironing_inset ironing_flow ironing_speed`.
- Before commit, run:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard

## Docs Impact

Update `docs/roadmap.md` with a runtime slice entry after implementation review approval. The roadmap entry must cite the same upstream boundary, state the first-value nullable filament override behavior, and keep full Orca multi-extruder/current-filament plus full ironing fill parity deferred.
