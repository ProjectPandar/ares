# Task 22G: Clipper 6 Closed Offset and Project Slice Closing

## Status and objective

This specification is a draft. Production or test implementation may begin
only after the exact specification and implementation-plan bytes receive
independent fixed-source/spec, independent Ares/plan, and direct default-model
approval.

Task 22G is the next bounded source-rewrite package in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`ca667a8a3b595cfd2bdde5ced357010830051360` produces the exact ordered
Task 22F pre-closing `ExPolygon` stream for every selected object, volume, and
layer. Task 22G ports the closed-polygon part of Orca's bundled ClipperOffset,
the directly used ExPolygon `offset_ex`/`offset2_ex` wrappers, and the project
consumer that applies the 3MF-derived slice closing radius.

The committed project resolves `slice_closing_radius=0.049`. Under its normal
coordinate scale and Orca's two float-narrowing seams this becomes
`+49000.0f32` followed by `-49000.0f32`. Those numbers are acceptance facts,
not production constants. The implementation must derive them from each
resolved object's 3MF Option and the selected project coordinate scale.

Task 22G stops immediately after closing. It does not select the largest
contour, simplify contours, combine volumes, generate surfaces or toolpaths,
or emit G-code. The public project API traverses the owned intermediate and
continues to return `SliceError::ProjectSlicingIncomplete`.

No implementation may recognize the fixture, read the reference G-code,
invoke Orca at runtime, hardcode 0.049 or 49000, substitute a different offset
or Boolean engine, or infer missing behavior from the expected output.

## Fixed Ares and upstream identity

The fixed Ares baseline is commit
`ca667a8a3b595cfd2bdde5ced357010830051360`, tree
`56343294a9195f53f63c6d3295272186c7ca64cd`. Exact-SHA Tier-1 run
`29642639170` passed format, Ubuntu/Linux, macOS, Windows, and WASM including
the real-project browser test.

All upstream citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored checkout may have a
different HEAD; source evidence is read from the fixed commit or verified
fixed blobs.

Primary fixed blobs are:

- `deps_src/clipper/clipper.hpp`,
  `06637effce040fa7d87c368437cb32398f19ee92`;
- `deps_src/clipper/clipper.cpp`,
  `1f16446ac8da1f0b9c802d8a9dee33f766919f6b`;
- `src/libslic3r/ClipperUtils.hpp`,
  `9c2fa239263c0cb097a4b4c3db823821615bd7c7`;
- `src/libslic3r/ClipperUtils.cpp`,
  `2f97e08f536e93c5fd27b4614980072285d2ce22`;
- `src/libslic3r/TriangleMeshSlicer.hpp`,
  `1f7bba9d273f930785279ef82ef3258f191acd3e`;
- `src/libslic3r/TriangleMeshSlicer.cpp`,
  `2c1c0da23fe569c93b5d243a14494792956533d0`;
- `src/libslic3r/PrintObjectSlice.cpp`,
  `07eb885eda83a495001467c22c0452dfc36e55c2`;
- `src/libslic3r/libslic3r.h`,
  `f4291d36df8175c700fa9374c5b5c07e6880e706`;
- `src/libslic3r/Point.hpp`,
  `039f361eaa18db9c6e7d2c35d1c61af78bcad51b`;
- `src/libslic3r/PrintConfig.hpp`,
  `0a7b7ba36f87c3d4517daf96d7d8825812e66358`;
- `src/libslic3r/PrintConfig.cpp`,
  `982953afa50af0217a4d64639116ff4a2e596e90`;
- `tests/libslic3r/test_clipper_utils.cpp`,
  `fe9961a771649c24fd7a2f5c6718995d6865c03f`;
- `tests/libslic3r/test_clipper_offset.cpp`,
  `3b855130be8ec9382438a8980a3713b6c0eac0a0`.

The closed offset code remains covered by the BSL-1.0 component notice added
with Task 22F. No new third-party implementation or license is introduced.

## Exact upstream rewrite boundary

The production dependency closure is:

- `clipper.hpp:138-139,144-167,538-575` for JoinType, the PolyNode fields used
  by ClipperOffset, and ClipperOffset state;
- `clipper.cpp:63-65,73-106,128-134,150-161,1000-1036` for constants,
  Clipper rounding, PolyNode child ownership, outer-node removal, and bounds;
- `clipper.cpp:3345-3777` for unit normals, closed AddPath normalization,
  orientation repair, positive/negative cleanup, offset preparation, and all
  closed square/miter/round corner branches;
- `ClipperUtils.hpp:17-34,326-355,389-393` for defaults, the Paths-to-ExPolygon
  conversion seam, and the used polygon, ExPolygon, `offset_ex`, and
  `offset2_ex` declarations;
- `ClipperUtils.cpp:264-293,303-315,333-351,360-410,437-558,560-585` for
  per-path raw offset, NonZero Difference, single-pass cleanup union,
  EvenOdd/NonZero ownership recovery, expand/shrink dispatch, ExPolygon hole
  ownership, and the exact two-stage wrapper;
- `TriangleMeshSlicer.hpp:20-46` and
  `TriangleMeshSlicer.cpp:1738-1824,2003-2034` for the closing parameters,
  branch selection, and fill-rule context already completed by Task 22F;
- `PrintObjectSlice.cpp:145-221` for per-object Option ownership, fixed
  `extra_offset=0`, and both parameter-bearing `slice_volume` consumers;
- `libslic3r.h:59-70,92-94` and `Point.hpp:651-669` for coordinate scaling;
- `PrintConfig.hpp:946` and `PrintConfig.cpp:6020-6028` for the existing
  nonnegative `slice_closing_radius` Option contract;
- `test_clipper_utils.cpp:13-43` and `test_clipper_offset.cpp:14-51,88-125`
  as source-owned behavioral vectors, not executable source-pinning tests.

`ClipperUtils.hpp:400-410` and `ClipperUtils.cpp:592-610` describe neighboring
generic closing helpers but are context only. The KSR consumer does not call
them. It calls `offset2_ex(union_ex(...), offset_out, offset_in)` directly.
Task 22G must not invent a project `closing_ex` call or claim the deferred
Polygons/Surfaces closing overloads.

Only `EndType::ClosedPolygon` is included. Closed lines, open butt/square/round
ends, polyline providers, safety offset, opening, surfaces, variable offset,
and direct ClipperOffset-to-PolyTree execution are deferred.

## Architectural contract

ARD-0024 remains normative. Task 22G extends the one pure-Rust source-cited
Clipper engine; it does not add another polygon engine, C++ binding, FFI,
unsafe graph, native-only path, or platform-specific oracle. The offset
generator emits ordered integer paths and delegates all cleanup Boolean and
PolyTree work to the released Task 22F kernel.

Offset cleanup is not allowed to call Task 22F's two-pass `union_ex` wrapper.
The fixed offset wrappers use one Clipper execution for each Paths or PolyTree
cleanup. Ares must expose a narrow internal single-pass seam and preserve that
distinction because it affects path order and ownership.

The private Rust destination is the existing `geometry::clipper` module with
real `mod` children for closed offset input, generation, execution, and
ExPolygon wrappers, plus one `project_slice::closing` stage. No new workspace
crate or public general-purpose geometry API is added.

## Closed offset domain and numeric contract

The included join domain is exhaustive:

```text
JoinType = Square | Round | Miter
EndType  = ClosedPolygon only
```

ClipperOffset defaults are miter limit 2.0, arc tolerance 0.25, and shortest
edge length 0.0. The directly used Orca wrappers override the miter limit to
3.0 for the default Miter join and set shortest edge length to
`abs(delta * 0.005)`. Round joins interpret the wrapper's miter-limit argument
as arc tolerance exactly as fixed source does.

The algorithm receives offset delta as `f64`, matching ClipperOffset's C++
state, but wrapper entry points accept `f32`, matching Orca's signatures.
Every generated integer coordinate uses the released Clipper `fixed_round`
contract, including the special 0.49999999999999994 case. Rust `round()` is
forbidden.

Near zero means strictly `-1e-20 < delta < 1e-20`. Near-zero generation copies
accepted closed contours in stored order; it is not an identity fast return.
Cleanup then uses the independent exact predicate `delta > 0`: a strictly
positive sub-tolerance delta uses Positive cleanup, while `+0.0`, `-0.0`, and a
negative sub-tolerance delta use Negative cleanup. `offset_paths(+0.0)` and
`offset_paths(-0.0)` likewise take the shrink branch because neither value is
greater than zero. No layer is allowed to return the input directly merely
because the delta is zero or near zero.

Miter threshold is `2/(limit*limit)` when the limit is greater than 2 and 0.5
otherwise. Arc tolerance is 0.25 when configured nonpositive, otherwise the
minimum of configured tolerance and `abs(delta)*0.25`. Steps, sine, cosine,
and steps-per-radian follow the fixed `acos`, `sin`, and `cos` sequence with
the excessive-precision cap.

## Closed path normalization and orientation

For each added closed path:

1. empty input is ignored;
2. trailing points whose distance from the first point is strictly below the
   configured shortest-edge threshold are removed;
3. the first point is retained;
4. later points whose distance from the last retained point is strictly below
   the threshold are skipped;
5. a retained path with fewer than three points is discarded;
6. its lowest point is the point with greatest Y, then smallest X;
7. the global lowest closed contour controls orientation repair.

When shortest-edge length is zero, equality rather than distance is used.
The comparison is strict `<`, not `<=`; KSR's threshold is exactly 245 and
tests must distinguish the nearest realizable squared distances below and at
the threshold: 60020 (`244^2 + 22^2`) and 60025 (`245^2`).

Before generation, if the globally lowest contour is not positive/CCW under
fixed Clipper area, every stored closed polygon is reversed. Input order is
otherwise unchanged. The higher Orca wrapper processes paths one at a time,
records original orientation, reverses the delta for a CW path, and reverses
its result afterward. This per-path seam is mandatory for holes.

## Miter, square, and round join behavior

Unit normal for an edge is `(dy, -dx) / length`. For each vertex, the cross
product of adjacent normals becomes `sinA`.

- If `abs(sinA * delta) < 1` and the normal dot product is positive, one
  previous-normal point is emitted and the corner returns without advancing
  the previous index.
- If `sinA * delta < 0`, the concave corner emits previous-normal point,
  original vertex, and current-normal point in that order.
- Miter computes `r=1+dot`; `r >= miter_threshold` emits one miter point,
  otherwise it executes the Square branch.
- Square uses the fixed quarter-angle `tan(atan2(...)/4)` construction and
  emits two points.
- Round uses at least one step, rotates the previous normal with the prepared
  sine/cosine pair, and emits the exact current-normal endpoint last.

No point rotation, canonicalization, deduplication, or after-the-fact corner
repair is added outside fixed source behavior.

## Positive, zero, and negative offset cleanup

Raw offset destination paths are cleaned by the Task 22F engine:

- positive delta executes one Union with Positive/Positive fill;
- negative delta computes fixed Clipper bounds, adds the four-point rectangle
  expanded by 10 integer units, sets `reverse_solution=true`, executes one
  Union with Negative/Negative fill, and removes the outermost result.

These bullets describe ClipperOffset's internal cleanup, which does not
short-circuit an empty generated destination. Wrapper cleanup is a separate
execution: positive `expand_paths` unions raw paths with NonZero/NonZero, while
negative `shrink_paths` first short-circuits an empty raw result and otherwise
performs its own fixed outer rectangle and Negative/Negative union. Therefore
an empty wrapper shrink remains empty and never constructs an outer rectangle
from synthetic zero bounds. The implementation and tests must keep these two
cleanup levels distinct.

Paths cleanup removes its first outer path. PolyTree cleanup promotes the
outer node's children only when the fixed one-root/has-children condition
holds; otherwise it clears the tree. Child order and parent ownership remain
observable.

## ExPolygon `offset_ex` and `offset2_ex` ownership

For each ExPolygon, the wrapper offsets its contour first. If the contour
vanishes, the ExPolygon vanishes. Each hole is offset independently with the
opposite delta.

For positive ExPolygon offset, remaining hole paths are reversed and appended
after contour paths. For negative offset, enlarged holes are subtracted from
the shrunken contour with one NonZero Difference; a hole may consume the
contour completely.

The complete ownership fill matrix is normative:

- the negative contour/hole subtraction is one Difference with
  NonZero/NonZero fill;
- converting the Paths result of the single-ExPolygon `offset_ex` overload uses
  one EvenOdd/EvenOdd Union because its `do_union` argument defaults to false;
- multiple ExPolygons are processed independently, and positive offset performs
  one final NonZero Paths Union only when more than one input ExPolygon produced
  output;
- negative multi-ExPolygon offset does not perform a cross-ExPolygon union;
- the multi-ExPolygon PolyTree `offset_ex` overload always uses one NonZero
  Union to recover contour/hole ownership;
- second-stage positive wrapper cleanup uses NonZero Union, while second-stage
  negative cleanup uses the fixed Negative outer-rectangle path.

EvenOdd and NonZero are not interchangeable implementation choices even when a
particular normalized KSR layer produces the same result.

`offset2_ex(input, first, second)` is exactly:

1. `expolygons_offset(input, first)` to ordered Paths;
2. `offset_paths<PolyTree>(paths, second)` using positive or negative cleanup;
3. fixed PolyTree-to-ExPolygon conversion.

It is not replaced by a newly designed morphological algorithm or Task 22F's
two-pass `union_ex`.

## Project post-closing stage

The stage consumes each `PreClosingPrintObject`, its corresponding resolved
project object, and the existing coordinate scale. It retains planned object
identity and layer plan, source volume index, `VolumeOrdinal`, volume type,
original slicing mode, every ordered layer slot, and every ordered ExPolygon.
Empty slots remain present.

The stage reads `slice_closing_radius` from the matching resolved per-object
`ObjectOptions`. It does not read a global default, filename, fixture hash,
reference G-code, or metadata. Negative or nonfinite external radius, and a
finite external radius whose exact scaling chain produces a nonfinite f32
delta, return one deterministic `SliceError::InvalidInput` naming
`slice_closing_radius`. Internal object association mismatches remain
invariants.

Orca's exact narrowing chain is normative:

```text
resolved Option f64
  -> MeshSlicingParamsEx closing_radius f32
  -> scaled intermediate f64 using project CoordinateScale
  -> offset_ex/offset2_ex argument f32
  -> ClipperOffset delta f64
```

KSR therefore narrows 0.049 to approximately 0.04899999871850014f32, scales
to approximately 48999.99871850014f64, then narrows to exactly 49000.0f32.
A direct f64-to-i64 scale is observably not the contract.

`extra_offset=0.0f32` comes from `PrintObjectSlice.cpp:166-168,184-187` and is
an upstream consumer constant, not a missing Option. With nonnegative radius,
the project branch computes positive `offset_out` and negative `offset_in` and
calls `offset2_ex` only when both signs are active. A zero radius keeps every
Task 22F ExPolygon record and point without offset generation. Canonical
post-closing serialization still starts with `ARES22G\0`; zero radius does not
change its format marker back to `ARES22F\0`.

`PositiveLargestContour` retains all post-closing ExPolygons in Task 22G.
Task 22H owns largest-contour selection.

## Normative behavioral vectors

Committed tests freeze complete points, start positions, siblings, and
ownership where source behavior is ordered. Minimum vectors are:

1. Empty and degenerate closed paths; repeated terminal and consecutive
   points; shortest-edge squared distances immediately below and equal to the
   threshold; CW and CCW input.
2. Positive, `+0.0`, `-0.0`, positive/negative sub-tolerance, ordinary
   negative, and complete-erosion offsets for convex and concave polygons,
   including complete cleanup ordering and ownership.
3. Miter success, sharp-corner Square fallback, direct Square, and Round with
   fixed arc tolerance and step order.
4. Negative bounds rectangle, reverse-solution output, outer removal, and
   multiple sibling order.
5. The fixed Orca square-with-hole vector:
   `offset_ex(+5)` produces contour
   `[(205,205),(95,205),(95,95),(205,95)]` and hole
   `[(145,145),(145,155),(155,155),(155,145)]`.
6. The same vector under `offset2_ex(+5,-2)` produces contour
   `[(203,203),(97,203),(97,97),(203,97)]` and hole
   `[(143,143),(143,157),(157,157),(157,143)]`.
7. Distinguishing vectors for single-ExPolygon EvenOdd recovery,
   multi-ExPolygon NonZero recovery, and negative NonZero Difference, plus
   disjoint/overlapping ExPolygons, surviving/vanishing holes, and an enlarged
   hole that consumes a contour.
8. Single-pass wrapper ordering tests that would differ from two-pass
   `union_ex` substitution.
9. Project object association, empty slot and metadata retention, zero-radius
   payload identity with an `ARES22G\0` marker, process-base radius, object
   override precedence, normal/large-bed scale, a non-integer narrowing vector,
   and a finite radius whose scaled f32 delta overflows to nonfinite.
10. Complete KSR oracle, representative layers, repeated execution, unchanged
    fixture hashes, and the still-incomplete public lifecycle.

Tests contain fixed expected values only. They never open Orca source, execute
an Orca binary, or read the reference G-code.

## Fixed-source oracle protocol

The canonical Task 22G oracle consumes the independently corrected Task 22F
`ARES22F` stream: 1,645,481 bytes, SHA-256
`209c6149c93994cc3ae6fa8e2f8f43dc9875b1b07b2320da9e67d8a2c43ab6e2`.
It applies only fixed-source `offset2_ex(+49000.f,-49000.f,Miter,3.0)` and
encodes the same object/volume/layer ownership with magic `ARES22G\0`.

The fixed MSVC `/fp:precise` probe is 396 LOC with SHA-256
`964308756ea4c24d942ac5b51c5219eaf807789ece0f374a906af596b67c6a46`.
Five runs are byte-identical. The probe is ignored evidence and is never a
build, test, or runtime dependency. Two independent read-only fixed-source
reviewers approved the corrected probe and all listed oracle values with no
remaining P0-P3 findings.

## KSR acceptance at this boundary

The complete post-closing encoding is exactly 1,644,681 bytes with SHA-256
`29ffb501c54190dd4336cc1371fc5e480c5b87ac6a8184366bd072bf5cb90919`.
It retains 1 object, 1 model-part volume, 460 ordered Regular layer slots, and
contains 2,890 contours, 395 holes, and 99,212 points.

Representative records are:

- layer 0: 6 contours, 6 holes, 922 points, 14,913 bytes, SHA-256
  `28fbbcc66d73c037a5dbb3c60363d83bfaeaaf1d9d8a49451594f227ea0d4fcf`;
- maximum-loop layer 46: 16 contours, 25 holes, 2,860 points, 46,233 bytes,
  SHA-256
  `8dba7c5e51c74e803903b513c5165dffb9d1c55be108e39fbccca4309a603e69`;
- layer 459: 737 bytes, SHA-256
  `c8822b67958531cb4b043d338b53f7329e0b00cb4f08108306763e763cd52f80`.

The committed project and reference G-code fixture hashes remain respectively
`698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`
and `10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3`.

## Included behavior

- ClosedPolygon ClipperOffset input, orientation, normals, and three joins.
- Positive, near-zero, and negative Paths cleanup through Task 22F.
- The exact directly used ExPolygon `offset_ex` and `offset2_ex` wrappers.
- Single-pass Paths/PolyTree union seams required by those wrappers.
- Per-object 3MF-derived slice closing radius and exact f32 scaling chain.
- Owned post-closing project stage and the complete fixed KSR oracle.
- Native, WASM, and browser-safe deterministic Rust behavior.

## Explicitly deferred behavior

- ClosedLine and all open end types; polyline and surface overloads.
- Safety offset, generic closing/opening helpers, and variable offset.
- Post-closing largest-contour selection from Task 22H.
- Resolution mapping, simplification, and StrictlySimple from Task 22I.
- Cross-volume negative/modifier Boolean combination and regions/surfaces.
- Perimeters, fill, supports, extrusion paths, G-code assembly, metadata,
  post-processing, and complete normalized reference-G-code equality.

## Structural and platform constraints

- Every Rust production and test file remains below 400 physical LOC.
- Tests live in separate real `mod` files.
- `include!` and `include_bytes!` may not split source or test modules.
- No unsafe, FFI, native Clipper, filesystem, process, thread, UI, terminal,
  OpenGL, platform branch, or second polygon dependency enters `ares-core`.
- No fixture identity, reference-G-code read, output lookup table, or
  KSR-specific production branch is allowed.
- Existing obsolete executable Orca source-pinning tests remain deleted; no
  new source-path/line/hash pinning test is added.
- Tier-1 remains WASM browser, Windows, macOS, and Linux.
- A single non-default `task22g-browser-oracle` build feature may compile the
  canonical byte encoder and a byte-only WASM test hook. It changes no slicing
  branch or expected value, is absent from normal artifacts, exposes no owned
  geometry type, and exists only so a real browser can verify the same complete
  checkpoint bytes as native tests. It is not a runtime option or fallback.

## Verification and review exit criteria

Implementation follows strict RED-GREEN-REFACTOR packages. Before any offset or
closing behavior is implemented, the complete native KSR oracle and real
browser oracle are registered against a signature-only incomplete seam and
must fail for missing Task 22G behavior. That same oracle remains registered
through every package and becomes GREEN only after the source-cited behavior is
complete. Each behavior package also first registers its focused failing test,
records the expected RED, implements the minimum source-cited behavior, and
runs focused plus relevant regression tests before review.

After implementation, one independent read-only reviewer must assess the same
candidate across requirement completeness, logical correctness, edge cases,
code quality, test coverage, and actual execution results. It returns a
prioritized fix list. Only the main thread changes code; the same reviewer
rechecks after every repair until all six axes pass or a concrete external
blocker is reproduced.

Then fresh whole-candidate specification, quality, and direct default-model
reviews must approve unchanged bytes. Any code or test edit invalidates those
approvals. Documentation review, the complete native/WASM/browser matrix,
exact manifest audit, Conventional Commit, normal push, remote identity, and
exact-SHA Tier-1 success are required before Task 22G is released.

Task 22G release does not complete the persistent user goal. Work proceeds to
Task 22H and later source-cited slices until normalized KSR G-code parity and a
final six-dimensional result review are both present.

**Status: DRAFT — implementation is forbidden until the exact specification
and plan receive all pre-implementation approvals.**
