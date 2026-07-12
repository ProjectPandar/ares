# PrintApply transform Z-rotation/mirroring predicate Spec

## Goal

Port OrcaSlicer's private `trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(...)` helper into `ares-core` as a staged private transform predicate for later print-object-region invalidation milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:556-580`: translation-Z guard, relative 3x3 transform, Z-axis check, X/Y Z-component check, unit-length check, and perpendicularity check.

Required context:
- `OrcaSlicer/src/libslic3r/Point.hpp:79-85`: `Transform3f` and `Transform3d` are Eigen 3D affine transforms with `float` / `double` scalar types.
- `OrcaSlicer/src/libslic3r/libslic3r.h:48-52`: Orca `EPSILON = 1e-4`.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:667-695`: later downstream invalidation context where transform compatibility affects cached-volume reuse.

## Requirements

- Extend private `crates/ares-core/src/print_apply/transform_state.rs`; do not add public APIs.
- Add a private helper equivalent to `trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(t1, t2)`.
- The helper must return `false` when `abs(t1.translation_z - t2.translation_z) > ORCA_EPSILON`.
- The helper must compute `m = inverse(linear_3x3(t2)) * linear_3x3(t1)`.
- The helper must return `false` unless relative Z column satisfies `abs(z.x) <= EPSILON`, `abs(z.y) <= EPSILON`, and `abs(z.z - 1.0) <= EPSILON`.
- The helper must return `false` when relative X or Y column has Z component whose absolute value exceeds `EPSILON`.
- The helper must return `false` when `x.squared_norm() - 1.0 > EPSILON * EPSILON` or `y.squared_norm() - 1.0 > EPSILON * EPSILON`.
- The helper must return `abs(dot(x, y)^2) < EPSILON * x.squared_norm() * y.squared_norm()` after all earlier guards.
- Add unit tests for accepted identity, accepted Z rotation, accepted XY mirroring, rejected Z translation mismatch, rejected tilted Z column, rejected X/Y Z components, rejected non-unit scale, and rejected non-perpendicular X/Y columns.
- Do not implement full Eigen-compatible transforms, public transform APIs, robust general matrix algebra beyond this predicate, mesh vertex transformation, bounding boxes, print-object-region invalidation, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
