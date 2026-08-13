# Task 22O.46 — sparse infill polylines for anchoring

## Status

Completed. The strict full-KSR fixed-MSVC O44/Clipper semantic replay completed
in clean Debug and Release builds, restored the pinned tree byte-exact, and
opened the implementation gate recorded below. The final focused, dependency,
workspace, formatting, warning-denying Clippy, wasm32, structural, and ignored
golden-progress gates pass at their documented boundary, and independent
source/specification and standards rereviews approve unconditionally. The
serial reversible mutation audit kills all 18 exercised wrong variants and
restores both production source files byte-exact.

## Goal and upstream boundary

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
public `Layer::generate_sparse_infill_polylines_for_anchoring` operation from
`OrcaSlicer/src/libslic3r/Layer.hpp:194-196` and
`OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1377-1504`.

The Rust destination is a dependency-first crate-private Layer operation under
`project_slice::prepare_infill::bridge_over_infill::sparse_anchoring`. It
borrows one retained lower-layer view and returns the final owned ordered
`Vec<Polyline>` produced for bridge anchoring. It consumes O45, which consumes
O44. It does not create a prepared-project successor or materialize the
transaction-local lower-layer map from `PrintObject.cpp:2725-2761`.

Keep the KSR-observable portion of `group_fills` private behind that operation:

- empty-template angle conversion at `Fill.cpp:52-59`;
- private pattern/bridge-angle group record context at `Fill.cpp:216-221,232-235`, the
  KSR-observable comparator decisions/equivalence shell at
  `Fill.cpp:275-281,284,307-308`, and the core
  group record at `Fill.cpp:336-342`;
- exact-corpus surface grouping at
  `Fill.cpp:829-835,855-858,861-862,864,866,881-884,891-898,943` and sparse
  generation projection at `Fill.cpp:867,925-926,934-936,979-989`;
- comparator-equivalent insertion, ordered materialization, and source-order
  geometry accumulation at `Fill.cpp:1012-1052`;
- mutual priority union/difference at `Fill.cpp:1054-1067`; and
- post-priority sparse filtering and CrossHatch generation at
  `Fill.cpp:1394-1397,1401-1406,1408-1441,1448-1449,1477-1482,1489,1492-1504`.

The crate-private seam is:

```rust
#[derive(Clone, Copy)]
pub(in crate::project_slice) struct SparseAnchoringLayer<'a> {
    pub(in crate::project_slice) planned: &'a PlannedLayer,
    pub(in crate::project_slice) fill_surfaces: &'a [RegionSurface],
    pub(in crate::project_slice) region_options: &'a RegionOptions,
    pub(in crate::project_slice) object_options: &'a ObjectOptions,
    pub(in crate::project_slice) nozzle_diameters: &'a OrcaFloats,
    pub(in crate::project_slice) scale: CoordinateScale,
}

pub(in crate::project_slice) fn generate_sparse_infill_polylines_for_anchoring(
    layer: SparseAnchoringLayer<'_>,
) -> Result<Vec<Polyline>, ClipperError>;
```

The view is the Rust equivalent of a borrowed source `Layer` under Ares's
already-enforced single-region graph. Do not pass prefiltered sparse surfaces,
projected CrossHatch scalars, grouped jobs, candidate payloads, or a map.
`SurfaceFillParams`, `SurfaceFill`, and the grouping operation remain private.

## Direct dependencies

The source-cited dependency boundary also includes:

- `Surface.hpp:9-33,35-114` for kinds, metadata, and surface predicates;
- `Layer.hpp:33-60,123-196` for Layer/LayerRegion ownership and the public
  method;
- `PrintRegion.cpp:7-22,25-30,37-38,50-53`, `Flow.hpp:13-115`, and
  `Flow.cpp:129-143,200-205` for the reached explicit-width, nominal
  non-first-layer `frInfill`
  construction and spacing;
- `FillBase.hpp:33-67,93-127,182-194` for fill parameters, connection gates,
  and the default zero overlap;
- `PrintConfig.hpp:87-98` for source pattern order and the KSR CrossHatch
  value;
- `Geometry.hpp:301-305` for degree-to-radian conversion;
- `ClipperUtils.hpp:36-38,340-378,431-455` and
  `ClipperUtils.cpp:264-300,360-410,437-506,737-811` for safety union,
  polygon-subject safety difference, and ExPolygon topology/order;
- `ExPolygon.hpp:300-306` for contour-before-holes flattening;
- `libslic3r.h:43-70,93-94` for scaling; and
- completed O44/O45 specifications and production seams.

Reuse Ares's existing `project_slice::perimeters::types::Flow`. Deepen its
existing resolver helper only as needed for nominal sparse `frInfill`; do not
add another Flow type or duplicate the rounded-extrusion arithmetic.

Re-export existing `union_safety_offset_ex` from the geometry root. Add only
the missing exact sibling:

```rust
pub(crate) fn difference_polygons_ex_with_safety_offset(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, ClipperError>;
```

It expands each clip polygon by the fixed raw safety offset and preserves
Polygon-subject/Polygon-clip add and execution order. The existing
ExPolygon-subject overload is not a substitute.

## Trusted contract

This dependency is intentionally narrower than generic `group_fills`:

- production traversal supplies the 18 retained KSR lower-layer views, each
  source-valid, post-O42, and single-region; focused Layer-seam tests may
  construct source-shaped views preserving the same four-kind/options/
  projection invariants (or deliberately cross a checked geometry range
  boundary), but do not authorize additional kinds or options;
- planned ID, height, and accumulated `print_z` are aligned with the borrowed
  surfaces and effective options;
- selected KSR lower layers are all non-first layers;
- retained kinds are `Top`, `BottomBridge`, `Internal`, and `InternalSolid`;
  the 18 selected lower layers contain no `Bottom` or `InternalVoid`;
- sparse pattern is CrossHatch, sparse density and all required flow values are
  finite and positive, both rotation templates are empty, model alignment is
  disabled, multiline is one, and anchor maximum is at least 0.05 mm;
- top density is positive, top pattern is MonotonicLine, and internal-solid
  pattern is Monotonic;
- all retained surfaces have `surface.thickness == -1`, `thickness_layers == 1`, and the same
  predecessor-resolved effective extruder one and nozzle 0.4 mm; the archive's
  raw zero selector fallback is already complete before this seam; normalized
  `BottomBridge` angles are nonnegative and every non-bridge angle is `-1`;
- topology is already normalized by prior source-cited stages; and
- typed option resolution already validated finite option ranges and the
  single-region/nozzle graph.

Keep these as private invariants/debug assertions where useful. Do not add a
new validation layer or a generic unsupported-pattern error surface.
Existing Flow helpers expose `SliceError` because they also serve an earlier
option-validation boundary. O46 receives their already-validated inputs: reuse
the arithmetic core and treat failure as a trusted invariant violation rather
than widening this operation beyond `ClipperError` or mapping it to empty
output.

## Required behavior

1. Borrow all source state. Clone only the geometry needed for ephemeral
   groups. Never `mem::take`, retag, reorder, or mutate O42/O43 surfaces or
   options.

2. Traverse the single region's stored surfaces in source order. The trusted
   contract already supplies positive sparse and top densities, so do not add
   defensive density skips. Do not prefilter to `Internal`.

3. Project each of the four retained KSR kinds into the only group fields that
   can affect this exact Layer result: representative kind, effective source
   pattern, f32 bridge angle, and source geometry. `Bottom`, `InternalVoid`, and
   every other kind are trusted-unreachable, not silent skip/fallback arms.

   The exact projection is `BottomBridge -> Monotonic`,
   `InternalSolid -> Monotonic`, `Top -> MonotonicLine`, and
   `Internal -> CrossHatch`. Preserve the stored nonnegative bridge angle for
   `BottomBridge` and `-1_f32` for non-bridges. Key 40 contains all four kinds
   and is the public discriminator. Do not materialize actual/non-sparse Flow,
   extruder, role, speed, or other private source fields: the corpus proves no
   later comparator clause distinguishes any projected pair before the returned
   sparse geometry.

4. Empty rotation templates compute degrees-to-radians in f64 and cast the
   result to f32. Do not add legacy odd-layer alternation. Align-to-model true
   and the template metalanguage are deferred.

5. Build only the nominal sparse `frInfill` Flow consumed by CrossHatch, from
   the object's nominal layer height with `first_layer=false`. Use the fixture's
   explicit sparse line width and already-resolved effective nozzle. Current
   planned height, surface thickness, solid/bridge Flow, first-layer width, and
   width/nozzle fallback variants must not enter this exact-corpus slice.

6. Preserve source f32/f64 cast order. Region density becomes f32 percent.
   `FillParams::density` is `f32(0.01_f64 * density_percent)`. Raw anchor and
   maximum values are cast to f32 before percent multiplication by the nominal
   promoted spacing; clamp anchor to maximum only after both conversions.

7. The observable comparator projection orders f32 bridge angle decreasing,
   then explicit source pattern rank: Monotonic `0`, MonotonicLine `1`, and
   CrossHatch `20`. Compare bridge angles with source `<`/`>` and do not cast a
   Rust enum ordinal. No host sort or hash grouping is allowed; ordered
   insertion must directly implement these two source decisions.

   Every source comparator clause after pattern is constant or already
   separated for the 18 retained KSR views, so none distinguishes a projected
   pair. It is deferred together with the
   unused earlier extruder clause; exact sparse angle, density, anchors,
   multiline one, and overlap zero remain O45 generation inputs rather than
   group-key fields. Do not invent private tests or wider synthetic inputs to
   expose deferred comparator state.

8. Implement set-equivalent ordered insertion and coalescing without a host
   sort over equivalent records. Equal bridge-angle/pattern keys share one group;
   their ExPolygons append in original surface order.

9. Materialize groups in comparator order. The group's representative surface
   is the first source surface. Flatten every ExPolygon as contour then holes,
   preserving all sibling and vertex order.

10. Process priority exactly once in group order. A multi-ExPolygon first
    nonempty group uses `union_safety_offset_ex`. Every later nonempty group uses
    `difference_polygons_ex_with_safety_offset` against all preceding raw
    polygons. Append the current group's original flattened subjects—not its
    clipped or safety-expanded output—to the preceding accumulator.

11. Leave a lone ExPolygon with no predecessor byte-equivalent and avoid an
    unnecessary union. If a later group exists, still append that lone group's
    raw contour/hole paths to the preceding accumulator before processing the
    later group. An empty intermediate or Clipper result remains empty success.
    Never canonicalize or sort final geometry.

12. Only after priority processing, select groups whose representative type is
    exact `Internal`. CrossHatch is absent from Orca's explicit switch at
    `Fill.cpp:1408-1438` but falls through to generation; Rust must use an
    explicit `CrossHatch => fill_surface(...)` arm.

13. For each selected grouped ExPolygon, reset spacing to the group's nominal
    sparse spacing, use exact accumulated lower-layer `print_z`, and call O45
    with density fraction, angle, anchors, multiline one, overlap zero,
    `dont_sort=false`, and explicit scale. Append O45 results in group then
    ExPolygon order.

14. Return only the owned ordered polylines. Add no bounding-box, resolution,
    loop-clipping, ZAA, adaptive, support, Lightning, or generic filler state;
    CrossHatch does not read it in this slice.

15. Return the first reachable `ClipperError`. Grouping and CrossHatch range
    errors are not `InfillFailedException`; do not catch, continue, or return a
    completed prefix. Drop all local accumulated paths and leave borrowed input
    unchanged.

## KSR path and oracle

O43 current keys become lower generation keys by subtracting one:

```text
[14,29,30,31,40,44,59,64,69,74,81,84,89,104,115,124,135,254]
```

All 18 sparse groups share:

```text
pattern CrossHatch=20       spacing f32 0x3ed06cbe
spacing f64 0x3fda0d97c0000000
density percent 0x41700000 density fraction 0x3e19999a
angle 0x3f490fdb           anchor 0x3fd06cbe
anchor max 0x41a00000      bridge angle 0xbf800000
overlap 0                  multiline 1
dont_sort false
```

The retained lower-layer input contains 210 surfaces: 103 `Internal`, 95
`InternalSolid`, 10 `Top`, and two `BottomBridge`. Before deferred postpasses,
the source comparator materializes exactly 43 groups: 18 `Internal`, 17
`InternalSolid`, six `Top`, and two `BottomBridge`. There are no Bottom or Void
records and no within-kind split by any later comparator clause.

The two retained bridges both have positive angles and therefore precede every
`-1` non-bridge. Lower 40's retained Ares angle is π-line-equivalent rather
than bit-identical to Orca's angle; O46 preserves the source comparator outcome
and final returned geometry, not a private bridge-angle diagnostic bit pattern.

The Linux Debug/Release diagnostic is 189 / 5,950 and
`08c1b067d7a19a2b24920ad6f4db0e2d0dd9ba31aa6e0941a05b2fc99e42fbe7`;
it is not a portable compatibility target. The earlier 189 / 5,947 C++ result
`4aebe72d382181c950c0cfd770c763af20cbd28a3c4db822203800ece9a7fb89`
is rejected: that experiment pinned O44's sorts but left active Clipper sorts
on Linux `std::sort`, creating an invalid hybrid.

The independent Ares/O45 replay produced 186 / 5,942 and
`bcb9b45b236023ab90b8bb5cc693eb89d694446b7093da10e0636d0fdc2d6b01`,
but retained Linux-captured post-priority inputs. The completed strict global
replay applies fixed-MSVC ordering to O44 and all active Clipper sorts, rebuilds
all 209 affected objects per mode, and confirms 186 / 5,941 with normative
ordered SHA-256
`917adc6ea02ad7cd7af79e45d90db6f4c1497bf5c8716d7f2f49b7de4b2070ef`.
Its Debug/Release per-key table is byte-identical and has SHA-256
`bf531afcde1d97a3dce2fb33e1d54c90b85ce42c31d1d0f632c7f52e606e9cb8`:

| key | paths | points | serialized SHA-256 |
|---:|---:|---:|---|
| 14 | 17 | 746 | `a1e692258f4b5002e0e4469d1d256d6c0b41b81b79a4b06a3b4c4446daa118b0` |
| 29 | 19 | 703 | `cc2d59528a2141aa393f3447af74e210b0d8f8dabd8c7d902da1c1d1686d0788` |
| 30 | 16 | 719 | `513f43efe8275385845432b21a09bb8d08c6c91ba2f6ebbaf09612b94e6c4f32` |
| 31 | 20 | 415 | `aae4fab8051f757186afda9d528a931820a72b08b7c99179fa7dbfc5e5723a0e` |
| 40 | 18 | 914 | `faffc973840b61e3789bdc2c13deed124416aec38096f57172446f0fabdebf04` |
| 44 | 25 | 681 | `0906c57cf80e33c96a7100ae60e896d919e51a66d127e778f27af58c9679046d` |
| 59 | 10 | 202 | `b19d2529ac0c9441600c8e231c60b961381f24c3826ddda2b3d54f66996930ba` |
| 64 | 10 | 180 | `c3fec241bb8843f311ebf98df4418e8c0e88ad26feeb437ac7f2a87441216f79` |
| 69 | 12 | 325 | `58acb0df99eb27a51cef7c65934f12c61025a96e8093c2720379c0684cee31c6` |
| 74 | 5 | 107 | `60b8e116db2d367ae33374bfcbb65196e2bbd8cce21af1437ac9623bb2eedcac` |
| 81 | 7 | 201 | `745d2e6e22a37ef9b58e45096a6132acbbdf22c85a531491c7573fe222c63261` |
| 84 | 7 | 197 | `27990e0972d10340888d6e05c7d2eaa67f2889266e7f3e8f21d394822c7d5abd` |
| 89 | 5 | 108 | `ccdd457b000c43ac2a169746e60f5ed1289fd04f83591431168ad00b0fe65154` |
| 104 | 3 | 77 | `7822089cf4c9a03aff0ca5242972c0dee6f9052da0972fb894840b4d56750e20` |
| 115 | 4 | 61 | `b1841222c6e82ffd60696129aea39f6351cfac1a4cd8b50c4736de8ec0d86f9a` |
| 124 | 3 | 148 | `aa32ca19d6ed54f2fa7ce93f9b23dfd6a750bff6d4f3b4891069a84391af885c` |
| 135 | 3 | 116 | `8de9d4fc24776ea390c28be629cf6cefc347762e7587033d64b07e1be87d9903` |
| 254 | 2 | 41 | `85a82391f1666a44459d0db790821a96453aa3dfb7360f5d30a4617204187409` |

Preceding non-sparse groups alter sparse geometry at 17 of 18 keys. The
historical Linux-captured post-priority sparse digest is
`d9a732b027dde4373bcb8400b90b937d2cc5404ecb6234af2fed5b2606b94c84`;
the corresponding captured invalid filter-first/no-prior diagnostic is
`20a7678298f136712ca9e6d44aa3377225a128098c5500a0a10f6ec643611466`.
Neither is a portable fixed-Clipper acceptance literal; final returned Layer
geometry supplies the normative oracle.

Layer 115 is the smallest literal priority discriminator. Correct strict
serialized output is four paths / 61 points with SHA-256
`b1841222c6e82ffd60696129aea39f6351cfac1a4cd8b50c4736de8ec0d86f9a`;
filter-first gives three paths / 60 points. Layer 254 is the two-path / 41-point
no-prior control with strict serialized SHA-256
`85a82391f1666a44459d0db790821a96453aa3dfb7360f5d30a4617204187409`;
its fixed-MSVC ordering differs from the Linux file even though its counts do
not.

## Error and ownership semantics

`Fill.cpp:1496-1499` catches only `InfillFailedException`, whose selected-tree
producer is Rectilinear consistency validation. CrossHatch and O44 do not
produce it. O46 therefore does not add an `InfillFailed` variant or a generic
catch. The recoverable branch remains deferred until a supported filler can
produce it.

Coordinate range errors from Clipper input/offset work propagate. Structural
open-path errors are impossible through source-shaped helpers. Clipper
execution returning false becomes empty success, matching the source wrappers.

A first or later grouping/O45 error must return `Err`, publish no prefix, and
preserve every borrowed surface and option bit. Use natural coordinates to
test this; add no callback, trait, global failure mode, or cfg(test) production
hook.

## Deferred behavior and scaffolds

Explicitly defer:

- nonempty rotation templates and `Fill.cpp:60-213` metalanguage;
- align-to-model true;
- every comparator decision except descending bridge angle and source pattern
  rank, namely `Fill.cpp:283,285-305`; all are constant or already separated
  before they could affect these exact KSR Layer results;
- extruder/role, `Bottom`, density-skip, lattice/lock/symmetry/Gyroid,
  non-sparse angle, fixed angle, actual/non-sparse Flow, bridge-flow, speed, and
  non-sparse spacing/anchor projection at
  `Fill.cpp:859-860,863,865,868-879,885-890,899-902,904-924,928-933,937-942,944-978`;
- zero/percent sparse-width and raw selector/nozzle fallback variants adjacent
  to `PrintRegion.cpp:7-53`; the retained KSR view uses explicit 0.45 mm width
  and predecessor-resolved extruder/nozzle state;
- non-Monotonic/MonotonicLine top-pattern bridge fallback to Rectilinear;
- deferred `has_internal_voids`/object-config setup and LockedZag
  lambdas/sidecars at `Fill.cpp:836-853,992-1010`;
- multi-region `region_id_group` and no-overlap sidecars/aggregation at
  `Fill.cpp:343-345` and their consumers;
- InternalVoid repair at `1069-1150`;
- narrow-solid helpers/splitting at `349-827,1152-1186`;
- generic `group_fills` reuse by `Layer::make_fills` and extrusion entities;
- all non-CrossHatch sparse generators and recoverable
  `InfillFailedException` continuation;
- `Bottom` and every surface kind outside the exact four-kind KSR
  corpus;
- arbitrary no-Internal/empty-layer views outside the 18 selected KSR lower
  layers;
- adaptive/support-cubic octrees and Lightning generators;
- density-over-80 linking, concentric loop clipping, resolution, ZAA, lattice,
  gyroid, and filler bounding-box behavior unused by CrossHatch;
- unused caller state/FillParams projection at
  `Fill.cpp:1398-1399,1442-1447,1450-1476,1483-1488,1490`;
- multiline greater than one and anchor maximum below 0.05, already deferred
  by O45;
- `PrintObject.cpp:2725-2761` map ownership and its `3203` consumer;
- clustering, anchor intersection, bridge direction/commit, extrusion,
  motion, G-code, CLI, filesystem, TBB, logging, timing, and debug SVG.

KSR enables narrow-solid detection, but that postpass only changes non-sparse
groups after priority. Instrumentation proves KSR sparse bytes equal the
snapshot immediately after `1054-1067`. KSR also has zero InternalVoid
surfaces. These branches are output-dead for this public Layer result, not
claimed complete.

Leave legacy `infills.rs`, `InfillOptions`, its collapsed anchor accessor,
rotation helpers, and rectangle-only narrow-solid scaffold untouched and
uncalled. No fallback is permitted.

O46 remains unwired. Public slicing still consumes/disposes O43 and returns
`ProjectSlicingIncomplete`; O46 changes no public option, API, G-code byte, or
workspace crate.

## Acceptance

Follow vertical TDD through only the Layer operation, except for the new shared
geometry overload's own direct discriminator:

- exact Polygon/Polygon safety-difference order, topology, fixed raw offset,
  empty success, and coordinate error;
- a pure single-Internal CrossHatch layer that freezes nominal Flow, angle,
  density, anchor, accumulated Z, O45 output, and source nonmutation;
- same bridge-angle/pattern-key multi-surface coalescing before O45;
- bridge/top/solid-before-sparse priority, including raw-prior accumulation and
  a hole-bearing multi-ExPolygon union;
- exact key-40 returned output as the all-four-kind discriminator for
  decreasing f32 bridge angle and explicit Monotonic/MonotonicLine/CrossHatch
  pattern rank, plus a static ban on host sort and derived enum ordinals;
- nominal object-height/non-first sparse Flow and exact explicit-width cast
  order, proven only through returned Layer geometry;
- Normal/LargeBed returned-output discriminators plus a reversible wrong-scale
  mutation proving exact `CoordinateScale` forwarding;
- a returned-output discriminator plus reversible mutation proving O46 forwards
  `dont_sort=false` into O45;
- exact literal layer-115 priority output and layer-254 control;
- full real-KSR 18-key fixed-MSVC counts, per-key ordered digest, totals,
  repeatability, and complete input snapshot nonmutation after the two current
  replays are reconciled;
- natural first and later grouping/O45 range errors with no prefix; and
- the unchanged public lifecycle terminal at O43.

Unit/oracle/error tests use literal in-process geometry and never read source,
helper files, environment state, golden G-code, or the filesystem. The separate
real-KSR integration test may use the committed archive through `KsrArchive`.

Reversible mutations must kill filtering before grouping, skipped safety
union/difference, clipped-prior accumulation, ascending bridge angle, omitted
publicly observable bridge-angle/pattern decisions, filling same-key surfaces
separately, current-layer-height spacing substitution, altered
angle/density/anchor casts, 15% overlap leakage, collapsed anchor maximum,
layer-number Z recomputation, wrong scale, `dont_sort=true`, dropped CrossHatch
dispatch, output reversal or canonicalization, and catch/continue of
`ClipperError`. Mutations of deferred
constant or correlated comparator decisions, `total_cmp`, and host-sort
permutations are outside mutation acceptance and enforced by source/static
review instead; do not invent wider inputs or a private test seam to expose
them.

The full-corpus audit covers both O44 active sort sites across all 103 O45
calls. Endpoint projection has 1,507 records and zero comparator-equivalent
pairs. Arc ordering has 1,439 records and 2,700 equivalent pairs across 30
calls in 82 equality classes. Therefore literal full-KSR expectations must use
the repository's audited fixed MSVC STL 14.44 control flow and reconciled
semantic output; Linux Debug/Release agreement is diagnostic only.

Every Rust source file must remain below 400 physical lines. Final verification
requires focused and dependency Nextest bands, workspace Nextest, rustfmt,
workspace all-target/all-feature warning-denying Clippy, core/browser wasm32
checks, diff/whitespace/LOC/include/fixture-read audits, pinned-Orca restoration,
the unchanged ignored golden progress probe, and unconditional independent
source/specification and standards reviews.

## Oracle record

The historical public-process harness
`/tmp/task22o46-ksr-oracle.cpp` (`7c0d8ce8...`) only invokes `process()`. Its old
script (`f6f3630a...`) cannot reproduce the 90 diagnostic structural files,
which came from transient, previously unrecorded instrumentation. Their
`516c34a7...` manifest is non-normative.

The replacement oracle is complete. The clean-tree
apply/build/run/verify/restore script is
`/tmp/task22o46-global-msvc-full-rebuild-verify.sh`, SHA-256
`7337a05c7c92ad9e43e579f3c7fc8bdec3317ead29d0920298c0c54c498ffca1`;
its patch is `/tmp/task22o46-global-msvc-oracle.patch`, SHA-256
`089783ddd831ad0ef51f2a91c2c1c1c51ce6bf68c7aab6a6bda1e110aad3634d`.
The proof directory `/tmp/task22o46-global-msvc-full-proof.zzdoO5` records 209
fresh objects per mode, identical 103-member fill/endpoint/arc ID sets, 1,507
endpoint records and zero ties, 1,439 arc records and 2,700 raw tie pairs in 30
calls and 82 complete exact-bit classes, byte-identical Debug/Release per-key
results, 186 paths / 5,941 points, and clean restoration to pinned commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The proof `totals.tsv`,
`provenance.tsv`, expected per-layer table, and evidence manifest are the
canonical record.

The historical captured-input priority replay source is
`/tmp/task22o46-priority-oracle.cpp`, SHA-256
`b8bc6b470e34d1030fe810a62ca45a7b28ec03f4e21dce857e844a01f1d858b5`;
its script is `/tmp/task22o46-priority-build.sh`, SHA-256
`e60b3237c32732cba7d144ba224e506087dd6b5ebab0b6750744346dab52ba02`,
and both Debug and Release serialize to
`ac462f6d558b9763d66908323f06cefca9051dd7d5b2cb6fc554d6215ad6fcad`.

The disposable Orca tree is restored and clean. Restored `Fill.cpp` and
`PrintObject.cpp` SHA-256 values are respectively
`6b46cccc74749bb352497ea90c176381c9adcf5cece7fd06333c6b83c56ee59d`
and `7efa9c467c6f32a46008a167d525458d582859f76706a5be6412a84d7c6ab589`.
