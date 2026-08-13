# Task 22O.74 — full fill grouping

## Status

Implemented specification. O74 Rust and its focused/PRE/POST tests exist, while
exact-tree final gate counts and unconditional independent review remain
pending. The implementation is crate-private and does not claim lifecycle or
O46 activation. Specification date: 2026-08-13.

## Goal and source boundary

Complete pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s `group_fills` as one deep Rust
module. Reuse O73's admitted base port of
`OrcaSlicer/src/libslic3r/Fill/Fill.cpp:216-346,829-1067` and add exactly:

- `Fill/Fill.cpp:349-595` — vibration filtering and `LineNode` state;
- `Fill/Fill.cpp:597-827` — line/non-line `split_solid_surface`;
- `Fill/Fill.cpp:1069-1150` — the observable, structurally dead
  `InternalVoid` continuation;
- `Fill/Fill.cpp:1152-1186` — option-gated narrow-group mutation/append; and
- source caller context at `Fill/Fill.cpp:1213-1224,1377-1397`.

`Fill.cpp:1394-1407` proves that O46's future sparse-anchoring caller obtains
the same complete `group_fills` result and then filters for `stInternal`.
Replacement of O46 is explicitly outside this milestone.

Reached support behavior comes from `Surface.hpp:35-114`,
`Point.hpp:187-247`, `Polygon.hpp:101-102,162`, `ExPolygon.hpp:451,483-484`,
`AABBTreeLines.hpp:269-361`, and
`ClipperUtils.hpp:320-322,344-387,364-370,414-449,509-520,543-550` with
`ClipperUtils.cpp:138-163,409-410,568-575,614-626,702-703,741-756,788-816`.
Existing O73 Flow/config/surface/Clipper dependencies remain part of the
complete module.

## Destination interface

`project_slice::group_fills` exposes exactly one crate-private entry:

```rust
pub(in crate::project_slice) fn group_fills(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<GroupedFills, SliceError>;
```

`object_index` remains an aligned print-object occurrence index and
`layer_index` remains the planned/record slot. Invalid indices and broken
alignment are internal programmer errors. An aligned absent layer returns an
empty result. The prepared graph is borrowed and remains unchanged on both
success and error.

O74 removes the callable `group_fills_base` seam and the `BaseGroupedFills`
type. It must not preserve them via aliases, re-exports, wrappers, feature
flags, or test-only calls. Base phases are private implementation details of
the one full operation.

The owned result is:

```rust
pub(in crate::project_slice) struct GroupedFills {
    pub(in crate::project_slice) surface_fills: Vec<SurfaceFill>,
    pub(in crate::project_slice) lock_region_param: LockRegionParam,
}
```

`GroupedFills` has no lifecycle marker, `has_internal_voids`, raw-void
inventory, graph borrow, or deferred-operation flag. `SurfaceFill` continues
to own region identity, representative metadata, authoritative ExPolygons,
params, ordered region group, and no-overlap ExPolygons. Lock sidecars remain
owned and ordered exactly as O73 specifies.

## Source parameter identity

Add `idx: usize` to `SurfaceFillParams` immediately before `role_speed`,
matching `Fill.cpp:257-260`. After comparator-equivalent interning and before
the source-order coalescing rescan, assign each comparator-ordered base params
record its zero-based ordinal exactly as `Fill.cpp:1020-1024` does.

The strict-weak comparator deliberately excludes `idx`. It also retains all
other O73 exclusions, ordinary float comparisons, explicit pattern/role ranks,
and comparator equivalence. `SurfaceFillParams` must not derive `Eq`, `Ord`, or
`Hash`.

The result keeps `idx` because it is source-visible POST metadata. Base,
unchanged, and all-narrow groups retain their comparator-order ordinal. A
partial appended group copies its source group's value; the appended vector
position must not overwrite it, so two result groups may share `params.idx`.

Oracle encoders must serialize `fill.params.idx`. Synthesizing `idx` from the
current group enumeration is forbidden.

## InternalVoid requirement

Port the observable source, not the comment's apparent intent.
`Fill.cpp:855-861` observes and skips `InternalVoid`; `1028-1051` excludes it
from `surface_fills` again; `1086-1096` tries to collect void polygons only
from those already filtered groups. The `!voids.empty()` guard at `1097`
cannot succeed for a result constructed by this function.

Therefore O74 continues excluding every `InternalVoid`, executes no geometry
repair, removes O73's internal and returned `has_internal_voids` state, and
never reads raw void ExPolygons or changes a solid group from them.

A focused test must include a source-shaped void plus printable groups and
prove the full result is exactly the same as the same graph without the void,
apart from predecessor graph contents that remain borrowed. No test-only
injection may fabricate an impossible `surface_fills` void group.

## Narrow option and traversal

Read `detect_narrow_internal_solid_infill` from the already resolved effective
object options in `PreparedPostExternalSurfaces`. Do not introduce a raw
option parameter or parse it again.

If false, the complete seam produces O73 behavior. If true, snapshot
`surface_fills.len()`, visit only that original prefix in order, skip every
representative kind except `InternalSolid`, and split from owned group geometry
using planned `layer.id`, coordinate scale, params, and metadata. Never process
an appended narrow group again.

No base group may be re-sorted or re-coalesced after the tail. The lock maps
are not recomputed or extended.

## Non-line split

The line-based patterns are exactly configured Rectilinear, Monotonic,
MonotonicLine, and AlignedRectilinear.

Every other pattern variant follows `Fill.cpp:605-629`;
`ConcentricInternal` is not present in the original group prefix.

For each source ExPolygon in order, flatten contour then holes, scale spacing
at the source cast, open by one spacing, and intersect the opening with the
original paths. An empty core appends the original to narrow output; otherwise
union the core, then append original-minus-core to narrow and
original-intersect-core to normal in source operation order.

Do not substitute the legacy rectangle-width test, a bounding-box heuristic,
medial-axis width, or filler path output.

## Line split

The line route must preserve `Fill.cpp:632-814` control flow, source casts,
operation order, and stable order of intermediate collections.

Scale spacing once. Compute base angle as `params.angle` plus the source f32
`PI / 2`; unless AlignedRectilinear, alternate with
`layer_id / representative.thickness_layers`. The aligning angle is
`-base_angle + PI`. Rotate flattened polygons using the source fixed-coordinate
rounding behavior.

For each source ExPolygon, obtain rotated bounds; intersect filled area with an
opening of `2 * spacing` then `3 * spacing`; shrink by
`spacing * 0.5 - scaled(overlap)`; build the line-distance tree from
contour-before-hole walls; process the source's `n_vlines` source-coordinate
vertical lines; collect sorted adjacent pairs whose midpoint is inside and
length is strictly greater than spacing; then filter the sections. The source
also allocates one terminal line that no later loop reads, so the observable
port deliberately omits that dead allocation.

Use the exact inclusive Y-overlap relation. Each `LineNode` owns its line,
stable previous/next section-node indices, `is_removed`, and state fields
`min_skips_taken: i32`, `total_short_lines: i32`,
`initial_touches_long_lines: bool`, and `initialized: bool`. The threshold is
scaled 4 mm, maximum allowed skips is two, and removal uses
`total_short_lines > 5`, not `>= 5`.

For each initial section, initialize only nonremoved short lines, reset the
next section before propagation, visit neighbors in recorded order, and
propagate removals backward through the stored queue order. Preserve these
two observable source peculiarities:

- at `Fill.cpp:558-559`, an already initialized neighbor first assigns
  `max(current min_skips_taken, neighbour_total_short_lines)` and then
  `min(that result, neighbour_min_skips_taken)` to `min_skips_taken`; it does
  not update `total_short_lines`; and
- at `724-742`, the reconstruction loop advances only the candidate used to
  search for availability; both appended geometry and the used-segment marker
  stay at `candidates_begin`. Neither may be silently corrected to the
  advancing candidate.

Live long nodes retain stale state when a new outer initial section seeds only
short nodes. Touch-state merging is logical AND; forward propagation precedes
the removal test. Backward FIFO entries are not deduplicated and have no
dequeue-time removed guard. Removal persists across outer passes, and output
retains empty sections plus original line order.

Reconstruction expands every section line by half a spacing in Y, retains
current traced lows/highs in order, closes disconnected traces exactly, and
appends unmatched segments in section order. Rotate reconstructed polygons
back, safety-union them, and difference original fill geometry to obtain
narrow candidates.

For each candidate narrow ExPolygon in order, shrink by half a spacing. If it
vanishes, expand by `0.3 * spacing`, bounding-box prefilter the normal fill,
and test intersection. A touching piece moves back to normal and is removed
from narrow. If no narrow pieces remain, return empty outputs. Otherwise
expand normal fill by half a spacing, intersect with the original fill, and
return it beside the remaining narrow output.

## Result mutation and ownership

Apply `Fill.cpp:1163-1183` exactly:

- `narrow.is_empty()` — leave the group byte-for-byte unchanged;
- `normal.is_empty()` — set only `params.pattern` to
  `ConcentricInternal`; keep original geometry, metadata, identity, region
  state, no-overlap, and index; or
- both nonempty — replace original ExPolygons with normal, copy params and set
  only its pattern, then append a new `SurfaceFill` with narrow ExPolygons.

For partial splits, copy `region_id`, `region_id_group`,
`no_overlap_expolygons`, and the complete params including `idx`. Model the
new representative exactly as source constructor plus assignments:

```text
kind = InternalSolid
thickness = original representative thickness
thickness_layers = 1
bridge_angle = -1
extra_perimeters = 0
```

Appending is ordered by original group traversal. Do not clone the full
original representative, derive an appended index, re-run priority clipping,
or interleave the new group into comparator order.

## Errors, atomicity, and portability

Return `SliceError` and no partial `GroupedFills`. The existing full grouping
coordinate error remains exactly:

```text
InvalidInput("fill-grouping polygon coordinate is outside the supported Clipper range")
```

Use it for all reached base and narrow Clipper/offset range failures. Open-path
errors remain unreachable for these closed paths. Preserve error phase order:
projection/lock Flow, coalescing, priority, then narrow groups and geometry in
source order.

All work is local and owned. No mutation may escape before success. The module
must remain deterministic, parallel-call safe, safe Rust, and portable across
browser WASM, Windows, macOS, and Linux. It must use no filesystem, threads,
UI, OpenGL, terminal, or native-only service.

Base performance remains `O(S log G)` plus Clipper work. Narrow traversal is
over the original groups only. Adjacent-section overlap links are built once;
the source propagation/reconstruction work is retained without rescanning the
prepared graph or cloning it.

## TDD contract

All behavioral tests call `group_fills` from a prepared graph. No private
filter, trace, split, comparator, priority, raw params, or caller-built fill
entry is exposed for tests.

Retained O73 tests now use the full seam with effective
`detect_narrow_internal_solid_infill = false`. They preserve projection,
comparator, coalescing, priority, sidecar, absent-layer, repeatability,
immutability, error, and PRE-oracle behavior while proving no callable `_base`
entry remains.

Implemented graph-native tests cover false-option PRE identity, the
InternalVoid no-op, non-line full/no/partial cores and ordered topology, all
four line patterns and layer alternation, all three result branches, appended
defaults/order, copied duplicate `params.idx`, region/no-overlap ownership,
unchanged locks, natural range failure, atomicity, immutability, repeatability,
and the all-layer POST checkpoint. This list describes the tests that exist;
it does not claim that every source quirk received an independently
discriminating focused witness.

Current compiling-mutation evidence is exact. The public-seam corpus killed
the vibration-filter identity substitution, `4 mm -> 3 mm`, maximum skips
`2 -> 1`, exact two-skip `>= 2 -> > 2`, removal depth `> 5 -> >= 5`, exact
`4 mm` `< -> <=`, touch-back removal, final normal expansion
`0.5 * spacing -> 0`, a zero non-line closing delta, and hard-coded Normal
scale. The KSR checkpoint specifically killed the filter/threshold/skip/
depth/touch-back/final-expansion subset; graph-native focused tests killed
the exact-4-mm, zero-closing-delta, and hardcoded-scale changes. The two skip mutations respectively produced
2,223 / 2,375 / 110,582 and 2,217 / 2,369 / 110,597
fill-ExPolygon/path/point totals.

The next-section reset removal, inclusive-Y-to-strict-Y change, `558-559`
correction, `candidates_begin` correction, early-closure removal,
reconnection `< -> <=`, one-coordinate-unit non-line spacing, and premature
f32 scale/cast changes survived the current corpus. They are retained by pinned-source
and static review, not reported as kills. FIFO/LIFO pending-order and
duplicate-queue cases are monotone-closure/static-review cases and likewise
are not counted as runtime kills.

Test geometry must be supplied as source-shaped `RegionSurface` records and
options in the prepared graph. Directly testing a private split function or
using the older `infills::narrow_internal` scaffold does not meet acceptance.

## KSR oracle and provenance

The normative acceptance is the fixed-MSVC-order POST result over all 460
ordered layer slots, including empty layers 260-459:

| measure | required POST value |
|---|---:|
| groups | 536 |
| fill ExPolygons | 2,218 |
| fill holes | 152 |
| fill contour-plus-hole paths | 2,370 |
| fill points | 110,610 |
| no-overlap ExPolygons | 2,928 |
| nonempty / empty layers | 260 / 200 |
| metadata SHA-256 | `cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387` |
| canonical geometry SHA-256 | `c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c` |
| layer table SHA-256 | `8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2` |

Fill totals exclude the no-overlap section; the canonical geometry digest
includes both. Metadata must encode actual `params.idx`, including the copied
base identity on appended narrow groups. POST metadata emits exact token
`stage post-narrow`; the disabled-option PRE witness emits exact token
`stage pre-narrow`. This stage selection belongs only to the test encoder and
is not production lifecycle state.

The POST result must replay O38's audited fixed-MSVC STL 14.44 predecessor
direction-map order. The Linux libstdc++ source run is retained only as
nonnormative provenance:

```text
metadata 36aecdaf4d3bfb8dadcaf63a0d0d39f3a12ad9b0b0e1aad0c5a9ceab19ef2eff
canonical geometry 13d36da11e01e99840b1cf058003ad18c26c29bd8d6bb0d33af23c1b2ce4534c
layer table 15dd3f792d2a9176630e30c2170487c872a9b94eb637fdb6eb6a2841667ece5a
```

This platform variant does not authorize bridge-angle normalization or
production sorting. Canonicalization remains an oracle-only view.

The source-backed metadata grammar remains unchanged and deliberately omits
`Flow::mm3_per_mm`. Rust-only focused assertions separately preserve exact
`f64::to_bits()` values for sparse, ordinary-bridge,
thick-internal-bridge, and LockedZag sidecar flows. The partial-split ownership
test also requires `0x3fbb_4fc3_4000_0000` on both the original and synthetic
group. These are exact Rust seam invariants, not additions to the C++ oracle
grammar, so they do not change either aggregate hash triplet.

The implementation also pins noncanonical, raw-order POST witnesses:

- layer 1 metadata:
  `b466abfd76770f5e776b9df3866cf12b07b836bee2a8a7ba721c66ae1f2851bf`;
- layer 1 authoritative geometry:
  `0938758d43750be165712735f6f5e1b6a1ae8fbb52a7f551b101118e1083c856`;
- layer 45 authoritative geometry:
  `33bf737e3d836096a20a821fcf1ace79dccda10973203408ba87ddee5ee25d64`;
  and
- layer 70 authoritative geometry:
  `7a8e9ec6e0aa2b1a8cd6bd8d1e9c261719b77168427f113fa051e7f5c551be71`.

These hashes are computed before canonical sorting and preserve production
group/ExPolygon/path order. Fixed-MSVC source-backed table provenance retains:

```text
1\t2\t29\t0\t723\t5,5\t0,29\t5,5
45\t4\t75\t15\t29423\t6,5,0,4\t0,29,1,20\t10,5,6,4
70\t8\t70\t0\t626\t2,6,6,6,6,6,5,4\t0,0,0,0,0,0,29,20\t9,10,10,10,10,10,5,4
```

The layer-45 and layer-70 geometry hashes use these same source-backed ordered
raw records, not canonical-sort substitutes.

With narrow detection false, the same full seam must retain the O73 PRE
checkpoint: 477 groups, 1,882 fill ExPolygons, 174 holes, 2,056 paths, 107,540
points, 2,547 no-overlap ExPolygons, and hashes
`a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900`,
`062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af`,
and `ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721`.
PRE is a disabled-option behavior regression, not O74's primary success value.

Pinned source and instrumentation metadata may be recorded as provenance but
are not acceptance. The manifest-length test and constants whose only asserted
property was a commit/hash length were removed. A clean pinned source tree and
source citations remain review inputs, while focused behavior plus POST output
are the release evidence.

## File boundary

Production behavior changes are limited to the existing group module:

- `project_slice/group_fills.rs` — only full entry and phase ordering;
- `project_slice/group_fills/types.rs` — `GroupedFills` and exact `idx`;
- existing `params.rs`, `params/{projection,locked}.rs`, `coalesce.rs`, and
  `priority.rs` — private base reuse, index assignment, and void-state removal;
- `project_slice/group_fills/narrow.rs` — gate and output mutation;
- `project_slice/group_fills/narrow/{split,filter,trace}.rs` — bounded source
  algorithm shards, including source-specific rotation and scanlines.
- `project_slice.rs` changes only the inactive-module reason from the completed
  O73/O74 wording to the later source-cited lifecycle boundary; it adds no
  lifecycle wiring.

Existing bounds, Clipper offset/boolean, and line-distance APIs cover the
reached reusable geometry; do not add a generic geometry facade.

Tests stay under
`project_slice/tests/prepare_infill/group_fills.rs` and its existing directory,
including the narrow/oracle shards. Every changed or new Rust source and test
file stays below 400 LOC. The implementation uses ordinary modules, without
`include!`, generated concatenation, or Cargo changes.

## Lifecycle and legacy restrictions

O74 creates no `PreparedPostGroupFills`, no lifecycle status, no
`prepare`/`dispose`, and no public API. `slice_project_sync` remains at the O72
incomplete sink.

O46's reduced sparse-anchoring grouping remains in place for this milestone.
O74 must not call it, wrap it, or replace it. The future replacement owner
will wire the full result at source `Fill.cpp:1394-1407` and delete the reduced
implementation in one source-cited change.

The older `infills::narrow_internal` rectangle/path implementation is not
reused and supplies no fallback. Likewise there is no fallback to
`group_fills_base`. No legacy shim is permitted.

## Included and deferred

Included: full graph-native grouping API; exact source `idx`; O73 base behavior
behind that API; InternalVoid's observable no-op; complete line/non-line
narrow detection; exact result mutation/append ownership; POST oracle and
provenance; and no-fallback/static gates.

Deferred: any future source change that makes InternalVoid repair reachable;
rotation-template grammar/PRNG; multi-region grouping; O46 replacement;
grouped-fill lifecycle activation; `Layer::make_fills`; generator dispatch;
`FillConcentricInternal` extrusion behavior; motion; G-code; CLI; and complete
golden parity.

## Final evidence — pending

The implementation and the oracle/mutation evidence above exist. Release
closure still requires one exact candidate tree to record focused, dependency,
and workspace Nextest commands and counts; strict lint/format/diff and every
Tier-1 result; LOC/static/Cargo-unchanged/zero-staged/clean-Orca checks; and an
unconditional independent source/specification and standards approval.

This section is intentionally a placeholder for those exact results. No final
command count or review verdict is inferred from implemented status.
