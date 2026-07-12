# Consume Wall Loops Design

## Scope

Implement concrete perimeter behavior for the existing `wall_loops` option before adding more option metadata. This slice covers classic perimeter loop count consumption in `ares-core`.

This does not add new options, new crates, Arachne wall generation, gap fill, wall sequencing, thin-wall repair, or true polygon offsetting.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp`: `PrintRegionConfig` declares `wall_loops` as the total number of perimeters.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp`: classic perimeter generation creates multiple nested perimeter loops from each slice boundary according to region configuration.
- Existing Ares destination: `crates/ares-core/src/options.rs` must parse `wall_loops`, `crates/ares-core/src/perimeters.rs` must consume it, and `crates/ares-core/src/pipeline.rs` must pass the parsed option into perimeter generation.

## Behavior

Ares currently emits exactly one external perimeter per contour. This slice changes that to emit `wall_loops` perimeter paths per contour when the option is positive.

Because Ares does not yet have libslic3r offset/boolean geometry, the implementation uses the same rectangular contour approximation style already used by skirts and brims:

- The first loop keeps the original contour points and is tagged as `External`.
- Additional loops are tagged as `Internal`.
- For axis-aligned rectangular contours, additional loops shrink the contour bounds inward by `loop_index * perimeter_line_width`.
- A loop is skipped if the shrink collapses the rectangle.
- Non-rectangular contours keep only the external contour for now; internal non-rectangular offsets stay out of scope until the upstream polygon offset boundary is ported.
- `wall_loops = 0` emits no perimeter paths.
- Missing `wall_loops` keeps the Orca/Ares default of `2`.

The pipeline derives `perimeter_line_width` from the existing extrusion width resolution for `PrintPathRole::ExternalPerimeter`: `outer_wall_line_width` when set, otherwise `line_width` when set, otherwise the current automatic nozzle-width fallback. This slice does not add `inner_wall_line_width`; internal loops use the same spacing as the resolved perimeter line width until the upstream inner-wall width boundary is ported.

An axis-aligned rectangular contour is executable-defined as exactly four contour points whose unique x values are the contour min/max x and whose unique y values are the contour min/max y. Point order may be the normalized `Contour::new` order; the contour does not need to repeat the first point as a closing point.

The loop count is intentionally consumed before adding more option metadata. It changes `LayerPerimeters`, `PrintPathRole`, extrusion role mapping, and G-code path output through the existing pipeline:

- `PerimeterRole::External` maps to existing `PrintPathRole::ExternalPerimeter`.
- `PerimeterRole::Internal` maps to new `PrintPathRole::InternalPerimeter`.
- `PrintPathRole::InternalPerimeter::as_str()` returns `internal_perimeter`.
- Internal perimeter paths close in `moves.rs`, the same as external perimeter paths.
- Internal perimeter extrusion role maps to existing `ExtrusionRole::Perimeter`.
- Internal perimeter extrusion width uses the default resolved `line_width` path in `ExtrusionOptions`; it does not use `outer_wall_line_width`.
- Internal perimeter speed uses existing `inner_wall_speed` when present and falls back to `outer_wall_speed`.
- G-code comments/metadata that expose path roles surface `internal_perimeter` through the existing role string flow.

## Acceptance Criteria

- A unit test proves `generate_perimeters` with `wall_loops = 3` and line width `0.4` emits one external rectangle at `(0,0)..(4,4)` and two internal rectangles at `(0.4,0.4)..(3.6,3.6)` and `(0.8,0.8)..(3.2,3.2)`.
- A unit test proves collapsed internal loops are skipped.
- A unit test proves `wall_loops = 0` emits no perimeter paths while preserving layer metadata.
- A unit test proves non-rectangular contours keep only the external perimeter even when `wall_loops > 1`.
- Unit tests prove internal perimeter paths map to `PrintPathRole::InternalPerimeter`, close in move generation, map to `ExtrusionRole::Perimeter`, use the default line width rather than `outer_wall_line_width`, and use `inner_wall_speed` when provided.
- A pipeline-adjacent test proves JSON option `"wall_loops": 3` is parsed through `SliceOptions::perimeter_options()` and increases generated perimeter artifacts/downstream print paths for an explicit four-point rectangular `LayerContours` fixture.
- Existing default pipeline behavior changes from one perimeter to the default two perimeters; tests must be updated only where they asserted the old scaffold count.
- `cargo test -p ares-core --lib`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Docs Impact

This internal behavior is documented by this SDD spec and focused regression tests. No user-facing docs or roadmap edits are required for this slice.

## Out Of Scope

- True inward polygon offsetting for arbitrary polygons.
- Hole-aware perimeter classification beyond the current contour model.
- Arachne perimeter generation.
- `inner_wall_line_width`, `wall_sequence`, `wall_direction`, `alternate_extra_wall`, `detect_thin_wall`, and gap-fill behavior.
- Any new `PrintConfig.hpp` option metadata milestone.
- Any independent Ares-owned pipeline design not tied to the upstream boundaries above.
