# Task 22O.75 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-13.

## Decision

Replace the temporary KSR-reduced grouping inside sparse-infill anchoring with
Task 22O.74's complete `group_fills` result.

The pinned upstream boundary is OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/Fill.cpp:1394-1407`: the public
`Layer::generate_sparse_infill_polylines_for_anchoring` caller obtains
`group_fills(*this, skin_inner_param)` and then selects only `stInternal`
groups. The reused fill-dispatch subset remains `Fill.cpp:1409-1499`, already
ported for the KSR-active CrossHatch case by Task 22O.46.

The Rust destination is
`project_slice::prepare_infill::bridge_over_infill::sparse_anchoring`.
Its crate-private entry accepts the prepared external-surface graph plus aligned
object and layer indices, calls `project_slice::group_fills::group_fills`,
filters `RegionSurfaceKind::Internal`, and feeds the existing CrossHatch
implementation from the returned authoritative group geometry and parameters.

Delete `sparse_anchoring/grouping.rs` in the same change. Do not preserve its
`Pattern`, `SurfaceFill`, comparator, priority pass, wrapper, alias, or fallback.
The transaction caller passes its existing prepared graph and aligned indices;
it does not reconstruct options or surfaces.

## Rationale

O46 intentionally used a reduced private grouping only because complete
`group_fills` did not yet exist. O74 now owns projection, comparator identity,
coalescing, priority clipping, narrow-solid handling, no-overlap ownership, and
LockedZag sidecars. Keeping the reduced copy would make the two upstream
`group_fills` callers disagree and would allow later options to bypass the
3MF-derived effective configuration.

Using graph indices rather than a caller-built `SparseAnchoringLayer` makes the
source caller relationship explicit and prevents the bridge transaction from
assembling a second option view.

## Included behavior

- call full O74 grouping before the `Internal` filter;
- preserve full grouping errors and atomicity;
- preserve source group and ExPolygon order;
- use actual grouped spacing, angle, multiline, anchor lengths, and geometry,
  converting grouped percentage density with source `float(0.01 * density)`;
- preserve zero sparse-density skipping in the owning bridge transaction;
- preserve deterministic, borrowed-input behavior and the fixed-MSVC KSR
  anchoring oracle, updating the oracle only if the full source caller proves a
  different result;
- delete the reduced O46 grouping implementation and tests coupled only to its
  caller-built seam.

## Deferred behavior

AdaptiveCubic, SupportCubic, Lightning, rotation-template grammar, and every
non-KSR fill generator remain rejected by the existing bridge transaction
capability gates. `Layer::make_fills` (`Fill.cpp:1213-1374`), extrusion
entities, ironing, motion planning, G-code generation, CLI success, and final
normalized golden parity remain later source-cited slices.

This task adds no public API, prepared lifecycle stage, filesystem access,
threading, UI, OpenGL, terminal behavior, Cargo feature, or compatibility
fallback. Public slicing still ends at `ProjectSlicingIncomplete`.

## Verification

Behavior tests use a real prepared external-surface graph. The fixed-MSVC KSR
18-layer anchoring oracle remains exact at 186 paths and 5,941 points with
aggregate SHA-256
`917adc6ea02ad7cd7af79e45d90db6f4c1497bf5c8716d7f2f49b7de4b2070ef`.
The graph is repeatable and unchanged. Focused results passed 1/1 anchoring,
35/35 full grouping, and 17/17 bridge transaction tests. Workspace Nextest
passed 6,516/6,516 with 27 slow tests and two configured skips; warning-denying
core all-target/all-feature Clippy, rustfmt, and diff checks passed.

The largest changed Rust file is `candidate_expansion.rs` at 363 LOC. The
reduced grouping and its direct caller-built tests are deleted; ordinary module
files remain and no `include!` or `include_bytes!` source splitting was added.
