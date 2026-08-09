# Task 22O.37 — Group bridge regions by overlapping expansions

## Status and source boundary

Released as implementation/documentation commits `a0caa5a`/`4d83d15`.
Exact-SHA Tier-1 run `31291016394` passed all five jobs and both browser
executions at `4d83d15832c7905d7ea9727d14c07c5a75eb7312`; its authoritative run
JSON is archived outside the repository at
`/tmp/task22o37-tier1-exact-sha.json`. O37 remains crate-private and inactive:
it adds no Option, lifecycle, adapter, golden expectation, or G-code byte, and
public slicing still consumes O26 before returning `ProjectSlicingIncomplete`.
Exact predecessor O36 is released as implementation/documentation commits
`b546e6f`/`3e927ed`, with successful exact-SHA run `31280579891` at
`3e927ed569d3db8d6f5c08b7843fb049fcc86412`. Pinned Orca remains v2.4.2
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Port only the next coherent translation-unit-local boundary in
`OrcaSlicer/src/libslic3r/LayerRegion.cpp`:

- `Bridge` at lines 174-179;
- `group_id` at lines 181-190;
- `get_grouped_bridges` at lines 192-260.

The exact upstream dependencies used by that body are the inclusive
`BoundingBoxBase::overlap` predicate in `BoundingBox.hpp:55-58` and the
single-`Polygon` NonZero intersection overload in
`ClipperUtils.hpp:496` / `ClipperUtils.cpp:696-697`. Ares already owns the
corresponding `BoundingBox` min/max representation and
`intersection_polygons_paths` Clipper kernel; O37 composes them without adding
a second geometry engine.

Deferred: `detect_bridge_directions` at `LayerRegion.cpp:262-308` and its direct
`detect_bridging_direction(const Lines &, const Polygons &)` dependency at
`BridgeDetector.hpp:75-119`, `merge_bridges` at `LayerRegion.cpp:310-351`,
`expand_bridges_detect_orientations` at lines 398-437, and the active
`LayerRegion::process_external_surfaces` body at `LayerRegion.cpp:486-623` with
its declaration at `Layer.hpp:86`, bridge-direction line generation and angle
selection, closing/merged bridge
surfaces, construction of expansion zones from flows and Options, lifecycle
activation, public adapters, fill, toolpath, seam, motion, serialization,
G-code, post-processing, and normalized KSR parity.

## Ares destination and compatibility boundary

Extend only the inactive private
`project_slice::prepare_infill::external_surfaces` module. Add the
source-shaped record:

```rust
pub(in crate::project_slice) struct Bridge {
    pub(in crate::project_slice) expolygon: ExPolygon,
    pub(in crate::project_slice) group_id: u32,
    pub(in crate::project_slice) bridge_expansion_begin: usize,
    pub(in crate::project_slice) angle: Option<f64>,
}
```

The C++ `bridge_expansion_begin` stores a const iterator initialized to
`bridge_expansions.end()`. Rust stores the equivalent index sentinel
`bridge_expansions.len()`; the deferred merge helper will replace it with the
first matching expansion index. This avoids self-referential borrowing without
changing the iterator-position contract. `angle` starts as `None`, matching
`std::nullopt`.

Add the namespaced helpers:

```rust
pub(in crate::project_slice) fn group_id(
    bridges: &mut [Bridge],
    src_id: u32,
) -> u32;

pub(in crate::project_slice) fn get_grouped_bridges(
    bridge_expolygons: Vec<ExPolygon>,
    bridge_expansions: &[RegionExpansionEx],
) -> Result<Vec<Bridge>, ClipperError>;
```

Ares returns `ClipperError` because its safe indexed Clipper kernel expresses
the upstream intersection failure boundary explicitly. O37 adds no scale
parameter: the inputs are already scaled, and this source function performs no
scaling. Do not add a request object, generic/trait overload, public export,
validation result, alternate grouping representation, or production test seam.

## Frozen bridge initialization and root traversal

`get_grouped_bridges` first allocates result capacity from
`bridge_expansions.len()`, exactly like the source reserve. It then consumes
`bridge_expolygons` in input order and pushes one `Bridge` per source with:

- the original moved `ExPolygon` and point buffers;
- `group_id = index as u32` in source order;
- `bridge_expansion_begin = bridge_expansions.len()`;
- `angle = None`.

The cast is the direct Rust equivalent of the source `uint32_t group_id++`.
Inputs larger than the representable or allocatable domain are not validated.
Capacity is structural evidence only and must not be instrumented in
production merely to make allocation behavior observable.

`group_id` follows parent links literally:

1. read `bridges[src_id as usize].group_id`;
2. while that value differs from `src_id`, replace `src_id` by the parent and
   read the next parent;
3. assign the root's `group_id` to itself and return it.

Do not perform full path compression, recursion, cycle detection, bounds
validation, sorting, or remapping. A malformed index remains a trusted internal
panic. Valid records produced by `get_grouped_bridges` start as roots, and the
lower-root union rule cannot create a cycle.

## Frozen boundary windows, overlap, and union order

Process `bridge_expansions` as consecutive adjacent boundary-ID windows, without
sorting or regrouping. For each window:

1. Cache one contour-only bounding box per expansion in the same order. O30
   output contours are trusted nonempty internal geometry; O37 does not add a
   public empty-contour validation boundary.
2. Enumerate each ordered pair `(current, candidate)` with the candidate after
   the current in that same window.
3. Short-circuit in this exact order:
   1. skip equal `src_id`;
   2. require inclusive bounding-box overlap by inlining in
      `group_bridges.rs` the exact four strict separation comparisons from
      `BoundingBoxBase::overlap`, using existing `BoundingBox::min`/`max`;
   3. call `intersection_polygons_paths` exactly once on the two contours,
      ignore both ExPolygon hole lists, and require a nonempty intersection.
4. For an intersecting pair, resolve each current root with `group_id`.
5. Point the higher root at the lower root. If the roots are equal, preserve
   the source `else` no-op assignment rather than adding a special branch.

Outer result order and every moved source ExPolygon remain unchanged. The raw
returned `group_id` fields are the exact parent forest at the end of ordered
pair processing; do not normalize every field to its final root. Later source
code deliberately calls `group_id` again when it needs a root.

The bounding-box predicate is inclusive, but edge-only polygon contact still
groups only if the NonZero Clipper intersection emits a nonempty area path.
Different boundary windows never interact, even when contours overlap. Holes
are deliberately irrelevant: only expansion contours participate in both bbox
and intersection checks.

Do not use public `ExPolygon` intersection semantics, union the contours,
inspect holes, sort by `(boundary_id, src_id)`, compact source records, clone
point buffers, retry, fall back, map errors, return partial groups, or call O36
inside O37.

## Ownership, assertions, and error order

`bridge_expolygons` is consumed once and moved into the local result.
`bridge_expansions` and all their point buffers are borrowed and unchanged.
Only local `Bridge.group_id` fields mutate. Successful output owns the original
source point buffers; no O37 geometry clone is permitted.

Bounding boxes for a complete boundary window are cached before any pair in
that window is intersected. Pair checks then occur in nested source order. On
the first Clipper failure, `?` returns that exact `ClipperError`; the partially
mutated local result is dropped and no partial vector escapes. Earlier windows
and pairs therefore determine error precedence but never mutate either input.

Source-supported O36/O30 input has valid source IDs and nonempty contours.
O37 intentionally trusts those invariants. An out-of-range `src_id` that reaches
root resolution and a malformed empty expansion contour remain internal panics;
no defensive fallback or stable public error is added. Equal-source and
disjoint-bbox pairs short-circuit before Clipper and root indexing exactly as in
the source condition.

## Tests and TDD

Use one ordinary test module under the external-surface test tree. Every
committed vector is a behavior-named Rust literal; source text/hash/line
pinning, serialized oracle blobs, `include!`, and `include_bytes!` are
forbidden. Any raw C++ helper, generated output, or serialized diagnostic stays
under `/tmp`.

Capture a real compiling RED against a temporary body that only initializes and
returns one source-shaped `Bridge` per input without performing grouping.
Function-pointer shape is not RED. Record chronological RED separately from
post-hoc mutation evidence and disclose stub-equivalent tests such as empty or
nonoverlapping inputs.

Focused tests must cover:

- zero sources and zero expansions;
- source initialization with zero expansions, including exact source order,
  root IDs, end-index sentinel, `None` angles, and moved point-buffer identity;
- one boundary window with multiple distinct sources and complete overlapping
  contours, proving exact pair order and lower-root union;
- transitive grouping and the raw parent forest/root traversal without adding
  full path compression; use an intentionally unsorted internal expansion
  vector when needed to expose the source's no-sort behavior;
- equal-source records, disjoint bounding boxes, and separate boundary windows
  remaining ungrouped in the exact short-circuit order;
- contours that overlap while holes differ, proving holes are ignored;
- complete multiple-boundary source and expansion vectors from a pinned original
  Orca Debug/`NDEBUG` helper, including result geometry, IDs, sentinel, angle,
  parent forest, and output ordering;
- first and later real Clipper coordinate failures after bbox overlap, with no
  returned partial result and borrowed expansion bytes unchanged;
- an invalid Clipper contour skipped by equal-source or disjoint-bbox
  short-circuit, proving those checks precede intersection;
- trusted malformed source-ID and empty-contour panic behavior where reachable;
- exact helper/result shape and crate-private visibility.

Run the pinned original Orca CLI on the KSR 3MF in a disposable environment as
the required project E2E, retaining only exit/result metadata under `/tmp` and
never reading or committing generated G-code. Build a disposable helper against
the exact pinned source/kernel in Debug and `NDEBUG`; require byte-identical
behavior-named vectors before transcribing reviewed Rust literals.

Post-hoc mutation candidates include sorting or regrouping expansions, grouping
across boundary windows, using holes, omitting or changing the inclusive bbox
prefilter, swapping intersection operands, swallowing intersection errors,
compressing every path, selecting the higher root, normalizing all output IDs,
cloning source geometry, changing the end sentinel, initializing an angle,
validating malformed IDs, and changing signature/visibility. Apply one mutation
at a time and restore exact bytes. Capacity reservation, equal-root no-op, and
some operand swaps may be behaviorally equivalent on valid fixtures; report
them as structural/equivalent survivors instead of false kills.

## Files, limits, and prohibitions

Allowed Rust edits only:

- `crates/ares-core/src/project_slice/prepare_infill/external_surfaces.rs`:
  register/reexport the private grouping module/types and add exact function-
  shape assertions;
- `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/types.rs`:
  add only the source-shaped `Bridge` record;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/group_bridges.rs`:
  the sole O37 grouping body, at most 220 physical lines;
- `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests.rs`:
  register the ordinary test shard and shape constants;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/group_bridges.rs`:
  focused O37 tests, at most 300 physical lines.

Existing `geometry.rs`, bounding-box/Clipper kernels, O35/O36 production and
test files, manifests/lock/dependencies, lifecycle/stage/cleanup/predecessor,
`project_slice.rs`, adapters, workflow, golden test, fixture expectations, and
G-code remain untouched. Allowed docs are this spec/plan, O36 spec/plan release-
state corrections, `docs/roadmap.md`, and
`docs/architecture/option-parity-v4.md`. No ARD change.

Every Rust file remains below 400 physical lines. No broad lint allowance,
`unsafe`, FFI, filesystem/native thread, platform branch, public API/hook,
hard-coded fixture identity/name/hash/layer-count/geometry branch, reference-
G-code read, binary oracle, legacy fallback, source concatenation, source
pinning test, second clipping engine, or dependency change.

## Implemented evidence state

The initialization-only stub compiled and ran ten focused tests: six
body-dependent witnesses failed at the stub seam and four initialization,
traversal, or short-circuit cases were disclosed as stub-equivalent passes. The
frozen implementation then passed focused debug/release 10/10. A disposable
pinned-Orca CLI sliced the KSR project successfully to a nonempty generated
G-code that was deleted without content ingestion; a linked original
`get_grouped_bridges` helper passed 45 assertions and emitted byte-identical
Debug/`NDEBUG` complete vectors.

The one-at-a-time post-repair campaign kills thirteen runtime mutations,
compiler-rejects two API/field mutations, and truthfully records the strict-to-
inclusive bbox comparison substitution as behaviorally equivalent behind the
area-producing NonZero intersection. All five repaired hashes restore exactly.
A review-required private pair-helper extraction removes Clippy excessive
nesting without changing tests or operation order. The repaired body/test
shards are 96/289 LOC; O36/O35/O28/O30/RegionExpansion/external-surface/
PolyTree/boolean-path/offset/O26 regressions pass 6/13/39/6/92/25/15/11/62/3.
Both repaired initial implementation rereviews return `VERDICT: APPROVE`.
O37 has no production caller and public slicing still ends after O26 with
`ProjectSlicingIncomplete`.

## Verification, review, release, and rollback

Require focused debug/release O37, O36/O35/O28/O30 and complete
RegionExpansion/external-surface regressions, PolyTree/boolean-paths/offset,
O26 lifecycle, workspace Nextest, all-target check, warning-denying Clippy,
rustfmt, four WASM checks, two optimized builds, export/JavaScript audit, and
two Playwright runs. If local Chromium lacks `libglib-2.0.so.0`, record each
failure exactly and require both exact-SHA CI executions; never label it a pass.

Static-audit the exact allowlist, ordinary modules, LOC, crate-private
visibility, ownership/error order, absence of all forbidden patterns and staged
or generated artifacts. Rehearse disposable rollback to exact released O36
`3e927ed...` and prove the primary candidate unchanged.

Fresh independent six-dimensional and default-model OpenCode reviewers must
approve spec, plan, implementation, and final documentation. Every review repair
requires affected and complete exact-candidate verification, refreshed evidence,
and both rereviews against the same diff.

Use separate Conventional Commits for implementation and documentation, push
only approved files, and require Tier-1 `headSha` to equal the pushed
documentation SHA with exactly five successful jobs and both browser executions.
O37 remains inactive and does not change any Option, public slicing, adapter,
golden expectation, or G-code byte. Public slicing must still consume O26 and
return `ProjectSlicingIncomplete`.

The next bounded source candidate after O37 is
`detect_bridge_directions` at `LayerRegion.cpp:262-308`, together with its
direct `detect_bridging_direction(const Lines &, const Polygons &)` dependency
at `BridgeDetector.hpp:75-119`. `merge_bridges`, complete external-surface
orchestration, fill/toolpath/motion/G-code remain deferred.
