# Consume Existing Slicing Options Design

## Scope

Implement concrete slicing behavior for existing, already parsed options before adding new option metadata. This slice covers:

- Sparse infill clipping across multiple layer contours so inner holes are not filled.
- `brim_type` behavior for `outer_only`, `inner_only`, `outer_and_inner`, `auto_brim`, and `no_brim`.

This does not add new options, new crates, or new pipeline stages.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp`: infill polylines are clipped against actual fill regions instead of generated over each contour independently.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp`: fill regions are derived from expolygon areas with holes excluded before infill generation.
- `OrcaSlicer/src/libslic3r/Brim.cpp`: `outer_inner_brim_area` derives `has_inner_brim` from `btInnerOnly` / `btOuterAndInner`, derives `has_outer_brim` from `btOuterOnly` / `btOuterAndInner` / `btAutoBrim`, expands outer contours, and shrinks holes for inner brim areas.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp` and `PrintConfig.cpp`: `BrimType` variants and option keys are already represented in Ares and must affect generated brim artifacts.

## Behavior

Sparse infill generation treats all contours on a layer as one even-odd filled area. Scanline intersections from outer contours and holes are merged, sorted, and paired. Filled spans become sparse infill paths; hole spans are skipped.

Brim generation remains first-layer only and keeps the current rectangular approximation. Ares does not yet store `ExPolygon` hole metadata, and `Contour::new` normalizes winding, so this slice classifies contours by containment parity:

- A contour contained by zero or an even number of other contours is an outer island.
- A contour contained by an odd number of other contours is a hole.
- Separate, non-contained contours are independent outer islands.
- Nested contours alternate outer/hole by the same parity rule.

For this simple contour model:

- `outer_only` and `auto_brim` generate outward brim loops around each outer island bounds.
- `inner_only` generates inward brim loops around hole contours only.
- `outer_and_inner` generates both outward outer loops and inward hole loops.
- `no_brim`, `painted`, and `brim_ears` produce no paths until their upstream geometry inputs exist.

## Acceptance Criteria

- A regression test proves sparse infill paths do not cross an inner hole.
- A regression test with outer square `(0,0)..(4,4)`, hole square `(1,1)..(3,3)`, `brim_width = 0.8`, `brim_object_gap = 0.0`, and effective line width `0.4` proves `inner_only` emits two inner loops at `(1.4,1.4)..(2.6,2.6)` and `(1.8,1.8)..(2.2,2.2)` and no outer loop.
- A regression test with the same fixture, `brim_width = 0.4`, `brim_object_gap = 0.0`, and effective line width `0.4` proves `outer_and_inner` emits an outer loop at `(-0.4,-0.4)..(4.4,4.4)` and an inner loop at `(1.4,1.4)..(2.6,2.6)`.
- A regression test proves `painted`, `brim_ears`, and `no_brim` emit no paths while their upstream geometry inputs are unavailable.
- Existing `brim_width`, `brim_object_gap`, line width fallback, and first-layer-only behavior keep passing.
- `cargo test -p ares-core --lib`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Docs Impact

No user-facing docs or roadmap updates are required for this slice. The change is internal `ares-core` artifact generation behavior and is documented by this SDD spec and focused regression tests.

## Out Of Scope

- True polygon offsetting, boolean repair, ear brim geometry, painted brim, support brim, and object-specific placement.
- Any new `PrintConfig.hpp` option metadata milestone.
- Any independent Ares pipeline design not tied to the upstream boundaries above.
