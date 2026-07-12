# Consume Ironing Spacing Design

## Goal

Consume OrcaSlicer's `ironing_spacing` and first-value `filament_ironing_spacing` options in Ares' existing ordinary-ironing path generation so configured line spacing changes generated Ironing print-path coordinates and downstream G-code coordinates instead of remaining option metadata only.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1137-1151` declares ordinary Ironing options plus the filament-specific Ironing override group, including `ironing_spacing` and `filament_ironing_spacing`, in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3385-3395` defines `filament_ironing_spacing` as a nullable float-vector millimeter option with default `nil`, min `0`, and max `1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4202-4210` defines ordinary `ironing_spacing` as a millimeter option with default `0.1`, min `0`, and max `1`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1500-1725` implements `Layer::make_ironing`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1511-1512` describes `IroningParams::line_spacing` as the spacing of Ironing lines and an input to extrusion-flow calculation.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1584-1588` selects `filament_ironing_spacing[extruder_idx]` when non-nil and falls back to ordinary `ironing_spacing`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1693-1700` assigns the selected spacing to `f->spacing`, uses it for `link_max_length`, and folds it into the Ironing extrusion height / flow calculation before `Fill::fill_surface` generates `erIroning` paths.

## Current Ares Boundary

- `crates/ares-core/src/options/ironing_type.rs` parses `ironing_type`, ordinary `ironing_inset`, and first-value `filament_ironing_inset` into `OrdinaryIroningConfig`.
- `crates/ares-core/src/print_paths/ironing.rs` duplicates eligible Ares top/solid paths as `PrintPathRole::Ironing`, applies effective inset to two-point line paths and closed four-corner rectangle loops, and leaves non-eligible shapes as compatibility-shell duplicates.
- `crates/ares-core/src/print_paths/generate.rs` calls ordinary Ironing before support-interface Ironing.
- `crates/ares-core/src/options/ironing_flow.rs` and `crates/ares-core/src/options/speed.rs` already demonstrate the Ares first single-active-filament nullable override pattern for Ironing flow and speed.

## Included Behavior

- Add private parsing for ordinary `ironing_spacing` with Orca-compatible default `0.1` and range `0.0..=1.0`.
- Add private parsing for first-value `filament_ironing_spacing` alongside ordinary spacing:
  - accept scalar numeric/string values and array values;
  - use only the first value because Ares ordinary Ironing currently has one active filament/extruder path;
  - treat missing `filament_ironing_spacing`, first-value JSON `null`, and first-value string `"nil"` as fallback to ordinary `ironing_spacing`;
  - validate non-nil values as finite millimeters in Orca's `0.0..=1.0` range;
  - reject empty arrays and non-scalar/non-numeric values with `SliceError::InvalidInput`.
- Store the selected effective spacing in `OrdinaryIroningConfig`.
- Route spacing into ordinary Ironing path generation for Ares' existing closed four-corner, axis-aligned rectangle compatibility shell:
  - apply existing effective inset first;
  - generate open horizontal `PrintPathRole::Ironing` line paths from inset min-x to inset max-x;
  - start at inset min-y and add lines every selected spacing while the line y-coordinate remains within inset max-y;
  - mark generated spacing lines as open paths;
  - preserve source Ironing metadata that current ordinary duplicates preserve: unsupported span, seam gap, and effective layer height.
- Keep two-point source paths, unordered/crossed four-corner paths, repeated-corner paths, repeated first/last-point polygons, zero-width/height paths, and other non-rectangular deferred shapes on the existing single-duplicate behavior.
- Treat selected spacing `0` as a valid upstream boundary value but preserve the existing single inset rectangle duplicate instead of attempting zero-distance line generation.
- Preserve existing `ironing_type`, `ironing_inset`, `filament_ironing_inset`, Ironing speed/flow/fan/hardware, and support-interface Ironing behavior.
- Keep `ares-core` platform-neutral and WASM-compatible.

## Deferred Behavior

- Full Orca `Layer::make_ironing` polygon collection, top-surface union, `intersection_ex`, and `Fill::fill_surface` generation.
- Exact Orca fill start-point, alternating, monotonic sorting, link-length, line connection, and clipping behavior.
- Spacing-driven Ironing extrusion-height / `Flow::rounded_rectangle_extrusion_width_from_spacing` / `flow_mm3_per_mm` parity.
- `ironing_pattern`, `ironing_angle`, `ironing_angle_fixed`, `ironing_direction`, and full pattern-specific geometry.
- Multi-extruder current-filament selection beyond Ares' current first-value path.
- Support-specific `support_ironing_spacing`, support Ironing fill generation, and support-specific spacing/flow composition.
- Non-rectangular polygon offsetting, holes, expolygons, region grouping, and Orca binary E2E geometry parity.

## Acceptance Criteria

- With `ironing_type = "top"`, a closed 4 mm by 3 mm top-surface rectangle, `ironing_inset = 0.5`, and `ironing_spacing = 1.0`, ordinary Ironing emits three open line paths: `(0.5,0.5)->(3.5,0.5)`, `(0.5,1.5)->(3.5,1.5)`, and `(0.5,2.5)->(3.5,2.5)`.
- With the same rectangle and inset, `ironing_spacing = 0.5` emits five open line paths from y `0.5` through y `2.5`.
- With `filament_ironing_spacing = [0.5]` and ordinary `ironing_spacing = 1.0`, ordinary Ironing uses the filament override and emits the five-line result.
- With `filament_ironing_spacing = ["nil", 0.5]` and ordinary `ironing_spacing = 1.0`, ordinary Ironing falls back to the ordinary spacing and emits the three-line result.
- With `filament_ironing_spacing = [null]` and ordinary `ironing_spacing = 1.0`, ordinary Ironing falls back to the ordinary spacing and emits the three-line result, matching the existing registry default representation for nullable filament Ironing spacing.
- Invalid ordinary or filament spacing values outside `0.0..=1.0`, non-numeric values, non-finite values, empty arrays, and non-scalar containers return `SliceError::InvalidInput` before G-code formatting succeeds.
- Selected spacing `0` remains accepted and keeps the existing single inset rectangle duplicate, preventing zero-distance line generation.
- Ordinary `ironing_spacing` and `filament_ironing_spacing` do not change support-interface Ironing duplicate coordinates.

## Verification

- Use TDD with `cargo nextest run -p ares-core ironing_spacing` for the new focused tests.
- Run adjacent focused regression coverage with `cargo nextest run -p ares-core ironing_inset filament_ironing_inset ironing_type_paths support_ironing_paths`.
- Before commit, run:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard

## Docs Impact

Update `docs/roadmap.md` with a runtime slice entry after implementation review approval. The roadmap entry must cite the same upstream boundary, state ordinary plus first-value nullable filament spacing behavior, describe Ares' rectangle-line compatibility-shell generation, and keep full Orca Ironing fill parity deferred.
