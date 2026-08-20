# Spec: Remove obsolete Clipper path-order pinning

## Observable contract

Geometry tests validate union, XOR, and safety-difference topology and error behavior without requiring a particular contour start vertex, result ordering, or internal subject-path insertion order. Equivalent rotated polygons and reordered disjoint results are accepted.

## Upstream boundary

Ares retains the geometric contracts used by OrcaSlicer 2.4.2 `src/libslic3r/ClipperUtils.cpp`; tests no longer encode one Clipper module's non-semantic path serialization. Production geometry behavior is unchanged.

## Acceptance

Obsolete source-stage checkpoint tests, fixed stage hashes, hard-coded perimeter geometry checksums, and exact downstream point-count inventories are removed. Geometry, compensation, perimeter, bridge, and fill-entity tests compare behavior and topology independently of equivalent contour rotation and result ordering while retaining option loading, public capability gates, nonempty valid output, structural count relationships, disjoint, empty, determinism, metadata, and input-immutability checks. Workspace tests no longer fail solely because equivalent polygons start at different vertices or are returned in a different order.
