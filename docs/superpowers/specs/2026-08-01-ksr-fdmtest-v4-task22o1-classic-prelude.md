# Task 22O.1: Classic Perimeter Prelude

## Status and objective

This bounded milestone directly ports the first executable portion of the
Classic perimeter generator reached after Task 22N. It advances the private
project lifecycle through transactional Classic capability validation and the
pre-onion `PerimeterGenerator::process_classic()` prelude. Public
`slice_project` still returns `ProjectSlicingIncomplete`.

The earlier Task 22O Package-A0 qualification/recovery documents are retained
as historical audit evidence. Their corrected source analysis remains useful,
but their external Windows campaign is not a prerequisite for Rust behavior
and must not be retried or treated as a production gate.

## Fixed upstream boundary

All behavior comes from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/LayerRegion.cpp::LayerRegion::make_perimeters` owns caller,
  inputs, dispatch, and output lifetime;
- `src/libslic3r/PerimeterGenerator.cpp::process_classic`, from entry through
  surface ordering and loop-count preparation immediately before the onion
  loop, owns the implemented prelude;
- `PerimeterGenerator.cpp::process_no_bridge` contributes its `chbNone`
  pass-through;
- `PerimeterGenerator.cpp::generate_lower_polygons_series` owns the two sampled
  lower-support offsets;
- `src/libslic3r/Flow.cpp::Flow::with_width` owns smaller-external-flow
  reconstruction; and
- `src/libslic3r/ShortestPath.cpp::chain_expolygons` plus
  `src/libslic3r/BoundingBox.hpp` own bounding-box-center surface ordering.

The Rust destination is crate-private
`ares_core::project_slice::perimeters::classic` plus reusable fixed-coordinate
bounding-box ordering in `ares_core::geometry`.

## Included behavior

1. Validate every populated Task 22N record before consuming any predecessor
   object. Preserve existing earlier `raft_layers` and multi-region errors.
2. Accept only the currently implemented Classic subset. Activated Arachne,
   spiral, fuzzy deformation, thin walls, alternating/first-layer special
   walls, overhang reversal, unsupported wall order, active outer-only brim
   reversal, active extra-overhang perimeters, and non-`None` counterbore modes
   return `UnsupportedProjectFeature` with the owning Option key.
3. Preserve Task 22N records and empty-slot alignment in an owned
   post-Classic-prelude object.
4. Consume the four Task 22N `Flow` values without recomputing them. Compute
   scaled widths/spacings, precise external/internal spacing, collapse
   tolerances, gap enablement, and smaller external Flow with fixed narrowing.
5. Grow lower slices for overhang support and build normal internal, normal
   external, and smaller-external two-sample lower polygon series.
6. Apply the counterbore-none pass-through, arc-aware surface simplification,
   per-surface union, bounding-box-center chaining, and fixed loop-count
   derivation including topmost one-wall behavior.
7. Read every production value from typed effective configuration loaded from
   the 3MF. Production code never reads the reference G-code or fixture
   identity.

## Deferred behavior

The milestone stops before `split_top_surfaces()` and before the Classic onion
shell loop. Onion offsets, smaller-width geometry selection, hierarchy,
traversal, open-path overhang splitting, gap medial axes, variable-width
extrusion, fill remainder, seam placement, infill, motion planning, G-code,
metadata, and post-processing remain source-cited follow-up slices. Arachne and
all rejected activated branches require their owning later milestone; there is
no old rectangular project fallback.

## Structure and acceptance

All Rust source and test files remain below 400 physical lines. Tests live in
normal `mod` files. No `include!`/`include_bytes!` source splitting, unsafe,
Orca runtime/FFI, source-text pinning test, fixture branch, or new dependency is
allowed. The opaque `task22n_synthetic.bin` embedding is removed and replaced
with readable behavioral parser construction.

Acceptance requires focused Task 22O.1 tests, Task 22 regressions, workspace
Nextest, rustfmt, warning-denying Clippy, workspace/default/all-feature checks,
and installed WASM target checks. This milestone does not claim complete Task
22O or KSR G-code parity.
