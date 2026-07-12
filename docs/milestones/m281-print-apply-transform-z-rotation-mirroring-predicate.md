# M281: PrintApply transform Z-rotation/mirroring predicate

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:556-580`, with transform alias context from `OrcaSlicer/src/libslic3r/Point.hpp:79-85` and `EPSILON` context from `OrcaSlicer/src/libslic3r/libslic3r.h:48-52`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Extend private staged transform helpers with enough 3x3 linear-matrix support for this predicate.
- Preserve the Z-translation equality guard using Orca `EPSILON = 1e-4`; return `false` when `abs(t1.z - t2.z) > EPSILON`.
- Preserve computing the relative linear transform as `m2.inverse() * m1` from the upper-left 3x3 blocks.
- Preserve accepting only relative transforms whose Z column is approximately `[0, 0, 1]` within `EPSILON`.
- Preserve rejecting relative X/Y columns with Z components beyond `EPSILON`.
- Preserve rejecting X/Y columns whose squared lengths exceed 1 by more than `EPSILON * EPSILON`, matching upstream `lx2 - 1. > EPSILON * EPSILON`.
- Preserve the final perpendicular check `abs(dot(x, y)^2) < EPSILON * lx2 * ly2`.
- Add tests for accepted identity, accepted Z rotation, accepted XY mirroring, rejected Z translation mismatch, rejected tilted Z column, rejected X/Y Z components, rejected non-unit scale, and rejected non-perpendicular X/Y columns.
- Defer full Eigen-compatible transform API, robust general-purpose matrix algebra, mesh vertex transformation, bounding boxes, print-object-region invalidation, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
