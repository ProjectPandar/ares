# Task 22O.50 nearest anchor-line AABB tree implementation plan

## Status

Implemented and complete. Independent source/specification review preceded RED; final independent implementation review approved unconditionally.

## Objective

Implement the exact nearest indexed-line build/query reached by pinned
Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`determine_bridging_angle`, without yet implementing direction aggregation.
The reached Eigen 5.0.1 archive hash is
`0dbb1f9e3aaad66f352c03227d8c983f6f0b49e0b07e71a7300f4abcc01aee12`.

## Plan

1. **Freeze the source boundary**
   - Review ADR/spec against `AABBTreeLines.hpp`, `AABBTreeIndirect.hpp`,
     `Line.hpp`, and `Utils.hpp`.
   - Use a temporary standalone C++ driver instantiating the actual pinned
     templates/Eigen to freeze complete adversarial node layouts, centroids,
     fixed bbox distances/final bits, nearest indices/distances/points, and tie
     ownership, including above-2^53 and `HI_RANGE` cases that distinguish the
     i64→f64→i64 primitive roundtrip from mixed-scalar Eigen bbox deltas. Keep
     i128-extension literals separate; restore Orca byte-exact
     and keep no runtime source pinning.

2. **Behavioral RED**
   - Register ordinary geometry test modules and the spec's exact compiling
     `LineDistanceTree::new` / `nearest` / `NearestLine` API with a `todo!()`
     query plus cfg(test) node snapshots.
   - Add literal tests for build partitions/layout, projection arithmetic,
     traversal/pruning/ties, empty/repeatability/nonmutation, Clipper
     `HI_RANGE`, native i64 hazards, and separately specified i128 extensions.
   - Run `cargo nextest run -p ares-core -E 'test(/task22o50/)'` and retain RED.

3. **Implement minimal deep module**
   - Add borrowed lines plus implicit nodes and source-exact QuickSelect.
   - Add pinned-Eigen fixed-scalar bbox distance, recursive nearest traversal,
     and exact X-then-Y non-fused segment squared-distance projection without
     reusing `Line::distance_to`.
   - Split build/query into ordinary submodules before any file reaches 400
     LOC. Add no generic tree abstraction beyond the reached 2D line use.

4. **Verify and document**
   - Run focused, geometry/O43-O50 dependency, workspace, rustfmt,
     warning-denying Clippy,
     `cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown`,
     diff/LOC/static gates, with no wasm-bindgen API addition.
   - Record exact evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis implementation review; main-thread repairs and
     re-review repeat until unconditional approval.

## Exit criteria

- Tree layout and every adversarial nearest result match pinned C++ literals.
- Source lines remain borrowed/unchanged and no brute-force fallback exists.
- Production remains cross-platform and lifecycle-neutral.
- No changed Rust file exceeds 400 LOC or uses source-splitting include macros.
- All gates and independent review approve.

## Execution evidence

The compiling RED failed 0/5. Implementation verification passes focused 8/8,
dependency 613/613, workspace 6,273/6,273, warning-denying workspace Clippy,
rustfmt, core/browser wasm32, diff, LOC, and static audits. The remaining plan
step was independent six-axis implementation review. Its first pass found the
centroid and discriminator defects; main-thread repairs passed every gate, and
the read-only re-review approved unconditionally with no remaining finding.
