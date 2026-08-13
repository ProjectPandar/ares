# Task 22O.40 — Merge bridge groups

## Status

Locally implemented, crate-private, inactive, unreleased, and independently
approved.
The compiling stub RED failed the first emitted-surface assertion. Eight O40
behavior tests pass, the O35-O40 focused regression set passes 69/69, and
workspace Nextest passes 6,101/6,101 with two skipped. Warning-denying Clippy,
rustfmt, native and wasm32 checks, diff/LOC/include audits, and the expected-red
public KSR progress probe have run on the candidate. The probe still stops at
the pre-core CLI `--options` requirement.

The initial six-dimensional review rejected a rustfmt failure, missing valid-
input coverage, and one incomplete citation. After formatting the candidate,
adding the single-bridge hole/default case, same-group narrow-gap closing case,
contiguous multi-expansion/hole case, completing the closing-default citations,
and rerunning every gate, the same review thread returned `VERDICT: APPROVE`
with zero findings.

O39 is released as implementation/documentation commits
`2038e93491de89e33f12ecb5379132a013bfc996` /
`c84119ee6871a176ec94117bc16f7e402c9caf96`; exact-SHA Tier-1 run
`31317150231` passed all five jobs and both browser executions at the
documentation SHA.

## Goal

Port the next bounded OrcaSlicer 2.4.2 rewrite slice needed by the
`ksr_fdmtest_v4` project path: `merge_bridges` from
`OrcaSlicer/src/libslic3r/LayerRegion.cpp:310-351`.

The Rust destination is a crate-private
`project_slice::prepare_infill::external_surfaces::merge_bridges` function.
It consumes the bridge records produced by O37 and updated by O39, associates
the O36 expansion records by source, closes each bridge group independently,
and returns `BottomBridge` region surfaces.

This slice does not complete project slicing. Public `slice_project` continues
to return `ProjectSlicingIncomplete` until the later external-surface, fill,
toolpath, motion, G-code, and processor slices are connected.

## Upstream source boundary

Primary source:

- `LayerRegion.cpp:310-351`: `merge_bridges`.

Direct dependencies:

- `LayerRegion.cpp:173-190`: `Bridge` and `group_id`;
- `Algorithm/RegionExpansion.hpp:85-92`: `RegionExpansionEx`;
- `ExPolygon.hpp:300-307`: contour-before-holes polygon conversion;
- `Surface.hpp:9-47`: `stBottomBridge` and default `Surface` metadata;
- `ClipperUtils.hpp:19,23-27,400-408` and
  `ClipperUtils.cpp:592-603`: Miter/3 defaults and flat-polygon morphological
  closing.

The pinned upstream checkout is commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Required behavior

For valid upstream-produced inputs, the Ares function must:

1. associate each contiguous `src_id` expansion run with its bridge;
2. resolve each bridge's root with the existing O37 grouping operation;
3. visit group roots and group members in ascending bridge index order;
4. append each member bridge contour, then holes, then that source's expansion
   contours and holes;
5. close each group independently with `+closing_radius` followed by
   `-closing_radius`, `JoinType::Miter`, and miter limit `3.0`;
6. emit one `RegionSurfaceKind::BottomBridge` surface per resulting ExPolygon;
7. use only the group root's calculated bridge angle; and
8. propagate Clipper errors without fallback or approximate output.

The function trusts internal invariants: source IDs and parent IDs are valid,
expansions are sorted into contiguous source runs by the later caller, group
roots are the lowest member IDs, root angles exist, and the closing radius is
positive whenever a nonempty group reaches closing. No validation is added for
these internal-only conditions.

## Rust design boundary

The Rust API may use ownership and temporary index/range tables instead of
copying OrcaSlicer's iterator-shaped `bridge_expansion_begin` field. Only final
geometry, ordering, surface metadata, bridge angle, and errors are parity
requirements. Pointer identity, field layout, iterator state, partial mutation
after failure, and behavior for invalid internal graphs are not part of the
Ares contract.

The flat polygon closing is composed from Ares's existing Clipper6 rewrite:
`offset_paths` for the positive pass and `offset_paths_tree` for the negative
pass. No second geometry engine, safety offset, scale conversion, or fixture
branch is allowed.

This slice creates no new non-negotiable architecture decision, so it does not
add an ARD. It stays within the existing platform-neutral core and source-cited
rewrite boundaries.

## Included and deferred behavior

Included:

- source-to-expansion association;
- group-root/member collection;
- contour/hole flattening;
- per-group flat-polygon closing;
- `BottomBridge` surface materialization with default metadata and root angle;
- removal of obsolete source-shape pins in the touched external-surface seam.

Deferred:

- `expand_bridges_detect_orientations` at `LayerRegion.cpp:395-437`;
- zone trimming after merged bridges;
- `LayerRegion::process_external_surfaces` flow/radius and custom-angle logic;
- lifecycle integration into `prepare_infill`;
- any new Option, fill path, toolpath, motion, G-code, or processor behavior;
- CLI/WASM project adapter changes and removal of the golden-test ignore.

## Tests and acceptance

Tests live in a separate `tests/merge_bridges.rs` module and verify behavior,
not source structure:

- empty input;
- one bridge with a hole and exact default surface metadata;
- a multi-member group using the root angle;
- source-specific expansion association without cross-source leakage;
- per-group closing, including disconnected outputs and narrow-gap merging;
- deterministic output order; and
- direct `ClipperError` propagation for out-of-range geometry.

The red-green loop must be recorded with the focused Nextest filter. The final
candidate must pass focused external-surface tests, the original ignored KSR
golden as an expected red progress probe, workspace Nextest, rustfmt, Clippy,
the <400 LOC audit, and the source-splitting macro audit. A fresh independent
reviewer must then assess requirement completeness, logic, edge cases, code
quality, test coverage, and actual execution. Findings are repaired and the
same reviewer repeats the review until approval.
