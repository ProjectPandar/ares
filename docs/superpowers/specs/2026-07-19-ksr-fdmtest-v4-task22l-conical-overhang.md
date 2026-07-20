# Task 22L: Post-Region Conical Overhang

## Status and objective

This specification is a draft. Production or tracked test implementation may
begin only after the exact specification and implementation-plan bytes receive
independent fixed-source/specification and current-Ares/plan approval.

Task 22L is the next bounded source rewrite in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`7f71ed8068102772d54346ac08184ef6b0bcd79b` produces the complete Task 22K
post-top-trim stream. Task 22L ports the non-cancellation success semantics of
OrcaSlicer's immediately adjacent `PrintObject::apply_conical_overhang()`
stage. It consumes Task 22K post-region state and effective Options resolved
only from the supplied 3MF.

This is the complete geometry unit, not a disabled-only gate, rectangle-only
approximation, or fallback. Enabled inputs support arbitrary ExPolygons,
holes, multiple regions, both coordinate scales, and top-to-bottom propagation.
The implementation remains platform-neutral and deterministic on WASM,
Windows, macOS, and Linux.

Task 22L stops before material or painted segmentation, compensation,
`make_slices`, surface classification, perimeters, fill, supports, extrusion
paths, G-code assembly, metadata, or post-processing. The public project API
executes Task 22L and continues to return
`SliceError::ProjectSlicingIncomplete`.

## Fixed identities and source blobs

The fixed Ares baseline is commit
`7f71ed8068102772d54346ac08184ef6b0bcd79b`, tree
`4e3a7445d340bd1dc22bdb184fbca6f2bad17521`. Exact-SHA Tier-1 run
`29704298779` passed format, Ubuntu/Linux, Windows, macOS, and WASM/browser.

All upstream citations refer only to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored upstream checkout is
read-only evidence; tracked tests never inspect it.

Fixed source blobs are:

- `src/libslic3r/PrintObjectSlice.cpp`,
  `07eb885eda83a495001467c22c0452dfc36e55c2`;
- `src/libslic3r/Layer.cpp`,
  `5bdc156d0172ec19894b630cc70d73b5aef8f82d`;
- `src/libslic3r/Layer.hpp`,
  `cb2e6c7c1a166a028ac8fceffaf9f42f3c2426b0`;
- `src/libslic3r/Polygon.cpp`,
  `32b4d062f1b8f898866a0e0e55672dcd5f54ac89`;
- `src/libslic3r/Polygon.hpp`,
  `7d996055e5d9403f871071ef82baa140c03492b5`;
- `src/libslic3r/Point.hpp`,
  `039f361eaa18db9c6e7d2c35d1c61af78bcad51b`;
- `src/libslic3r/ExPolygon.hpp`,
  `ce7ebe892f64b3d4e2e9fb0c85bd77b99e889d54`;
- `src/libslic3r/SurfaceCollection.hpp`,
  `1895516aa2eb1fa30be3cf63bb211f7db420f3af`;
- `src/libslic3r/Surface.hpp`,
  `b63c283251d63154a3a7071694c87b637ec7dff7`;
- `src/libslic3r/ClipperUtils.cpp`,
  `2f97e08f536e93c5fd27b4614980072285d2ce22`;
- `src/libslic3r/ClipperUtils.hpp`,
  `9c2fa239263c0cb097a4b4c3db823821615bd7c7`;
- `src/libslic3r/PrintConfig.cpp`,
  `982953afa50af0217a4d64639116ff4a2e596e90`;
- `src/libslic3r/PrintConfig.hpp`,
  `0a7b7ba36f87c3d4517daf96d7d8825812e66358`;
- `src/libslic3r/libslic3r.h`,
  `f4291d36df8175c700fa9374c5b5c07e6880e706`;
- `deps_src/clipper/clipper.hpp`,
  `06637effce040fa7d87c368437cb32398f19ee92`;
- `deps_src/clipper/clipper.cpp`,
  `1f16446ac8da1f0b9c802d8a9dee33f766919f6b`.

## Exact upstream rewrite boundary

The owning boundary is:

- `PrintObjectSlice.cpp:1194-1203` for the released Task 22K predecessor;
- `PrintObjectSlice.cpp:1204-1206` for the caller sequence between Task 22K
  and conical overhang;
- `PrintObjectSlice.cpp:1394-1509` for
  `PrintObject::apply_conical_overhang()`;
- `Layer.cpp:21-29` for layer emptiness;
- `Layer.cpp:117-136` for `Layer::merged(float)`;
- `Polygon.cpp:52-68` and the owning declaration in `Polygon.hpp` for the
  signed cross-product area accumulation used by the strict hole threshold;
- `Point.hpp:185-206` for the signed integer point representation consumed by
  polygon and Clipper paths;
- `SurfaceCollection.hpp:49-81` and `Surface.hpp:35-72` for surface-vector
  emptiness and `set(..., stInternal)` rebuilding;
- `ExPolygon.hpp:21-36` for constructing a hole path as the contour of a
  temporary ExPolygon;
- `ClipperUtils.cpp:264-301,430-575,634-667,737-836` and
  `ClipperUtils.hpp:17-27,73-246` for the safety-offset enum, path providers,
  raw offset, ExPolygon offset,
  two-pass PolyTree booleans, union/XOR overload roles, the 10-coordinate
  safety offset, Miter join, and miter limit 3;
- `libslic3r.h:52,60-70,93-96` for `EPSILON`, runtime scaling, and
  `SCALED_EPSILON`;
- `PrintConfig.cpp:4974-5001` and the owning fields in `PrintConfig.hpp` for
  the three conical-overhang Options.

The caller cancellation check at line 1204 and one cancellation check at line
1421 before each layer pair are classified but deferred. Ares has no public
cancellation or AbortSignal contract. Task 22L must not add a fake no-op
callback, test-only cancellation seam, or native-only control plane. This
deferral does not change an uncancelled output.

## Option ownership and validation

Task 22L consumes existing typed effective Options; it does not add a dynamic
map or a second parser:

- object `make_overhang_printable_angle`: finite float in `[0, 90]`, default
  `55`;
- object `make_overhang_printable_hole_size`: finite float at least `0`,
  default `0`, measured in square millimeters;
- object `layer_height`: finite positive nominal object layer height, already
  validated by Task 22A planning;
- region `make_overhang_printable`: bool, default `false`;
- region `bottom_shell_layers`, `top_shell_layers`,
  `sparse_infill_density`, and `wall_loops`: the four existing fields used by
  `Layer::merged` eligibility.

After effective configuration resolution and the released Task 22K
predecessor, the Task 22L orchestration boundary validates every resolved
object configuration before it mutates any `PostRegionPrintObject`.
Validation follows resolved-object vector order and checks angle before hole
size. An invalid angle returns exactly
`SliceError::InvalidInput("invalid Orca option make_overhang_printable_angle")`;
an invalid hole size returns exactly
`SliceError::InvalidInput("invalid Orca option make_overhang_printable_hole_size")`.
This validation runs even when the eventual object has zero retained layers,
zero regions, only disabled regions, or angle exactly `90.0`. It defines
precedence among Task 22L checks only; loading, effective resolution, Task 22A
planning, or Tasks 22B-K may already have returned an error. Every resolved
configuration is validated successfully before the first Task 22L object
mutation, so a later invalid object leaves every earlier post-K object
unchanged.

Task 22L does not silently clamp, replace, or fall back. The pure stage accepts
only validated records. For each object it returns first for an empty retained
layer vector, then for angle exactly `90.0`, and only then derives scaled
geometry. Raw validation therefore precedes those gates at orchestration, while
the pure geometry stage retains the fixed upstream gate order.

Object and region overrides continue to follow the released 3MF effective
configuration rules. Production must not read raw metadata after resolution,
use a global `PerimeterOptions` switch, infer an Option from geometry, or
special-case a fixture.

## Exact arithmetic and coordinate scales

The stage uses the runtime `CoordinateScale` selected before Task 22A:

- normal scale factor: `0.000001` millimeters per integer coordinate;
- large-bed scale factor: `0.00001` millimeters per integer coordinate.

The following arithmetic and rounding boundaries are non-negotiable:

1. `epsilon_scaled = f32(0.0001 / scale_factor)`, exactly `100.0` for normal
   and `10.0` for large-bed input.
2. `angle_radians = angle * PI / 180.0` and `tan_angle = tan(angle_radians)`
   are evaluated in `f64`.
3. `distance_scaled = -f32(tan_angle * layer_height / scale_factor)`.
4. Distance is not first quantized to `i64`; doing so would lose the fixed
   upstream fractional scaled delta.
5. `hole_area_scaled = f32(hole_size / scale_factor / scale_factor)` and the
   resulting `f32` value is promoted for comparison with signed polygon area.
6. The per-layer actual `PlannedLayer.height` is never used for the distance.
7. Offsets use Miter joins and miter limit `3.0`.
8. Cross-region safety subtraction expands each clip path separately by the
   fixed `10.0` integer-coordinate offset before the NonZero difference. This
   value is not rescaled for large beds.

Both exact f32 conversions for distance and hole area must be finite before
pair iteration. A nonfinite derived scalar or any subsequent
`ClipperError::CoordinateOutOfRange` maps exactly once to
`SliceError::InvalidInput("project conical overhang geometry is nonfinite or outside the supported Clipper range")`.
No arbitrary raw upper bound is introduced. Empty layer vectors and angle 90
return before derivation; one-layer objects derive before discovering that the
pair iterator is empty. Upper-empty, zero-region, and all-disabled pair gates
also occur after derivation. Geometry errors never become identity output.

## Required stage semantics

For each post-Task-22K print object:

1. The pure stage receives a configuration that passed the orchestration-wide
   raw validation transaction.
2. Empty layer vectors return unchanged without deriving scaled geometry.
3. Valid angle exactly equal to `90.0` returns unchanged without deriving
   scaled geometry.
4. Otherwise, distance and hole-area f32 values are derived and checked even
   for a single layer or zero regions.
5. Adjacent layer pairs are visited from the second-highest lower layer down to
   layer zero.
6. The already modified lower layer from one iteration becomes the upper layer
   of the next iteration. This top-to-bottom cascade is observable.
7. An upper layer is empty only when every region's upper-layer surface vector
   has zero elements. ExPolygon area is not inspected.
8. A layer pair is skipped when the upper layer is empty.
9. A layer pair is also skipped when every current lower region either has an
   empty surface vector or has `make_overhang_printable == false`.
10. The gate is cross-region. It must not invent same-region pair matching.
11. The top layer is never modified as a lower layer.

Objects are processed independently. The operation mutates only retained
region surface collections. Planned layers, object and region IDs, effective
Options, object ordering, and complete occurrence-keyed volume sidecars remain
unchanged.

## `Layer::merged(SCALED_EPSILON)` parity

For one layer, a region participates in the merged footprint only when at least
one of these is true:

- `bottom_shell_layers > 0`;
- `top_shell_layers > 0`;
- `sparse_infill_density > 0`;
- `wall_loops > 0`.

For every participating region, all surface ExPolygons are offset outward by
`epsilon_scaled` using the fixed ExPolygon offset algorithm. Positive offsets
within that region use the upstream paths-union rule. The resulting paths from
all participating regions are appended and passed through the fixed two-pass
Union-to-Paths then Union-to-PolyTree NonZero sequence into an ordered
ExPolygon vector. The caller then performs the fixed additional two-pass
`union_ex` normalization on both upper and current footprints.

`make_overhang_printable` is not a merge-eligibility field. A layer can be
nonempty by surface cardinality while all of its surfaces are excluded from
the merged footprint by the four-field filter.

## Hole protection parity

Hole protection executes only when `hole_area_scaled > 0.0`. A zero value,
including negative zero, skips the branch and allows projection to close
holes.

For each hole of each current merged ExPolygon:

1. use the absolute signed polygon area;
2. protect only when the area is strictly less than the scaled threshold;
3. copy the canonical clockwise hole unchanged as the contour of a temporary
   one-contour ExPolygon; do not reverse or canonicalize it, and preserve its
   negative signed area before intersecting it with the complete upper merged
   footprint;
4. require a nonempty intersection;
5. require XOR of that intersection and the complete hole to be empty;
6. only then subtract the hole from the upper merged footprint before the
   negative conical offset.

Equal-area, larger, partially covered, and uncovered holes are not protected.
Multiple and non-rectangular holes follow the same ordered Clipper operations.

## Region ownership and safety subtraction

After hole protection, the complete upper merged footprint is offset by
`distance_scaled`. Regions are processed in their existing vector order with
indices ascending; region IDs are not re-sorted. For every enabled region:

1. intersect that region's pair-start upper-layer surfaces with the shrunken
   complete upper footprint; after a higher pair these surfaces already contain
   its top-down cascade mutation, but they stay fixed during this region loop;
2. union the intersection into ordered candidate islands `p`;
3. remove each candidate island for which `difference(p, current_poly)` is
   empty, because it is already fully covered by the original current merged
   footprint;
4. concatenate the current lower ExPolygons followed by the remaining `p`, put
   every path in the Clipper Subject role, then union them;
5. rebuild that full collection as Internal surfaces with the exact default
   metadata tuple `(-1.0, 1, -1.0, 0)` for thickness, thickness layers, bridge
   angle, and extra perimeters;
6. subtract `p` from every other current lower region using the separate-path
   10-coordinate safety offset;
7. rebuild each affected other collection with the same exact Internal default
   metadata tuple.

`current_poly` is fixed before the region loop. It is not recomputed after a
region mutation. A partially covered candidate is retained as a whole island;
only fully covered islands are removed. Enabled regions still perform the
union/set path when `p` is empty, matching upstream normalization and metadata
reset semantics. A skipped pair and the never-lower top layer are not passed
through `set`; their existing metadata, including nondefault test values,
remains unchanged.

## Rust destination boundary

The production stage belongs in real modules under:

```text
crates/ares-core/src/project_slice/conical_overhang.rs
crates/ares-core/src/project_slice/conical_overhang/geometry.rs
```

The stage exposes one crate-private orchestration API that consumes mutable
`PostRegionPrintObject` values, their ordered resolved object contexts, and the
runtime coordinate scale. Geometry helpers consume and return existing
`ExPolygon`, `Polygon`, and `RegionSurface` types. There is no parallel contour
model and no conversion through the old public pipeline.

The existing Clipper rewrite gains only the reusable production operations
required by the source boundary: ExPolygon union, XOR, arbitrary ExPolygon
offset/path offset access, and exact safety-offset difference. These helpers
retain the existing two-pass Paths-then-PolyTree NonZero ordering. Same-region
union concatenates both inputs in order as Subject paths instead of treating
the second input as a Clip operand. All operations propagate `ClipperError`.

The old `contours/overhang_printable.rs` rectangle compatibility shell is not
used, called as fallback, or widened in this task. Its global switch,
rectangle-only behavior, and missing region ownership are not the Task 22L
contract.

A fresh executable-test audit found no remaining test that reads or pins an
Orca checkout, commit, source blob, or source text, so Task 22L has no obsolete
source-pinning test to delete. Source identities remain documentation and
ignored review evidence only.

## Pre-implementation oracle contract

An ignored, independently compiled C++ oracle must mechanically use the fixed
modified Clipper 6 sources and reproduce the exact operations above before
production implementation begins. It must refuse a mismatched Orca tree or
source blob and emit ordered binary and text vectors for:

- normal- and large-scale epsilon, distance, and hole-area f32 bit patterns,
  including angle `+0` negative distance zero, angle `-0` positive distance
  zero, and negative-zero hole area;
- empty, single-layer, angle-90, disabled, and interior-empty gates;
- surface-vector cardinality when the contained ExPolygon itself is empty;
- a three-layer top-down cascade using nominal rather than actual heights;
- arbitrary non-rectangular geometry and complete erosion;
- hole zero with a real covered hole, strict-less-than, equal, full-cover,
  partial-cover, no-intersection, multiple, and non-rectangular hole cases;
- merge-ineligible but cardinality-nonempty regions;
- overlapping same- and cross-region merge inputs that lock both union passes;
- multi-region enabled/disabled and multiple-enabled ordered ownership, plus
  exact normal- and large-scale 10-coordinate safety subtraction;
- the pair-wide current-layer gate: when one enabled lower region is nonempty,
  another enabled but empty lower region still receives its upper projection
  and subtracts that projection from the first region with the safety offset;
- fully covered, partially covered, and empty `p` candidates;
- same- and other-region Internal metadata rebuilding for nonempty and empty
  `p`, skipped/top metadata preservation, and unchanged sidecars/plan
  identities.

The fixed oracle contains 40 objects. Every oracle-owned source file remains
below 400 physical lines and uses real C++ translation units rather than
textual source inclusion. Oracle-owned translation units compile as C++20 with
`/O2 /fp:precise /W4 /WX /DNDEBUG`. Fixed `clipper.cpp` is compiled and linked
as its own translation unit with `/W0` because the unchanged third-party source
emits C4244 at line 3762 under `/W4 /WX`; it is never textually included.

Independent clean compilation and two complete runs froze these exact outputs:

| Mode | Kind | Bytes | SHA-256 |
|---|---|---:|---|
| synthetic | binary | 23,615 | `7acbe44192edf030fb4b93cdab3593d83dde5800a5faa62bdb8d12002d5c8779` |
| synthetic | text | 59,481 | `6dc0fbd639b4eec91b8af2dba9fe953262ace6e550abfa9982125de03979e9a8` |
| stepped disabled | binary | 490 | `0834c61cc48aece1afd52d060c5c2a58f7243124664ad0a7dd3f500d6735b790` |
| stepped disabled | text | 1,405 | `576047f25c1b781477c5aff12c7d738f91710370401c109192544d71f928cf8b` |
| stepped enabled | binary | 554 | `33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505` |
| stepped enabled | text | 1,478 | `a8301caa9cd3a5a504b60eaa830379cabcf0bc88dc644b4a0996c5161409ba21` |
| KSR disabled | binary | 2,008,706 | `7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07` |
| KSR disabled | text | 2,795,812 | `6ac1b174fa012c46b43d537f42a6f35977497b244183b995fa3343c8b7f33d2c` |
| KSR enabled | binary | 2,370,813 | `46ac3ce00c40e2ba812d4f9589ce8d996949ab1e97a301243c28131865d834dc` |
| KSR enabled | text | 3,256,773 | `d77bedca008e1b54b3c32e2e746be47d51edcea5bad5e725967f964078ee2ef1` |

Both runs match byte-for-byte for every row. Synthetic case 3 freezes exact
hole equality at f32 `0x53800000` and signed f64 area
`0xc270000000000000`; cases 27 and 28 freeze negative-zero hole bits
`0x80000000` and angle-negative-zero distance bits `0x00000000`, while angle
positive zero produces distance bits `0x80000000`. Large-scale cases freeze
epsilon `0x41200000` and nonzero squared hole bits `0x503a43b7`. Cases 6, 7,
10, and 19 distinguish reset from preserved nondefault Internal metadata.
Case 38 combines two reverse pairs, two enabled regions, prior-pair cascade
state, and nominal-height use. Expected constants never change to accommodate
Ares output. Case 39 proves the current-layer gate is pair-wide rather than
per-region: lower region 20 starts empty but receives the upper island, while
lower region 10 receives the exact safety notch at `x=799990` and
`y=199990..800010`. Its trace locks `region_passes=2`,
`projected_before=1`, `projected_after=1`, `empty_projection=1`, and
`safety_diffs=2`. An independently injected per-region empty skip changes both
the trace and ordered output, so the frozen identity rejects that defect.

## Real 3MF anti-hardcoding vectors

Tests build a deterministic two-layer stepped project from real 3MF entries.
Both archives replace the committed KSR profile's angle `55` with `45`, then
replace the mesh/model/settings entries. They have identical mesh, transforms,
object metadata, `layer_height=0.2`, angle `45`, and hole size `0`; after those
common replacements, the only semantic difference between them is
`make_overhang_printable` changing from `"0"` to `"1"` in the project settings
entry.

Native tests serialize the pair with the existing Rust `KsrArchive`; Chromium
serializes the same sorted entries with tracked `fflate`. Their physical ZIP
bytes are intentionally encoder-specific and are frozen separately. Their
ordered semantic-entry framing, loaded typed Options, Task 22K bytes, and Task
22L bytes must agree across encoders. Tests must never require the Rust ZIP
writer to reproduce `fflate` bytes or vice versa.

Both archives pass through the real ZIP loader, typed Option composition,
planning, mesh slicing, region composition, Task 22K, and Task 22L. Tests must
prove:

- loaded effective object Options are `0.2`, `45`, and `0`;
- the only region's effective switch is false or true from the archive;
- Task 22K input bytes are identical because Options are not encoded there;
- false Task 22L output differs from K only by magic;
- true Task 22L output matches the pre-implementation fixed-source oracle and
  changes only retained lower region surfaces;
- upper retained geometry, plan values, IDs, Options, ordering, and complete
  sidecars remain unchanged;
- both archives and all checkpoint outputs are repeatable.

The corrected 45-degree identities are frozen before orchestration GREEN:

- native Rust ZIPs: 181,446 bytes /
  `ee928a255109b491b0640da279b86d9282c573ec49a400e3cc4529eac915030e`
  and 181,447 bytes /
  `be286d7abb2bef8ab5e8b650657b114ea35c4dcff3a1463eba1a0dd278a89faa`;
- browser `fflate` ZIPs: 190,380 bytes /
  `c4c0ea05709a6fadd8b2d0d6d34dab1cad5420865c5993b58b9d8e91a8f73313`
  and 190,381 bytes /
  `130260c5c63846759aa66d25e68ff9bb07cf5aeec86ef7da9476c12761f3836d`;
- shared disabled/enabled semantic frames: 1,020,460 bytes /
  `ade484830a6492b50c3233e51debf5eab1db7d3e3bbf81fa8cd72f10226ea9ef`
  and 1,020,460 bytes /
  `f61089d040d1edf002f1dedca66b433e4982e18b9ce69a6385aa42dbf4c780b9`.

Additional real-3MF mutations prove angles `-0.1` and `90.1` and hole size
`-0.1` fail with their exact errors at the new Task 22L orchestration boundary
even with the region switch false. Pure tests cover nonfinite raw values,
angle-before-hole precedence, a later invalid object without partial mutation,
finite f64 values whose exact f32 derivations overflow at both scales, and the
single unified Clipper error mapping. Synthetic fixed-oracle vectors, not ad
hoc rectangles, cover holes and multi-region ownership.

## KSR checkpoint contract

The committed KSR archive contains, in `Metadata/project_settings.config`:

- `layer_height = "0.2"`;
- `make_overhang_printable = "0"`;
- `make_overhang_printable_angle = "55"`;
- `make_overhang_printable_hole_size = "0"`;
- `bottom_shell_layers = "3"`;
- `top_shell_layers = "5"`;
- `sparse_infill_density = "15%"`;
- `wall_loops = "2"`.

There is no model, object, volume, or layer-range override for these keys. The
effective one-region switch is false and all four merged-footprint eligibility
fields remain nonzero. Task 22L must nevertheless execute the stage after Task
22K and prove these values came through the real 3MF resolver.

The Task 22L checkpoint uses magic `ARES22L\0` and otherwise keeps the Task 22J
wire layout. For the committed KSR project every byte after the eight-byte
magic equals the released Task 22K checkpoint. The independently derived
disabled KSR identity is:

- length: `2,008,706` bytes;
- SHA-256: `7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07`.

The KSR check alone proves only the disabled real-project path. It cannot
substitute for the enabled real-3MF and fixed-source geometry vectors. The
committed project and reference G-code remain unchanged. Public slicing still
returns `ProjectSlicingIncomplete`; Task 22L does not claim G-code parity.

## WASM and browser boundary

The previous non-default `task22k-browser-oracle` feature is replaced, not
aliased, by `task22l-browser-oracle`. Default core and adapter builds expose no
Task 22 hook. The feature build exposes exactly:

- `task22lBrowserInputOracle`, returning the complete `ARES22K` input;
- `task22lBrowserOracle`, returning the complete `ARES22L` output.

Native Task 22K regressions remain under `cfg(test)`; no obsolete K browser
export remains. The browser parser accepts J, K, and L magic with exact EOF,
safe-integer, dense retained-layer, dense region-ID, Internal-surface, and
record-identity checks.

Before fixture fetch, an independent L known-answer vector exercises changed
geometry and rejects truncated or trailing input. Fresh Chromium then proves:

- the public KSR boundary remains incomplete;
- feature exports are exactly the two L functions;
- exact KSR K input and disabled L output identities;
- the browser-specific `fflate` ZIP identities, shared semantic identities,
  and fixed output of the real two-layer false/true Option-only pair;
- repeatability, complete parsing, unchanged plan/sidecars, and changed lower
  retained geometry for the enabled archive;
- two consecutive runs from fresh optimized bindgen output.

Browser implementation is split into real imported modules before either
existing browser file would exceed its approved line budget.

## Structural constraints and deferrals

Every changed Rust production and test file remains below 400 physical LOC.
Tests live in separate real modules. Rust source splitting may not use
`include!`, `include_bytes!`, or related embedding macros. Existing fixture
embedding remains fixture input, not source splitting; Task 22L adds no source
embedding.

No new dependency, crate, unsafe code, native-only API, filesystem access,
process API, thread API, fixture-dependent production path, or legacy fallback
is introduced. Task 22L tracked tests do not execute Git, inspect Orca source
identity, or require the ignored source checkout. Source identity belongs only
in documentation and review evidence.

Explicitly deferred behavior includes:

- caller and per-layer-pair cancellation checks;
- MMU/painted and fuzzy segmentation and interlocking;
- XY contour/hole and elephant-foot compensation;
- `make_slices`, surface typing beyond the fixed Internal reset, perimeters,
  fill, supports, extrusion paths, G-code assembly, metadata, post-processing,
  and normalized reference-G-code comparison.

## Acceptance and review gate

Task 22L is complete only when:

1. the fixed-source identities above are independently reproduced and
   reviewers approve the exact spec/plan bytes before implementation;
2. each implementation package records focused RED evidence before production
   code for that package exists;
3. fixed geometry, stage, real-3MF, KSR, public-boundary, WASM, and Chromium
   contracts pass;
4. Task 22A-K predecessor checkpoints remain exact;
5. rustfmt, strict workspace Clippy, workspace checks, full nextest, wasm32,
   export, LOC, macro, unsafe, hardcoding, fixture identity, and diff gates
   pass;
6. an independent read-only reviewer assesses requirement completeness,
   logical correctness, edge cases, code quality, test coverage, and actual
   execution, returning a concrete repair checklist;
7. the main thread repairs every finding and the same review thread repeats
   until all P0-P3 lists and the repair checklist are empty;
8. exact reviewed bytes are committed, pushed normally, and pass exact-SHA
   Tier-1 before the next source slice starts.

Any checkpoint mismatch is an implementation defect until fixed-source
evidence and independent review prove otherwise.
