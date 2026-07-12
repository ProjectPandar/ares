# Consume Ironing Inset Design

## Goal

Consume OrcaSlicer's `ironing_inset` option in Ares' existing ordinary-ironing path generation so configured inset values affect generated Ironing print paths and downstream G-code coordinates.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1142` declares `ironing_inset` in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4212-4220` defines `ironing_inset` as a `coFloat` millimeter option with default `0`, min `0`, max `100`, and the upstream rule that `0` means half the nozzle diameter.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1501-1720` implements `Layer::make_ironing`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1687-1689` applies the effective ironing inset by offsetting top-surface polygons inward by `ironing_inset`, or by `0.5 * nozzle_diameter` when configured inset is zero.

## Current Ares Boundary

- `crates/ares-core/src/options/ironing_type.rs` parses `ironing_type`.
- `crates/ares-core/src/options/ironing_flow.rs` parses ironing flow ratios.
- `crates/ares-core/src/print_paths/ironing.rs` currently duplicates eligible top/solid Ares print paths as `PrintPathRole::Ironing` without changing their geometry.
- `crates/ares-core/src/print_paths/generate.rs` calls ordinary ironing before support-interface ironing.

## Included Behavior

- Add a private parser for `ironing_inset` with Orca-compatible default `0`, range `0.0..=100.0`, and millimeter units.
- Resolve the effective inset for ordinary ironing as:
  - configured `ironing_inset` when greater than zero;
  - `0.5 * first nozzle_diameter` when configured value is zero or omitted.
- Apply the effective inset only while duplicating ordinary Ironing paths for existing Ares top/solid path boundaries.
- For two-point straight infill paths, shorten the Ironing duplicate by the effective inset at both ends along the segment direction. Drop the duplicate if the effective inset would collapse the segment.
- For four-point axis-aligned rectangular path loops, move each edge inward by the effective inset. Eligible Ares rectangle loops must be `PrintPath::is_closed() == true`, contain exactly four distinct non-repeated corner points, have positive width and height, be ordered as adjacent perimeter edges in either clockwise or counterclockwise winding, and collapse when `width <= 2 * inset` or `height <= 2 * inset`.
- Treat unordered/crossed four-corner paths, four-point paths with repeated corners or zero width/height, repeated first/last-point polygons, and other non-eligible shapes as deferred non-rectangular geometry: duplicate them unchanged rather than offsetting or dropping them.
- Preserve existing ordinary-ironing gates from `ironing_type`: no ironing, top surfaces, topmost, and all solid.
- Preserve support-interface ironing behavior; `support_ironing` duplicates must not consume this ordinary `ironing_inset` slice.
- Keep `ares-core` platform-neutral and WASM-compatible.

## Deferred Behavior

- Full Orca `Layer::make_ironing` polygon collection, union, `intersection_ex`, and `Fill::fill_surface` generation.
- `ironing_pattern`, `ironing_spacing`, `ironing_angle`, `ironing_angle_fixed`, and `ironing_expansion`.
- Filament-specific `filament_ironing_spacing`, `filament_ironing_inset`, and multi-extruder current-filament selection.
- Non-rectangular polygon offsetting, holes, expolygons, region grouping, and Orca binary E2E geometry parity.
- Support ironing pattern/spacing/inset behavior.

## Acceptance Criteria

- With omitted `ironing_inset`, ordinary top-surface ironing paths are inset by half the first nozzle diameter.
- With `ironing_inset = 0.1`, ordinary top-surface ironing paths are inset by `0.1` mm instead of half the nozzle diameter.
- With a too-large inset for an eligible line/rectangle, the ordinary Ironing duplicate is omitted instead of emitting collapsed geometry.
- Closed four-corner rectangle loops are inset in both clockwise and counterclockwise point order, while unordered/crossed four-corner paths, four-point repeated-corner or zero-width/height paths, and repeated first/last-point polygons are duplicated unchanged.
- Invalid `ironing_inset` values outside `0.0..=100.0`, non-numeric values, and non-finite values return `SliceError::InvalidInput` before G-code formatting succeeds.
- Support-interface ironing duplicates are unchanged by ordinary `ironing_inset`.

## Verification

- Use TDD with `cargo nextest run -p ares-core ironing_inset` for the new focused tests.
- Run the existing ordinary/support ironing focused tests with `cargo nextest run -p ares-core ironing_type_paths support_ironing_paths`.
- Before commit, run:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard

## Docs Impact

Update `docs/roadmap.md` with a new runtime slice entry after implementation review approval. The roadmap entry must cite the same upstream boundary and state the included rectangle/line Ares behavior plus deferred full Orca ironing fill parity.
