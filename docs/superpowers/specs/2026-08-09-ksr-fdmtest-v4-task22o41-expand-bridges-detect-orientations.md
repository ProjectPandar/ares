# Task 22O.41 — Expand bridges and detect orientations

## Status

Locally implemented, crate-private, inactive, and independently approved after
the initial review's sorting and error-ledger coverage findings were repaired.
Six focused tests, 53 external-surface regressions, and 6,107 workspace tests
pass with two skipped. Workspace warning-denying Clippy, rustfmt, diff, LOC,
and include audits pass. The normalized KSR golden remains the expected RED at
the CLI `--options` boundary.

## Goal and upstream boundary

Port OrcaSlicer 2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/LayerRegion.cpp:395-437::expand_bridges_detect_orientations`,
into the crate-private
`project_slice::prepare_infill::external_surfaces::expand_bridges_detect_orientations`
function.

The function composes the already ported O36 `expand_expolygons`, O37
`get_grouped_bridges`, O39 `detect_bridge_directions`, and O40
`merge_bridges` helpers. It extracts `BottomBridge` geometry from the supplied
surfaces, returns early when none exists, sorts anchors by `(src, boundary)`,
sorts expansions by `(src_id, boundary_id)`, produces merged bridge surfaces,
and subtracts those surfaces only from zones marked `expanded_into`.

## Rust boundary and behavior

The Rust entry accepts `&mut [RegionSurface]`, `&mut [ExpansionZone]`, the
closing radius, and the explicit platform-neutral `CoordinateScale`. Matching
surface geometry is moved out while metadata and nonmatching surfaces remain
in place, following the existing O35 extraction convention. Errors from every
Clipper-backed stage propagate directly; already completed mutations are not
rolled back.

Included behavior is the exact helper orchestration, ordering, early return,
zone flag use, and final clipping. Deferred behavior is
`LayerRegion::process_external_surfaces` at `LayerRegion.cpp:486-623`, lifecycle
activation, configuration plumbing, fill/toolpath/motion/G-code stages, and CLI
activation. Existing Ares pipeline stages remain temporary compatibility
shells and this slice adds no fixture branch or Ares-owned slicing design.

## Acceptance

Behavior tests at the crate-private functional seam cover no-source early
return, matching-source extraction, nonmatching surface preservation, sorted
composition, clipping only expanded zones, metadata defaults, and direct
Clipper errors. Tests live in a separate module; all Rust files remain below
400 physical lines. Focused and workspace Nextest, rustfmt, and warning-denying
Clippy must pass. The normalized KSR golden remains the continuing end-to-end
RED until downstream source slices and adapters are complete.
