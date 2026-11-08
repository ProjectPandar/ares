# Painted-ear radius and EFC projection

## Source boundary

Pinned libslic3r revision `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `Brim.cpp::make_brim_ears`395–454: transformed painted points, direct `scale_(head_front_radius)` radius at419,24-sided disc, EFC centre projection and translation.
- `Brim.cpp::closest_point_on_expolygons`105–125: iterate contours/holes and their segments in order, compare unquantized squared segment distances with strict `<`, retain the associated projected integer point.
- `Brim.cpp::closest_point_on_matching_island`132–147: retain matching raw-island intersection selection, otherwise the complete EFC outline.
- `Line.hpp::line_alg::distance_to_squared`45–73: clamped segment projection, degenerate/end-point cases, unquantized distance and truncating integer candidate conversion.
- `Point.hpp:208`: the disc's double-coordinate constructor rounds coordinates; preserve the existing disc rounding.

Rust destination: `crates/ares-core/src/brims/ears.rs::painted` and its existing private closest-point helper. Remove the obsolete `PaintedParameters::flow_width` and `spacing` fields and their sole initializers in `project_slice/gcode_emit/brim.rs`; those two initializer removals are the only caller changes.

The existing painted-ear implementation is a compatibility shell around this upstream helper. Replace its older flow-quantized radius and nearest-vertex behavior, rather than introduce a second brim path or an independent pipeline.

## Proven input boundary

Both existing fixtures have the same Native/Rust painted radius and raw outlines, including vertex/hole ordering. The EFC fixture also has identical adjusted outlines. Local XY matches using the actual Native model matrix/original point/centre-offset operands and the cited transform operations; optimized-out Native locals must not be represented as direct captures.

These inputs separate the helper discrepancy from model metadata, EFC outline generation and object placement. Native constructs its disc with the raw radius; the current Rust outer arm quantizes by flow spacing and subtracts gap/spacing, while its inner arm performs a mixed-unit quantization. The current projection selects vertices instead of segment interiors.

## Included behavior

Use the raw scaled painted radius for both outer and inner disc construction. Preserve existing circle discretization, outline clipping, inner-hole intersection, forbidden-area construction and path generation.

Replace vertex selection with an ordered segment scan. Reuse existing scalar segment-distance behavior; derive the corresponding clamped integer candidate with Native's truncating cast rather than Polygon::point_projection's rounded candidate. Preserve strict-distance tie order and matching-island selection. Do not change shared geometry primitives or callers to introduce a new projection API.

Retire only the no-longer-consumed flow-width and spacing parameters. Do not retain a dummy read, ignored compatibility field or alternate radius algorithm.

## Test-first contract

After this SPEC, record the failing state with these unchanged existing tests:

- `brim_painted_ears_from_ear_points::brim_painted_ears_from_ear_points_matches_native`
- `brim_painted_efc_snaps_ear_centre::brim_painted_efc_snaps_ear_centre_matches_native`

Preserve input hashes, Native activation checks and complete-output assertions. No new unit seam, fixture, test normalization or comparator change is needed.

Both tests must pass after the geometry correction. In particular the first fixture initially differs at M73 before the first brim motion; only a geometry-only correction passing the unchanged complete-output assertion can establish that this discrepancy was downstream of brim geometry. If that first difference remains, or another producer must change, retain that discrepancy as out of scope rather than adjust timing/progress output.

Run fmt and strict workspace/all-target clippy at each submitted commit. Preserve failed attempts and residuals as failures; focused parity does not establish whole-suite parity.

## Deferred behavior

Automatic ears, global brim grouping/flow, shared clipping and EFC generation, model transformations, seam/travel/spiral geometry, fan scheduling, pressure advance, M73/time estimation, headers/configuration and all comparison policies remain unchanged. No fallback or duplicated processing path is introduced.
