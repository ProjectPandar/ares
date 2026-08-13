# Task 22O.74 architecture decision record

## Status

Implemented, with exact-tree final gate counts and unconditional independent
review still pending. The Rust module and its focused/PRE/POST tests exist;
the module remains crate-private and lifecycle-inactive. Decision date:
2026-08-13.

## Decision

Replace O73's callable base seam with one crate-private, graph-native deep
module entry:

```rust
pub(in crate::project_slice) fn group_fills(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<GroupedFills, SliceError>;
```

There is no public or crate-private `group_fills_base` compatibility entry and
no `BaseGroupedFills` alias. Existing projection, coalescing, priority, and new
narrow-splitting files are private implementation phases behind `group_fills`.
This is not a shallow full-name wrapper around an independently callable base
module.

The seam continues to borrow the smallest prepared graph common to the two
upstream callers. It does not accept a caller-built layer view, raw surfaces,
an options bag, callbacks, traits, or test-only geometry inputs. It returns one
owned result and never mutates the prepared graph.

## Upstream rewrite boundary

The source target is pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- reuse O73's port of `src/libslic3r/Fill/Fill.cpp:216-346,829-1067`;
- add `Fill/Fill.cpp:349-595`, the `LineNode` anti-vibration filter;
- add `Fill/Fill.cpp:597-827`, `split_solid_surface` for line and non-line
  patterns;
- close `Fill/Fill.cpp:1069-1150` according to its actually reachable
  `InternalVoid` behavior;
- add `Fill/Fill.cpp:1152-1186`, the option gate and append/mutate tail; and
- retain `Fill/Fill.cpp:1213-1224,1377-1397` as the two future users of the
  complete result.

Directly reached dependencies include `Surface.hpp:35-114` for source surface
defaults and predicates, `Point.hpp:187-247`, `Polygon.hpp:101-102,162` and
`ExPolygon.hpp:451,483-484` for bounds and rotation,
`AABBTreeLines.hpp:269-300,300-361` for sorted intersections and inside/outside
classification, and
`ClipperUtils.hpp:320-322,344-387,364-370,414-449,509-520,543-550` with
`ClipperUtils.cpp:138-163,409-410,568-575,614-626,702-703,741-756,788-816`
for the exact bounding-box prefilter, offsets, opening, union, difference, and
intersection operations.

`Fill.cpp:1394-1407` is the source citation for a later O46 replacement: sparse
anchoring calls the same full `group_fills` and then keeps only `stInternal`
groups. O74 makes that replacement possible but does not perform it.

This is a source-cited `libslic3r` rewrite slice, not an Ares-owned fill
pipeline.

## Owned result and identity

`GroupedFills` owns exactly:

```rust
pub(in crate::project_slice) struct GroupedFills {
    pub(in crate::project_slice) surface_fills: Vec<SurfaceFill>,
    pub(in crate::project_slice) lock_region_param: LockRegionParam,
}
```

The existing `SurfaceFill`, representative metadata, lock sidecars, and
`SurfaceFillPattern::{Configured, ConcentricInternal}` remain the full domain
model. Grouped ExPolygons remain authoritative; no representative geometry is
introduced.

O74 adds source field `idx: usize` to `SurfaceFillParams`, in the source
position before `role_speed`. Base materialization assigns each comparator-
ordered group its zero-based ordinal, matching `Fill.cpp:1020-1024`. `idx` is
excluded from the strict-weak comparator and from grouping identity. A partial
narrow split copies the original params, so the appended group retains the
original group's `idx`; it is deliberately not the appended vector index and
need not be unique. The POST oracle must encode `params.idx`, not synthesize it
with `enumerate()`.

`SurfaceFillParams` still derives no equality, ordering, or hashing. O73's
manual comparator and all base group ordering remain unchanged.

## InternalVoid source truth

The apparent repair body at `Fill.cpp:1069-1150` is structurally dead for
values produced by the same source function:

1. `Fill.cpp:855-861` records that an `InternalVoid` exists but does not project
   params for it.
2. `Fill.cpp:1028-1051` again excludes `InternalVoid` from `surface_fills`.
3. `Fill.cpp:1086-1096` gathers `voids` only by scanning those
   `surface_fills`.
4. Consequently `voids.empty()` is always true and the guarded body at
   `1097-1149` is unreachable.

The exact observable port is therefore a no-op after the existing skip. O74
must not recover raw void polygons from `PreparedPostExternalSurfaces`, create
an `InternalVoid` group, grow solids into voids, or claim active repair parity.
It removes O73's `has_internal_voids` field from `ProjectedLayer` and from the
owned result; no replacement state is needed. A future upstream change that
makes the body reachable requires its own source-cited milestone.

## Narrow-splitting state and behavior

The private tail context contains only values actually read by the source
postpass: the effective object's
`detect_narrow_internal_solid_infill`, `PlannedLayer::id`, and
`CoordinateScale`. All remaining inputs come from each owned `SurfaceFill`:
representative kind and thickness-layer count, pattern, spacing, overlap,
angle, and authoritative ExPolygons. No raw predecessor surfaces or void
geometry reach the tail.

When the option is false, `group_fills` returns the O73 base behavior through
the full seam. When true, it snapshots the original group count and examines
only those groups, in order, whose representative kind is `InternalSolid`.
Groups appended during this pass are never recursively split.

The line-based set is exactly configured Rectilinear, Monotonic,
MonotonicLine, and AlignedRectilinear. Every other pattern takes the non-line
core route. The non-line route opens each flattened ExPolygon by one scaled
spacing, intersects the core with the original, and appends normal and narrow
differences in source order.

The line route preserves source arithmetic and control flow:

- scale spacing and the fixed 4 mm vibration threshold at the source cast
  points;
- add the f32 half-turn to the group angle, alternate by
  `layer_id / thickness_layers` except for AlignedRectilinear, and round
  rotated fixed coordinates exactly like source `polygons_rotate`;
- open by `2 * spacing` then `3 * spacing`, shrink by
  `0.5 * spacing - scaled(overlap)`, build vertical sections, and pair sorted
  inside intersections longer than one spacing;
- represent `LineNode` links with stable section/index pairs and retain
  `State { min_skips_taken, total_short_lines,
  initial_touches_long_lines, initialized }`, `is_removed`, maximum two long
  skips, and removal depth strictly greater than five;
- seed only live short nodes for each initial section, leaving live long-node
  state from the preceding outer pass intact while resetting the complete next
  section before propagation;
- preserve the source assignments at `Fill.cpp:558-559`, including the
  surprising write of `neighbour_total_short_lines` into
  `min_skips_taken`; and
- preserve reconstruction's `candidates_begin` geometry dereferences and its
  used-segment marker insertion at `724-742`; only the availability-search
  candidate advances.

Accepted predecessor paths merge the touch flag as logical AND, a node
propagates forward before its removal test, and backward-removal FIFO entries
are neither deduplicated nor guarded again when popped. Removal persists across
outer passes; state resets only at the source sites. Filter output retains
empty sections and original within-section order.

After tracing, the source safety union and original-area difference produce
candidate normal/narrow geometry. Narrow pieces that disappear under a half-
spacing shrink and touch normal fill after a 0.3-spacing expansion move back
to normal. If any narrow geometry remains, normal geometry is expanded by half
a spacing and clamped to the original.

Tail mutation is exact:

- no narrow output leaves the group unchanged;
- all-narrow output changes only the existing pattern to
  `ConcentricInternal`, retaining its original geometry and `idx`; and
- partial output replaces the original geometry with normal geometry and
  appends one group with copied params except the synthetic pattern, copied
  region ID/group and no-overlap polygons, and narrow geometry.

The appended representative has source-constructor defaults for fields not
assigned at `Fill.cpp:1175-1179`: kind `InternalSolid`, copied thickness,
`thickness_layers = 1`, `bridge_angle = -1`, and `extra_perimeters = 0`.
Appended groups remain after all base groups in original processing order.
They are not re-interned, re-sorted, coalesced, or priority-clipped, and lock
sidecars do not change.

## Error, atomicity, and performance

The existing graph-index/alignment contract remains internal: invalid indices
or broken prepared alignment are programmer errors, while an aligned absent
layer returns an empty `GroupedFills`.

All reached Clipper/offset coordinate failures in both base and narrow phases
map to the existing exact error:

```text
fill-grouping polygon coordinate is outside the supported Clipper range
```

Closed-path operations make open-path variants unreachable. Error order is
projection and lock Flow, base coalescing, base priority geometry, then narrow
groups and their ExPolygons in source order. Work is local and owned, so any
error drops the whole owned result and leaves the borrowed graph unchanged.

Base complexity remains `O(S log G)` plus Clipper work. The narrow pass visits
only the original `G` groups. For line splitting it precomputes adjacent-
section overlap edges once and stores indices rather than pointers; the
source propagation cost remains proportional to the visited node/edge suffixes
for each initial section. It does not add graph clones, canonicalization,
native services, unsafe code, filesystem access, or host-only behavior.

The production module split is:

- `group_fills.rs` — the only entry, graph/context resolution, phase order,
  and shared error mapping;
- existing `group_fills/{types,params,coalesce,priority}.rs` and
  `params/{projection,locked}.rs` — O73 behavior, result rename, exact `idx`
  assignment, and removal of void continuation state;
- `group_fills/narrow.rs` — option gate, original-count traversal, and result
  mutation for `Fill.cpp:1152-1186`;
- `group_fills/narrow/split.rs` — pattern dispatch and source geometric split;
- `group_fills/narrow/filter.rs` — `LineNode` and vibration propagation from
  `Fill.cpp:349-595`; and
- `group_fills/narrow/trace.rs` — source traced-polygon reconstruction from
  `Fill.cpp:693-777`.

`project_slice.rs` changes only the inactive-module reason from the completed
O73/O74 wording to the later source-cited lifecycle boundary; it adds no
lifecycle wiring or behavior.

Existing bounds, Clipper offset/boolean, and `LineDistanceTree` APIs cover the
reached reusable geometry. Source-specific checked rotation, scanline
construction, filtering, and reconstruction stay inside the narrow shards;
O74 adds no generic geometry facade.

Every new or changed Rust source and test file must remain below 400 LOC; split
before reaching the limit. No `include!` or generated source concatenation is
permitted.

## Lifecycle and replacement boundary

O74 remains crate-private and lifecycle-inactive. It adds no
`PreparedPostGroupFills`, `prepare`/`dispose` pair, lifecycle status, public
export, or O46 activation. Public slicing still disposes the O72 successor and
returns `ProjectSlicingIncomplete`.

O46's reduced
`prepare_infill/bridge_over_infill/sparse_anchoring/grouping.rs` remains a
temporary compatibility implementation during O74. It is neither called by
the new module nor used as a fallback. A later source-cited milestone may wire
`Fill.cpp:1394-1407` to the full result and then delete that reduced grouping.
Likewise, `Layer::make_fills` activation and `Fill.cpp:1192+` filler dispatch
remain later milestones.

The older public `infills::narrow_internal` rectangle heuristic and path
generator are a separate legacy scaffold. O74 neither calls nor wraps them;
their tests are not parity evidence for this grouping slice.

## TDD and oracle decision

All production-behavior tests cross `group_fills` using a prepared graph. O73
base/projection/coalescing/priority cases are retained through that same full
seam with the effective narrow-detection option set to false. Narrow focused
cases and the real KSR POST checkpoint cross the full seam with the option
true. No split/filter helper, caller-built `SurfaceFill`, or raw params seam is
exposed for tests. Encoder grammar tests may continue to test the encoder
directly.

The normative real-KSR POST result replays O38's audited fixed-MSVC STL 14.44
bridge-direction order across all 460 layer slots:

```text
536 groups
2,218 fill ExPolygons
152 fill holes
2,370 fill contour-plus-hole paths
110,610 fill points
2,928 no-overlap ExPolygons
metadata SHA-256 cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387
canonical geometry SHA-256 c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c
layer table SHA-256 8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2
```

The test encoder must emit the exact `stage post-narrow` token for POST and use
actual `params.idx`. Its disabled-option PRE witness retains the exact
`stage pre-narrow` token. Canonical geometry is oracle-only; production order
is never sorted to satisfy a digest. The Linux libstdc++ POST
metadata, geometry, and table hashes
`36aecdaf4d3bfb8dadcaf63a0d0d39f3a12ad9b0b0e1aad0c5a9ceab19ef2eff`,
`13d36da11e01e99840b1cf058003ad18c26c29bd8d6bb0d33af23c1b2ce4534c`,
and `15dd3f792d2a9176630e30c2170487c872a9b94eb637fdb6eb6a2841667ece5a`
are nonnormative provenance for the known predecessor tie-order difference.

The false-option full-seam regression retains O73's fixed-MSVC PRE totals and
three hashes. Pinned source and instrumentation checksums are provenance, not
behavior acceptance. The pin/length-only manifest test and constants used
only for that purpose were removed; passing a source hash cannot substitute
for focused behavior and POST result evidence.

The aggregate encoder deliberately retains the audited source-backed grammar;
it does not add `Flow::mm3_per_mm`. Exact `mm3_per_mm` preservation is a
separate Rust-only focused invariant. Rust tests compare the `f64::to_bits()`
literals for sparse, ordinary-bridge, thick-internal-bridge, and LockedZag
sidecar flows, and compare the copied partial-split value
`0x3fbb_4fc3_4000_0000` on both the original and synthetic group. These checks
do not redefine or extend the C++ oracle grammar, so the POST and PRE aggregate
hashes above remain unchanged.

Raw-order evidence complements the canonical aggregate geometry hash. These
digests are calculated before canonical sorting and therefore pin the actual
group/ExPolygon/path order returned by Rust:

- layer 1 metadata in the source-backed grammar:
  `b466abfd76770f5e776b9df3866cf12b07b836bee2a8a7ba721c66ae1f2851bf`;
- layer 1 authoritative geometry:
  `0938758d43750be165712735f6f5e1b6a1ae8fbb52a7f551b101118e1083c856`;
- layer 45 authoritative geometry:
  `33bf737e3d836096a20a821fcf1ace79dccda10973203408ba87ddee5ee25d64`;
  and
- layer 70 authoritative geometry:
  `7a8e9ec6e0aa2b1a8cd6bd8d1e9c261719b77168427f113fa051e7f5c551be71`.

The fixed-MSVC source-backed table provenance retains these exact rows:

```text
1\t2\t29\t0\t723\t5,5\t0,29\t5,5
45\t4\t75\t15\t29423\t6,5,0,4\t0,29,1,20\t10,5,6,4
70\t8\t70\t0\t626\t2,6,6,6,6,6,5,4\t0,0,0,0,0,0,29,20\t9,10,10,10,10,10,5,4
```

The layer-45 and layer-70 geometry digests above are calculated from the same
source-backed ordered raw records; they are not canonical-sort substitutes.

The current compiling-mutation record is intentionally narrower than the
original mutation wishlist. The public-seam corpus killed an identity
substitution for the vibration filter, the `4 mm -> 3 mm` threshold change,
the maximum-skip `2 -> 1` change, the exact two-skip `>= 2 -> > 2` change,
the removal-depth `> 5 -> >= 5` change, exact `4 mm` `< -> <=`, touch-back
removal, the final normal-expansion `0.5 * spacing -> 0` change, replacement
of the non-line closing delta by zero, hard-coded Normal scale, option/kind
gate inversions, dynamic appended traversal, direct layer parity, zeroed
source `idx`, and contour-only flattening. The KSR checkpoint specifically
killed the filter/threshold/skip/depth/touch-back/final-expansion subset; the
remaining kills came from graph-native focused tests. The two skip mutations respectively produced
2,223 / 2,375 / 110,582 and 2,217 / 2,369 / 110,597
fill-ExPolygon/path/point totals, distinct from the accepted POST totals.

The next-section reset removal, inclusive-Y-to-strict-Y change, source
`558-559` correction, `candidates_begin` correction, early-closure removal,
reconnection `< -> <=`, one-coordinate-unit non-line spacing, and premature
f32 scale/cast changes survived the current behavioral corpus. They remain
because pinned-source and static review require them; none is reported as a
killed mutation. FIFO/LIFO pending-order and duplicate-queue cases were
discharged as monotone-closure/static-review cases, not runtime kills.

## Final evidence — pending

This section is intentionally a placeholder. Fill it only from the final exact
candidate tree; implementation status does not imply these results.

- focused/dependency/workspace Nextest commands and exact counts: **pending**;
- Clippy, rustfmt, Tier-1 target, diff, LOC, static, Cargo-unchanged,
  zero-staged, and clean-Orca results: **pending**; and
- unconditional independent source/specification and standards review:
  **pending**.

## Included and deferred

Included: one full graph-native grouping seam; removal of the
base result/API; exact source `idx`; the proven InternalVoid no-op; complete
line and non-line narrow classification; exact mutation/append ownership; and
the fixed-MSVC POST result oracle.

Deferred: making the currently dead upstream void repair body reachable;
rotation-template grammar/PRNG; multi-region graph/coalescing; O46 replacement
and deletion; a prepared grouping lifecycle; filler creation including
`FillConcentricInternal`; extrusion entities; motion; G-code; CLI; and complete
golden parity.

## Rejected alternatives

- A `group_fills` wrapper that returns `group_fills_base` when detection is
  false leaves two production seams and is rejected.
- Carrying `has_internal_voids` or raw void polygons for an unreachable body
  invents behavior not present in the pinned source and is rejected.
- Reusing the legacy rectangle heuristic or O46's reduced grouping is a
  legacy fallback and is rejected.
- Recomputing appended `params.idx`, re-sorting appended groups, or running
  priority clipping again changes source identity/order and is rejected.
- Activating a lifecycle or replacing O46 in the same milestone exceeds the
  source slice and is rejected.
