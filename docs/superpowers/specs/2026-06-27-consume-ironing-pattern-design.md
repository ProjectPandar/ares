# Consume Ironing Pattern Design

## Objective

Consume OrcaSlicer's existing `ironing_pattern` option into concrete ordinary Ironing path geometry in `ares-core`. This slice must move an already registered option from metadata/config parsing into observable slicing behavior; it must not add new option metadata or invent an Ares-owned ironing pipeline.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1137-1151` declares the ordinary ironing option group, including `ConfigOptionEnum<InfillPattern> ironing_pattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4178-4188` defines `ironing_pattern` as an enum with user values `rectilinear` and `concentric`, defaulting to `ipRectilinear`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1507-1518` stores `pattern`, `line_spacing`, `angle`, `fixed_angle`, and `inset` in `IroningParams`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1584-1600` copies the selected `config.ironing_pattern` into ironing parameters with spacing, inset, flow, speed, and angle.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1610-1629` recreates `Fill::new_from_type(f_pattern)` when the ironing pattern changes.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1693-1718` applies the selected filler spacing and emits `erIroning` extrusion paths.

## Ares Destination Boundary

- `crates/ares-core/src/options/ironing_type.rs` owns ordinary ironing runtime parsing for `ironing_type`, `ironing_inset`, `filament_ironing_inset`, `ironing_spacing`, and `filament_ironing_spacing`.
- `crates/ares-core/src/print_paths/ironing.rs` owns the current ordinary Ironing compatibility shell that duplicates eligible top/solid paths and generates spacing-driven rectangular rectilinear Ironing paths.
- `crates/ares-core/src/pipeline/tests/` owns path-level regression tests for option-to-path behavior.

## Included Behavior

1. Parse `ironing_pattern` in the ordinary ironing runtime parser, defaulting to `rectilinear` when omitted.
2. Accept exactly Orca's ordinary ironing enum values `rectilinear` and `concentric`.
3. Reject non-string or unknown `ironing_pattern` values with `SliceError::InvalidInput` whose message includes `ironing_pattern`.
4. Preserve current rectilinear behavior: a closed four-corner axis-aligned rectangular ordinary Ironing source path with positive spacing generates open horizontal Ironing line paths inside the already selected inset bounds.
5. For `ironing_pattern = "concentric"`, a closed four-corner axis-aligned rectangular ordinary Ironing source path with positive spacing generates closed rectangular concentric Ironing loops inside the already selected inset bounds. The outer loop uses the inset rectangle, and each subsequent loop steps inward by `ironing_spacing` while both X and Y extents remain positive.
6. Preserve source path metadata on generated Ironing paths: role `PrintPathRole::Ironing`, closed flag matching generated geometry, unsupported span, seam gap, and effective layer height.
7. Preserve existing zero-spacing and unsupported-shape behavior: zero spacing or non-eligible geometry duplicates the selected ordinary Ironing geometry once without pattern-specific fill expansion.
8. Keep `support_ironing_pattern` out of scope; ordinary `ironing_pattern` must not alter support-interface ironing geometry.

## Deferred Behavior

- Full Orca `Layer::make_ironing` polygon area collection, clipping, `FillConcentric`, and `FillRectilinear` parity.
- Non-rectangular concentric fill, holes, island chaining, path ordering, no-sort behavior, `link_max_length`, and exact extrusion-width calculation from filler spacing.
- `ironing_angle`, `ironing_angle_fixed`, `ironing_direction`, `solid_infill_rotate_template`, and rotation behavior.
- `ironing_expansion` and full offset/intersection behavior beyond the existing Ares inset compatibility shell.
- Multi-region/extruder grouping and current-extruder filament override selection beyond Ares' current first-value path.
- `support_ironing_pattern` and support-specific contact-layer polygon fill generation.
- Orca binary E2E geometry parity.

## Acceptance Criteria

- Omitting `ironing_pattern` with `ironing_type = "top"`, `ironing_inset = 0.5`, and `ironing_spacing = 1.0` over the existing 4 mm by 3 mm rectangular top-solid test fixture still emits three open rectilinear Ironing lines at Y coordinates `0.5`, `1.5`, and `2.5`.
- Explicit `ironing_pattern = "rectilinear"` emits the same three open lines for that fixture.
- `ironing_pattern = "concentric"`, `ironing_inset = 0.5`, and `ironing_spacing = 0.5` over the same fixture emits two closed loops:
  - `(0.5,0.5) -> (3.5,0.5) -> (3.5,2.5) -> (0.5,2.5)`
  - `(1.0,1.0) -> (3.0,1.0) -> (3.0,2.0) -> (1.0,2.0)`
- `ironing_pattern = "concentric"` with `ironing_spacing = 0` keeps the existing single closed inset rectangle duplicate.
- Invalid `ironing_pattern` values fail before path output with an error mentioning `ironing_pattern`.
- Ordinary `ironing_pattern` does not change support-interface ironing duplicate points.

## Verification

- TDD RED: `cargo nextest run -p ares-core ironing_pattern` fails before implementation because the pattern parser/path behavior is absent.
- Focused GREEN: `cargo nextest run -p ares-core ironing_pattern ironing_spacing ironing_type_paths support_ironing_spacing` passes after implementation.
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC guard confirms every touched Rust file is at or below 400 LOC.

## Safety And Documentation

This slice is local to `ares-core` option parsing and print-path generation. It adds no dependencies, no filesystem access, no terminal behavior, no UI behavior, and no non-WASM APIs. `docs/roadmap.md` must be updated after implementation to record the consumed runtime slice and to keep deferred upstream behavior explicit.
