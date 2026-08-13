# Task 22O.53 architecture decision record

## Status

Implemented and verified after independent source/specification review; final independent implementation re-review is the remaining milestone gate.

## Decision

Port the complete geometry-only `construct_anchored_polygon` lambda from pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `OrcaSlicer/src/libslic3r/PrintObject.cpp:2939-3111`, as the next dependency after O50-O52. Direct source dependencies are:

- `Flow.hpp:62,69::scaled_width/scaled_spacing` and `libslic3r.h:60-94::scale_`;
- `Polygon.hpp:162-168,196-217`, `Polygon.cpp:422-448`, and `Line.cpp:135-139` for polygon rotation, closed edges, and extents;
- `Point.hpp:104-114,187-242`, `MultiPoint.cpp:21-34`, and the local anchor rotation at `PrintObject.cpp:2940-2950` for 2-D integer arithmetic and `std::round` rotation;
- `Line.hpp:152-197`, including exact directed-line equality at line 166, plus `BoundingBox.hpp:16-155` and the reached extents helpers;
- O52's pinned `AABBTreeLines::intersections_with_line<true>` and `outside` queries;
- the complete flat-Paths safety union chain: constants/providers in `ClipperUtils.hpp:17-32,46-148,339,362`, `ClipperUtils.cpp:267-293::raw_offset`, `:333-365::expand_paths/offset_paths`, and `:392-408::offset(Polygons)`.

The Rust destination is a crate-private operation under `project_slice/prepare_infill/bridge_over_infill/anchored_polygon.rs`, split into ordinary `sections.rs` and `tracing.rs` modules:

```rust
construct_anchored_polygon(
    bridged_area: &[Polygon],
    anchors: &[Line],
    bridging_flow: Flow,
    bridging_angle: f64,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError>
```

Inputs are borrowed and output is owned, matching the C++ by-value nonmutation boundary without cloning policy into the caller. The operation is not wired into O43 or the public lifecycle in this milestone.

## Required behavior

The operation scales Flow spacing and width by f64 division followed by truncating `coord_t` conversion. It rotates area and anchors by `aligning_angle = -bridging_angle + PI * 0.5`, preserving C++ expression order and `std::round` ties-away-from-zero. It derives separate area-X and anchor-Y extents, creates center-shifted vertical scanlines with the exact `size_t`/double expression, and uses O52 to collect ordered area and anchor intersections.

Every adjacent area-intersection pair is tested using the source integer-sum midpoint and retained only when `outside(midpoint) < 0`. Each retained section is extended to the nearest lower/upper anchor using the two source `upper_bound` predicates. Extension evaluates `f64(endpoint.y) ± f64(scaled_width) * (0.5 + 0.5)` and truncates the compound-assignment result back to `coord_t`; it is not integer addition/subtraction. A single forward pass merges inclusive-overlapping neighbors, erases zero-length sections, and uses ARD-0024's fixed-MSVC STL 14.44 sort with the source comparator `a.a.y < b.b.y` after the equality guard.

Reconstruction keeps low/high traces in scanline order. Segment ownership is per-slice identity; Rust indices replace stable C++ element addresses without changing identity. Candidate windows use the exact two `upper_bound` predicates. Strict squared distance below `36.0 * double(spacing) * spacing` connects directly; otherwise three points bridge by integer `spacing / 2`. Unmatched traces close by the same half spacing, and unused segments seed new traces in source order. Output applies the flat-Polygon `union_safety_offset` and rotates back by `-aligning_angle` without an additional reorder.

## Trusted domain

This is an internal operation with source preconditions, not a new validation surface. Bridged-area polygons are nonempty valid closed contours satisfying O52 `outside` preconditions; anchors are a nonempty ordered set of arbitrary open or closed line edges and do not inherit the contour requirement. Flow spacing and width are finite positive values whose scaled conversions are positive and representable. Rotated coordinates, extents, extent subtraction/addition, vertical-line count and index arithmetic, anchor extension, midpoint addition/division, squared distances, threshold multiplication, and half-spacing additions all remain in defined C++ signed/unsigned and f64-to-`coord_t` domains. O52's contour preconditions continue to apply. Clipper coordinate failures from the final safety offset are propagated as the first error. Tests near limits remain within this domain and do not freeze Rust overflow, saturation, or malformed-input repair.

## Verification

A temporary pinned-C++ harness copied the complete lambda and used the actual pinned headers, Eigen, Flow, AABB traversal, and Clipper archives. All output-affecting intersection and section sorts were replayed through the audited MSVC STL 14.44 implementation; the linked Clipper archive used the same fixed-sort patch. Provenance SHA-256 values are: harness `dcdd609b834f80d5837a4c660f1d49686773b096ad738c5e2205d1dd28b934a1`, build script `b3458a8ac4af878f011d6479413a66406ea9c517580152dba57d790336940314`, literal output `6537a6cd45e4bf683b2acd996bb069168452eb50e8d2dcf870f1674b3f34d5de`, fixed-sort header `b0c5afcc36e5db5a51112dd2054ce757cc9ced6c76b0e4654d300666136b5777`, Debug libslic3r archive `ec47c8b945656e0d52f7223234b80ec66068b4a9c671e71daa5030e049e2a41b`, command graph `ce98084159ce6e6dfb6c10de3514ce9578e0b186c5d6550157db0412a7cd3d90`, and compiler wrapper `35b8a3b2d9996e93510da702121461a94771d193ea5effea84c41ef8a0658fe1`. The Rust literals freeze normal/large scales, nonzero rotation, axis alignment, empty and multi-section/two-path output, and flat Paths union order. Temporary oracle artifacts were removed after recording these hashes.

Nineteen reversible mutations were killed individually: scanline center, midpoint cast order, adjacent-window stepping, both anchor bounds, mixed-width cast, inclusive overlap, section sort deletion, conventional comparator substitution, both trace bounds, distance cast order, strict threshold, half spacing, segment identity, final trace handling, safety union, inverse rotation, and flat-Paths offset. Byte-exact restored SHA-256 values are sections `6b2b8eb967abcef0353be9e33a5a0d2a65c15cb2ae409e69e2c59596550fe761`, tracing `34f63de82e7296cb07fb1e222fb9b1d4dc9903fc157fb85bad53fd9b6f2cea67`, operation `4b3559e3af9dbba34fe48106ce72d11825cd8506aa65bc8603b3162605105b0e`, and flat wrapper `b7e0952f23c3310c5e58a2071f88c711d19bd05ebdbf470fd93dbaa9a668db74`. Labeled structural tests freeze the otherwise valid-contour-equivalent adjacent traversal and the otherwise post-merge-equivalent greater-than-32 comparator permutation. Flat Paths tests freeze overlapping union plus hole/component order and differ from raw per-path offset. The current PolyTree-to-ExPolygon contour/hole flattening is observationally identical for this returned `Vec<Polygon>` boundary, so it is not a separately killable alternative and no hierarchy abstraction is introduced.

Final gates pass focused 20/20, dependency 650/650, workspace 6,310/6,310, warning-denying Clippy, browser wasm32, x86_64/aarch64 Windows, x86_64/aarch64 macOS, rustfmt, diff, LOC, static, and clean pinned-Orca checks.

## Consequences

O53 completes the reusable anchored-polygon geometry dependency but does not port its `PrintObject::bridge_over_infill` call sites. Candidate clustering, O46/O47/O48/O49/O51 composition, boundary-anchor assembly, collision rerun, opening/closing/intersection/difference postprocessing, surface mutation/commit, prepared successor, extrusion, motion, G-code, and CLI activation remain deferred.

Production and tests use ordinary modules, every source file stays below 400 LOC, and `include!`, `include_bytes!`, and `include_str!` are prohibited for source/test splitting.
