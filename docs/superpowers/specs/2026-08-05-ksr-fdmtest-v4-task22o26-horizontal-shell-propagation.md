# Task 22O.26 — Horizontal-shell propagation Spec

## Status

Implemented from Ares O25 predecessor
`251b53bf101d8a3f72b96cf540ea4a80ef7cb917` against pinned OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The complete pre-implementation
spec and plan received literal approval from an independent reviewer and a
separate default-model OpenCode reviewer, and O25 exact-SHA Tier-1 run
`31028569875` is fully green. O26 local implementation evidence is frozen below;
final independent six-dimensional and default-model OpenCode reviews approve
the implementation. Exact pushed-SHA Tier-1 remains the release gate.

## Upstream source boundary

This milestone rewrites the complete executable remainder of
`PrintObject::discover_horizontal_shells` after O25:

- the `evstAll` skip at
  `OrcaSlicer/src/libslic3r/PrintObject.cpp:3974-3976`;
- Z capture, Top/Bottom/BottomBridge order, shell-count gates, and source
  gathering at `PrintObject.cpp:3978-4008`;
- directional count-or-thickness neighbor traversal at
  `PrintObject.cpp:4011-4023`;
- safety intersection and empty-intersection control flow at
  `PrintObject.cpp:4024-4059`;
- density/mode narrow-wall filtering at `PrintObject.cpp:4061-4087`;
- minimum-width repair at `PrintObject.cpp:4089-4122`;
- collection reconstruction and external metadata grouping at
  `PrintObject.cpp:4124-4145`;
- the `EXTERNAL` target and source-order loop completion at
  `PrintObject.cpp:4146-4150`.

Supporting source boundaries are:

- `libslic3r.h:52,60-61,92-94` for `EPSILON`, coordinate scales, and truncating
  scaled-coordinate conversion;
- `Flow.hpp:61-69` for scaled flow width;
- `ClipperUtils.cpp:614-619` for path opening as shrink then expand;
- `SurfaceCollection.cpp:25-42` and `Surface.hpp:294-301` for stable grouping
  and merge properties;
- `Surface.hpp:260-285` for fresh and template-based surface construction.

The exact stop is after line 4150, before debug SVG output at lines 4152-4160.
The next source boundary begins after `discover_horizontal_shells`, with the
remaining `prepare_infill` caller operations. Candidate gathering and neighbor
mutation are one coherent boundary because the upstream loop is explicitly
serial: an early propagation may rebuild a later layer whose later source
collection must observe that mutation. A project-wide immutable pre-gather is
not source-compatible.

The Rust destination is a crate-private
`prepare_infill::horizontal_shell_propagation` successor after
`PreparedPostHorizontalShellPromotion`. O19-O26 are temporary
source-compatibility state around cited `libslic3r` behavior, not an Ares-owned
slicing pipeline.

## Exact traversal and control flow

After complete inherited alignment validation, visit each object, its current
single compatible region, each planned layer in bottom-to-top array order, and
each source kind in exactly this order:

1. `Top`;
2. `Bottom`;
3. `BottomBridge`.

For each populated record:

1. observe the already committed O25 extra-solid promotion;
2. read that record's typed resolved `RegionOptions`;
3. when `ensure_vertical_shell_thickness == EnsureAll`, skip all O26 work for
   the record before flow conversion, source-kind visits, geometry, or rebuild;
4. capture `print_z` and compute `bottom_z` as `print_z - height`, never from
   `slice_z`;
5. use `top_shell_layers` only for Top, and `bottom_shell_layers` for Bottom and
   BottomBridge; preserve the exact `== 0` gate and do not saturate or convert
   the configured count to `usize`;
6. gather exact matching surfaces from `slices` first, then the current working
   `fill_surfaces`; flatten each `ExPolygon` as contour followed by holes and
   preserve surface/path/point order;
7. skip the source kind when the resulting path vector is empty, without
   union, cleanup, area filtering, or mutation.

An aligned `None` source record represents an empty local layer-region and
therefore performs no source-kind work. It must not remove that array position
from neighbor distance or thickness calculations. When an active source scans
an aligned `None` neighbor, treat the neighbor as an empty fill collection at
that exact array index: still invoke the ordered safety-intersection operation
with the carried subject and empty clip, then apply the same density/mode
stop-kind or continue-neighbor branch to its empty result. Do not skip the
index, stop unconditionally, or bypass the operation/error order.

Top scans lower array indices. Bottom and BottomBridge scan higher array
indices. Stored layer IDs never determine traversal. A neighbor remains in the
loop while either its array distance is below the configured layer count or its
strict thickness predicate is true:

- Top: `current.print_z - neighbor.print_z < top_shell_thickness - 1e-4`;
- Bottom kinds:
  `neighbor.bottom_z - current.bottom_z < bottom_shell_thickness - 1e-4`.

Preserve the source's strict `<`, `EPSILON == 1e-4_f64`, count-or-thickness
short circuit, and direction. BottomBridge uses bottom settings.

For each neighbor, flatten exactly `Internal | InternalSolid` from its current
working `fill_surfaces` and compute the existing path safety intersection with
carried `solid`. If empty, density exactly zero or mode `None`/`CriticalOnly`
stops only the current source-kind scan, matching `goto EXTERNAL`; positive
infill with `Moderate` continues to the next neighbor. `EnsureAll` is already
unreachable.

## Numeric and geometry behavior

Use the retained `CoordinateScale` only after proving it equals
`CoordinateScale::from_printable_area` for the typed resolved printable area.
Convert a `Flow` width exactly as upstream `scaled_width()` does: retain the
`f32` width, divide by the selected `f64` scale factor, truncate to `i64`, cast
that scaled integer to `f32`, then perform the source `f32` multiplication.
Out-of-range scale or Clipper geometry returns exactly:

`SliceError::InvalidInput("horizontal-shell propagation geometry is outside the supported Clipper range")`.

Preserve the first failure in literal serial operation order.

The first narrow-wall factor is:

- density exactly zero: `1.0_f32`;
- otherwise `None`: `0.5_f32`;
- otherwise `CriticalOnly`: `0.2_f32`;
- otherwise `Moderate`: `0.0_f32`.

Its margin uses the **neighbor** external-perimeter scaled width. For a positive
factor, compute
`too_narrow = diff(new_internal_solid, opening(new_internal_solid, margin,
margin + 10.0_f32, Miter, 5.0))`. Opening returns flat paths and performs a
Miter-5 shrink by `margin`, then a Miter-5 expand by `margin + 10.0_f32`. If
`too_narrow` is nonempty, one plain difference becomes both
`new_internal_solid` and the carried `solid`.

The second filter margin uses the **source/current** record's solid-infill
scaled width multiplied by `1.0_f32` for `None` or `3.0_f32` otherwise. It uses
the same asymmetric Miter-5 opening. When nonempty, expand `too_narrow` by the
same margin as a flat NonZero positive offset using the upstream default Miter
join, miter limit `3.0`, and the existing shortest-edge configuration. Then
intersect without safety against reachable local kinds satisfying upstream
`is_internal() && !is_bridge()` (`Internal`, `InternalSolid`, and
`InternalVoid`), and append the result to `new_internal_solid`. Top, Bottom,
and BottomBridge are excluded. Do not update carried `solid` in this second
filter; the upstream assignment is commented out.

Reuse existing NonZero flat-path adapters, safety-intersection semantics,
offset topology/order, and `union_ex`; do not pre-union operands, substitute
EvenOdd or PolyTree output, sort, canonicalize, deduplicate, or add a safety
offset where the source has none. Add only the missing path-to-path asymmetric
opening adapter, implemented through the existing two offset stages and the
shared safety-offset constant.

## Collection reconstruction and metadata

Whenever a nonempty intersection reaches the rebuild block:

1. append original neighbor `InternalSolid` paths to `new_internal_solid`;
2. apply NonZero `union_ex`;
3. emit fresh `InternalSolid` surfaces with default metadata;
4. safety-difference all original `Internal` ExPolygons by the unioned solid
   paths and emit fresh `Internal` surfaces with default metadata;
5. append those Internal paths to the external clip paths;
6. retain only original Top, Bottom, and BottomBridge surfaces; this deliberately
   drops original InternalVoid when a rebuild occurs;
7. group retained external surfaces in stable original order by the first
   existing compatible group's first member;
8. define compatibility as exact kind, thickness, thickness_layers, and
   bridge_angle equality; deliberately exclude extra_perimeters;
9. difference each group against accumulated internal paths without safety and
   emit fragments in group order using the first member as a complete metadata
   template, including its extra_perimeters.

Output order is fresh InternalSolid, fresh Internal, then external groups in
first-seen group order. Add only a narrow `RegionSurface` operation that clones
all kind/metadata from a template while replacing geometry; do not introduce a
general one-use SurfaceCollection abstraction.

## Serial transaction, ownership, and cleanup

Before cloning or geometry, validate the complete O25 envelope: typed retained
scale, object and O18-O25 sidecar lengths, record/plan/input/prelude/lslice
lengths, `Some`/`None` alignment, source object and transform identity, planned
array index and layer ID, current layer/region, region ID, and the established
single-compatible-region constraint.

O26 must remain serial and transactional:

1. for any object containing a non-EnsureAll source record, create an aligned
   temporary working clone of each present record's `fill_surfaces`; original
   `slices` remain immutable and are read directly;
2. execute the complete layer/type/neighbor traversal against the working fill
   graph, so every later gather observes every earlier rebuild;
3. mark a neighbor dirty whenever the upstream rebuild block executes, even if
   the rebuilt value compares equal;
4. do not clone or move perimeters, thin fills, fill boundaries, predecessor
   trees, or O19-O24 sidecars;
5. on any error, discard the working graph and dispose the exact unmodified O25
   input, with zero commits;
6. after complete success, move the exact O25 graph and replace only dirty
   records' original `fill_surfaces`; drop clean working clones without touching
   their originals.

EnsureAll-only objects, zero-count/empty-source branches, and clean records
retain exact vector capacity/pointer and all inner allocations. Dirty records
receive fresh source-faithful collections. `slices` and every unrelated record
field remain exact. The successor adds no durable cache or branch sidecar and
owns the exact O25 predecessor, objects, caches, projections, trims,
regularizations, and filters. Disposal reconstructs O25 and delegates to its
iterative cleanup. Public slicing invokes O26 once after O25, disposes it, and
continues returning `ProjectSlicingIncomplete`.

## Reachable vocabulary and explicit deferrals

Current Ares reaches Top, Bottom, BottomBridge, Internal, InternalSolid, and
InternalVoid at this boundary. Upstream internal-bridge variants are absent from
the temporary O17-O26 representation and remain deferred; do not broaden the
enum speculatively.

Also deferred:

- trace logging and per-layer/caller cancellation, and any new cancellation API;
- debug SVG/filesystem output at `PrintObject.cpp:4152-4160`;
- `combine_infill`, later `prepare_infill` operations, external-surface
  processing, fill generation, toolpaths, seams, ordering, motion, G-code, and
  post-processing;
- multi-compatible-region behavior beyond the inherited explicit constraint;
- public API or persisted-format changes, new dependencies, fallback, Orca
  runtime/FFI, reference-G-code reads/replay, and fixture name/hash/layer-count/
  geometry-identity branches.

## Tests and acceptance

1. Direct asymmetric-opening tests freeze empty/disjoint/holed/repeated path
   behavior, exact two-stage deltas, Miter-5 configuration, topology/order, and
   first/second-stage coordinate failures.
2. RegionSurface tests prove template geometry replacement preserves kind,
   thickness, thickness_layers, bridge_angle, and extra_perimeters exactly.
3. Gate/gather tests prove O25 visibility, EnsureAll, Top→Bottom→BottomBridge,
   exact zero count, slices-before-fill, contour-before-holes, exact kind
   selection, and zero geometry on empty sources.
4. Window tests cover top/down, bottom/up, BottomBridge bottom options, variable
   heights, strict `1e-4`, count OR thickness, and nonconsecutive stored IDs.
   Trusted direct states with negative top and bottom counts plus positive
   thickness prove the pinned gate is exactly `== 0`, not `<= 0` or a
   saturating unsigned conversion.
5. A mandatory serial witness makes an early propagation rebuild a later layer
   and proves that layer's later gather observes the rebuild. Global pre-gather,
   independent per-source staging, reversed order, or parallel traversal must
   fail it.
6. Control tests distinguish density-zero/None/CriticalOnly stop-kind behavior
   from positive-density Moderate continue-neighbor behavior.
7. Numeric tests freeze factors, neighbor external versus current solid flow,
   truncating scaled-integer-to-`f32` order, normal/LargeBed scales, asymmetric
   `+10.0_f32`, Miter-5, first-filter carried-solid update, and second-filter
   non-update.
8. Repair tests include Internal/InternalSolid/InternalVoid, exclude external
   bridge kinds, and freeze flat path ordering. An acute-corner/configuration
   witness proves the repair growth uses Miter-3 with the existing shortest-edge
   setting rather than inheriting the adjacent opening's Miter-5.
9. Rebuild tests freeze existing-solid union, Internal safety difference,
   InternalVoid removal, output order, stable first-group behavior, holes and
   disconnected fragments, merge-key fields, excluded extra_perimeters, and
   first-template metadata.
10. Every inherited mismatch fails before O26 events/clones/geometry. Failure
    injection at each ordered geometry site, including out-of-range neighbor
    external-flow scaling, out-of-range current solid-flow scaling, both opening
    stages, and a late external group, proves their exact serial precedence, one
    stable error, zero original commits, exact rollback, and one O25 disposal.
11. Ownership freezes exact predecessor, outer/record/unrelated-field buffers,
    O19-O24 sidecar allocation/content, clean fill/geometry allocations, and
    dirty fresh collections. A geometry-equal rebuild is still dirty.
12. Shared constrained-stack tests retain both independent 10,000-node
    predecessor families with Unix/non-Windows 64 KiB and Windows 256 KiB
    baselines for success, late failure, and public-incomplete cleanup.
13. Lifecycle tests prove O26 exactly once after O25, zero O26 invocations for
    every earlier capability/O17-O25 failure, O26 error precedence, and terminal
    `ProjectSlicingIncomplete` on O26 success.
14. Two independent real KSR captures reassert O25 and freeze 460 aligned O26
    record visits, 460 EnsureAll skips, zero source-kind/geometry/rebuild/dirty
    commits, exact allocation and digest preservation, and one prepare/disposal.
    Tests never read the reference G-code.
15. Normal typed 3MF mutations prove EnsureAll after active O25 promotion,
    active Moderate horizontal propagation with nonzero rebuilds, ZIP
    order/compression/timestamp and non-slicing rename invariance, scale
    selection, and repeatability. Independent resolved model-part/archive
    witnesses vary the EnsureAll gate, top and bottom layer counts and thickness
    windows, sparse-density control flow, neighbor external-flow width, and
    current solid-infill-flow width. Direct retained-state witnesses assign
    distinct source and neighbor flows to prove O26 consumes each aligned
    `PerimeterInputRecord` flow rather than recomputing a global/first-record
    value. Synthetic tests cover branches the real archive does not naturally
    reach.
16. Existing optimized `sliceProject` executes EnsureAll and active O26 3MF
    archives without a trap or new export; success remains
    `ProjectSlicingIncomplete` and both Playwright repetitions agree. Existing
    `slice_stl` coverage remains a regression only because that binding does not
    enter the project-slicing pipeline.
17. Compiling mutation witnesses kill source reordering, missing gates/holes,
    ID-based traversal, `== 0` changed to `<= 0`/saturation, AND windows, wrong
    direction/EPSILON, pre-gathering, absent-neighbor skipping, stop/continue
    reversal, global/first-record option reads, wrong aligned flow provenance,
    factor/flow/cast/opening changes, repair growth changed from Miter-3,
    second-filter carried-solid updates,
    bridge repair inclusion, rebuild/group/metadata changes,
    original-before-success mutation, all-record commit, and public or cleanup
    bypass. Production is restored byte-exactly before final GREEN.
18. Focused O26 tests, explicit O21-O26 regressions, workspace Nextest, native
    all-target check, strict all-target/all-feature Clippy, four WASM checks,
    optimized browser builds/export audit, two Playwright runs, rustfmt, diff,
    dependency, staging, rollback, LOC, and forbidden-pattern audits all pass.
19. Every Rust file remains below 400 LOC and every new O26 shard is at most 300
    LOC. New code contains no `unsafe`, `include!`, `include_bytes!`, broad lint
    allowance, binary oracle, source-text/hash/line pinning test, fixture branch,
    Orca command/FFI, reference-G-code access, or fallback. Tests use ordinary
    `mod` and real files.
20. The complete spec and plan are approved before implementation. After
    implementation, an independent six-dimensional reviewer and separate
    OpenCode reviewer validate requirements completeness, logic correctness,
    boundary cases, code quality, test coverage, and actual execution. The
    parent sole writer fixes all blockers and repeats review until approval.
21. The final committed and pushed O26 SHA passes the exact repository Tier-1
    matrix on Windows, macOS, Ubuntu, format, and optimized browser-WASM with
    export audit and both Playwright runs. Pending or failing exact-SHA CI blocks
    shipping.

## Documentation and rollback

After evidence is frozen, update `docs/architecture/option-parity-v4.md`,
`docs/roadmap.md`, this spec, and the plan with exact counters/digests, test and
mutation totals, review evidence, and the next cited source boundary.

Mechanical rollback restores O25 as the public terminal consumer; removes only
the O26 module/state/wiring/tests/docs, the path-opening adapter and its narrow
geometry reexports, and the narrow RegionSurface template seam; and retains all
O25 options, geometry, sidecars,
dependencies, persisted formats, and public API unchanged.

## Frozen implementation evidence

Two independent real-KSR captures preserve O25 checksum
`58727684244877231975278290246623082466` and O25 record digest
`160750122870413723145549886803558415603`. The O26 EnsureAll capture preserves
surface digest `-107673730348313625723619859456104452971`, freezes event digest
`55157732452648897477979936233453742487`, and records exactly 460 visits, 460
skips, zero source/neighbor/geometry/rebuild visits, zero commits, and one
prepare/disposal per capture.

A resolved typed Moderate archive capture freezes surface digest
`55371787254720044626064449746884984931`, event digest
`71433667081695804905700384637078674080`, and 5,469 ordered geometry events.
Its raw event totals are `[460, 460, 0, 1380, 1010, 547, 143]` for fill
clones, record visits, EnsureAll skips, source-kind visits, neighbor visits,
rebuilds, and dirty commits. All 547 rebuilds follow nonempty intersections and
commit 143 distinct dirty records. Repack/order/compression/timestamp variation
is identical.

The final O26 filter passes 45 tests; supporting opening and surface-template
filters pass six and one tests. The complete workspace passes 5,908 tests with
2 skipped. Thirty-three compiling behavioral mutations are killed without compile
failures or survivors, followed by source restoration, formatting, all-target
native checks, and strict all-target/all-feature Clippy. The final suite includes
a controlled production serial-rebuild gather, all-site rollback fingerprints,
full active sidecar and clean-inner-geometry allocation snapshots, actual
aligned external/solid-flow overflow rollback, an actual geometry-equal
production rebuild, resolved archive/model-part count, thickness, density, flow
and scale variants, and non-slicing rename invariance.
Optimized native and browser-WASM builds, export audit, and two 11-test Playwright runs use the
existing `sliceProject` API. No dependency change is introduced. Final
independent six-dimensional approval is recorded in
`/tmp/task22o26-implementation-independent-approved-final.md`; the separate
default-model OpenCode approval is recorded in
`/tmp/task22o26-implementation-opencode-approved-final-review.txt`. Exact
pushed-SHA Tier-1 is intentionally not claimed until observed.
