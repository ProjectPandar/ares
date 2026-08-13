# Task 22O.53 anchored bridge polygon implementation plan

## Status

Complete. Independent source/specification review preceded RED; implementation, oracle, mutation, runtime, portability, and static gates pass pending final independent implementation re-review.

## Objective

Port pinned `PrintObject.cpp:2939-3111::construct_anchored_polygon` and its complete reached helper closure (`Flow.hpp`, `libslic3r.h`, `Polygon.hpp/.cpp`, `Line.hpp/.cpp`, `Point.hpp`, `MultiPoint.cpp`, `BoundingBox.hpp`, O52 `AABBTreeLines`, and `ClipperUtils.hpp/.cpp` flat Paths offset) as a crate-private, lifecycle-neutral geometry operation on existing O50-O52 and Flow seams.

## Plan

1. **Freeze the source boundary**
   - Independently review the ADR/spec against pinned `PrintObject.cpp`, `Flow.hpp`, `libslic3r.h`, `Polygon.hpp/.cpp`, `Line.cpp`, `Point.hpp`, `MultiPoint.cpp`, `AABBTreeLines.hpp`, `ClipperUtils.hpp`, and ARD-0024.
   - Build a temporary oracle with the actual pinned dependencies to freeze rotation, scanlines, section vectors, trace vectors, final flat polygons, errors, and both runtime scales. Normative output uses audited MSVC STL 14.44; any non-MSVC driver is arithmetic/traversal evidence only and must expose every output-affecting pre-sort vector for replay through ARD-0024, including Clipper Paths ordering. Record exact compiler/dependency provenance and remove all artifacts after literals are committed.

2. **Behavioral RED**
   - Add ordinary anchored-polygon production/test modules and a compiling operation with `todo!()`.
   - Freeze exact literals for every acceptance discriminator, including intermediate test-only observations only where final output cannot distinguish source arithmetic or ownership.
   - Retain a focused RED proving the operation is not supplied by existing geometry scaffolding.

3. **Minimal source-shaped implementation**
   - Add the flat polygon safety-offset wrapper directly corresponding to `ClipperUtils.hpp:362`.
   - Implement source-order scaling/rotation/extents/scanlines and section extraction on O52.
   - Implement anchor extension, one-pass merge, fixed-MSVC section sorting, identity-preserving trace reconstruction, flat safety union, and inverse rotation.
   - Add no configuration parsing, public validation, alternate geometry engine, fallback, stage successor, or candidate mutation.

4. **Verify, mutate, and review**
   - Run focused O53, O43-O53/geometry/Flow dependency, workspace, rustfmt, warning-denying Clippy, native Linux plus core/browser wasm32, Windows and macOS compile/test CI, diff/LOC/static and clean-Orca gates.
   - Kill every acceptance-critical reversible mutation listed in the spec, restore byte-exact, and remove temporary artifacts.
   - Record evidence in ADR/spec/plan, roadmap, and option parity.
   - Launch an independent six-axis implementation review; return findings to the main thread, repair, and re-review until unconditional approval.

## Exit criteria

- Ordered output and intermediate discriminators match the actual pinned C++ lambda.
- Every arithmetic, bound, comparator, ownership, and final union branch is test-discriminated.
- Inputs remain unchanged and errors are atomic.
- Production stays crate-private, portable, and lifecycle-neutral.
- Every source is below 400 LOC, ordinary modules are used, and include macros are absent.
- Runtime, static, mutation, and independent review gates all pass.
