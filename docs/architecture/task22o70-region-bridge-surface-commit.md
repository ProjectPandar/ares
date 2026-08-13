# Task 22O.70 architecture decision record

## Status

Accepted and implemented source-cited boundary.

## Upstream boundary

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintObject.cpp:3385-3386` — remove old internal solid/sparse
  surfaces and append the complete rebuilt surface vector;
- `SurfaceCollection.cpp:127-138` — stable in-place `remove_types`; and
- `SurfaceCollection.hpp:78` plus `Surface.hpp:259-261` — the reachable lvalue
  copy-append order and ownership.

## Rust destination seam

Add ordinary private module
`project_slice/prepare_infill/bridge_over_infill/region_bridge_surface_commit.rs`:

```rust
pub(in crate::project_slice) fn commit_region_bridge_surfaces(
    fill_surfaces: Vec<RegionSurface>,
    new_surfaces: &[RegionSurface],
) -> Vec<RegionSurface>;
```

## Decision

The operation consumes the original vector and borrows `new_surfaces`. It
stably retains every original surface except `InternalSolid` and `Internal`,
preserving all metadata/topology and relative order, then copies and appends
every `new_surfaces` element in caller order, matching Orca's named-lvalue call.
The trusted composer constructs `new_surfaces` as O67 results followed by O68
then O69 results. The operation performs no geometry, validation, sorting,
deduplication, option lookup, error mapping, or fixture dispatch.

Inputs are same-region transaction state assembled from O67, O68, and O69. The
operation is infallible because upstream removal/append is infallible and all
geometry errors have already propagated.

## Included and deferred

Included: exact region-local collection commit at `3385-3386`.

Deferred: outer region/layer/map/cluster traversal and scheduling, second bridge
pass at `3393+`, prepared successor and lifecycle activation, extrusion, motion,
G-code, CLI, and complete golden parity.

The unwired `ares-core`-private operation introduces no filesystem, terminal,
OS, thread, UI, OpenGL, unsafe, or platform behavior. All source/test files stay
below 400 LOC and use ordinary modules without include-macro source splitting.

## Verification

The behavioral RED retained `Internal`; the minimal implementation fixed the
stable two-kind removal. Focused tests pass 3/3, the workspace passes
6,457/6,457 with two skipped, strict Clippy/rustfmt and all five portability
checks pass, and 15/15 compiling collection/order mutations are killed with
byte-exact production restoration. Independent six-axis review approved after
the full contour-and-ordered-holes snapshot repair, with no remaining item.
