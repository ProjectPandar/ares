# Multi-region compensation prerequisite

## Objective and upstream boundary

Rewrite the multi-region `PrintObject::slice` compensation branch in
`OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp:1314-1365`, its existing
`_shrink_contour_holes` dependency at1512-1544 and
`Layer.cpp:119-138::Layer::merged` into the Rust compensation boundary.
This is an upstream prerequisite for modifier ownership, not an Ares pipeline
milestone or approval to merge Normal support before Tree parity.

## Included behavior

- Grow the merged outline and assign overlap to the earliest region.
- Merge only regions configured to extrude.
- Apply negative XY/elephant-foot to the outline, then intersect each region.
- Preserve first-region external flow and uncompensated aggregate islands.
- Initialize closing epsilon lazily across objects in one compensation job,
  including negative-only first calls and zero closing radius.
- Retain single-region success/output and global option/flow preflight errors.

## Rust boundary and scaffold disposition

`compensation/multi_region.rs` owns the cited multi-region branch;
`compensation.rs` dispatches and retains the single-region compatibility shell;
`compensation/preflight.rs` prepares source-selected first-region flow.
Existing geometry offset/union/intersection and `slice_ordering::make_slices`
remain their source-owned dependencies. Remove only compensation's previous
multi-region rejection once its source branch is implemented. Do not remove
later consumer limitations before their own source behaviors are ported.

## Deferred behavior

Native process-static persistence and parallel first-arrival determinization
remain unresolved reference semantics. Region-owned perimeter groups,
typed/infill/fill outputs, all-region support traversal and Tree generation
are subsequent upstream slices. No Normal raw-slice backup, scheduling,
support-type dispatch or `slicing_parameters` dependency belongs here.

## Acceptance and evidence

The unchanged public modifier complete-output test remains an open overall
gate; later `UnsupportedProjectFeature` is a real failure, never a PASS.
Independent-prerequisite eligibility requires clean-HEAD proof that already
successful public project outputs remain unchanged, retained complete native
controls, all workspace tests, fmt/clippy, Tier1 compile checks and an actual
browser/host/native comparison. Do not change the KSR fixture or assertion.

Supplementary existing stage examples cover aggregate elephant-foot/shared
boundaries, positive overlap priority, subtractor exclusion, first-trigger
radius and mixed signs/clockwise hole clipping. Expected geometry comes from
worked source behavior, not production-kernel calls or source-structure pins.
They do not prove all printers/options, native cache-lifetime parity or Tree.
