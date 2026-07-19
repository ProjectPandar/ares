# Task 22I: Resolution-Gated ExPolygon Simplification

## Status and objective

This specification is a draft. Production or test implementation may begin
only after the exact specification and implementation-plan bytes receive
independent fixed-source/specification, independent current-Ares/plan, and
direct default-model approval.

Task 22I is the next bounded source-rewrite package in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`bf0d91283f1d2e704633dd6ea4022ea79bd34e8b` produces the exact ordered Task
22H post-largest-contour `ExPolygon` stream. Task 22I ports Orca's global
`resolution` mapping, closed-loop Douglas-Peucker simplification,
`ExPolygon::simplify`, and the exact Clipper StrictlySimple repair closure.

The stage reads only the resolved global `resolution` already stored in the
input 3MF. It does not read a filename, fixture digest, reference G-code,
metadata, process-global default, or out-of-band test parameter. The committed
KSR project has `resolution=0.012`, which enables the fixed `0.0025 mm`
tolerance and yields scaled tolerance 2500 under its existing normal
coordinate scale.

Task 22I stops immediately after simplification. It does not combine model
volumes, create regions or surfaces, generate perimeters, infill, supports,
extrusion paths, or G-code. The public project API executes the new stage and
continues to return `SliceError::ProjectSlicingIncomplete`.

## Fixed Ares and upstream identity

The fixed Ares baseline is commit
`bf0d91283f1d2e704633dd6ea4022ea79bd34e8b`, tree
`7f2bab0d44c35869542ee162fb2f4a4771456509`. Exact-SHA Tier-1 run
`29665234136` passed Windows, Ubuntu/Linux, macOS, format, and WASM.

All upstream citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored Orca checkout is
evidence only; tracked tests never inspect it.

Fixed source blobs are:

- `src/libslic3r/PrintConfig.hpp`,
  `0a7b7ba36f87c3d4517daf96d7d8825812e66358`;
- `src/libslic3r/PrintConfig.cpp`,
  `982953afa50af0217a4d64639116ff4a2e596e90`;
- `src/libslic3r/PrintObjectSlice.cpp`,
  `07eb885eda83a495001467c22c0452dfc36e55c2`;
- `src/libslic3r/TriangleMeshSlicer.cpp`,
  `2c1c0da23fe569c93b5d243a14494792956533d0`;
- `src/libslic3r/TriangleMeshSlicer.hpp`,
  `1f7bba9d273f930785279ef82ef3258f191acd3e`;
- `src/libslic3r/ExPolygon.cpp`,
  `185e92508449a425064b26690e3d74d06a16fda8`;
- `src/libslic3r/MultiPoint.cpp`,
  `694d3ea9d0b59d81181f05b7bbd4fb617751bb6d`;
- `src/libslic3r/MultiPoint.hpp`,
  `de386b501cbb21056b068c5e456d305afa9089b4`;
- `src/libslic3r/Line.hpp`,
  `d8240702b24168c3e7efa90971ac2babab1dffaf`;
- `src/libslic3r/ClipperUtils.cpp`,
  `2f97e08f536e93c5fd27b4614980072285d2ce22`;
- `deps_src/clipper/clipper.hpp`,
  `06637effce040fa7d87c368437cb32398f19ee92`;
- `deps_src/clipper/clipper.cpp`,
  `1f16446ac8da1f0b9c802d8a9dee33f766919f6b`;
- `src/libslic3r/Polygon.hpp`,
  `7d996055e5d9403f871071ef82baa140c03492b5`;
- `src/libslic3r/libslic3r.h`,
  `f4291d36df8175c700fa9374c5b5c07e6880e706`;
- `src/libslic3r/libslic3r.cpp`,
  `e94d99dcd0ccb856018a0089c6ebb6b8a931d245`;
- `src/libslic3r/Point.hpp`,
  `039f361eaa18db9c6e7d2c35d1c61af78bcad51b`;
- `src/slic3r/GUI/Plater.cpp`,
  `b92a228f442258ece2591c30717f8f935cad930a`.

No new third-party crate or second geometry engine is introduced. Ares extends
its released source-cited Clipper 6.4.2 rewrite under the existing BSL-1.0
provenance.

## Exact upstream rewrite boundary

The direct stage and Option boundary is:

- `PrintConfig.hpp:1554-1562` and `PrintConfig.cpp:5172-5179` for the global
  `resolution` field;
- `PrintObjectSlice.cpp:166-177` for mapping the resolved Option to disabled or
  fixed `0.0025 mm`;
- `TriangleMeshSlicer.hpp:37-48` for the `double` resolution field and
  `TriangleMeshSlicer.cpp:2025-2044` for `scaled<float>`, stage order,
  per-layer/per-ExPolygon traversal, and contiguous append semantics;
- `libslic3r.h:59-70`, `libslic3r.cpp:1-3`, and `Point.hpp:651-669` for the
  selected scaling factor and double-division-then-float conversion.

The exact simplification boundary is:

- `ExPolygon.cpp:223-259` for contour-then-holes simplification and Boolean
  repair;
- `MultiPoint.cpp:164-230`, `MultiPoint.hpp:94-99`, and
  `Line.hpp:41-76,155-188` for the iterative Douglas-Peucker algorithm and
  finite-segment squared distance;
- `ClipperUtils.cpp:1019-1030` for the StrictlySimple NonZero Paths union;
- `ClipperUtils.cpp:303-344,641-654,738-739,813-814` for default non-strict
  construction and the following Paths and PolyTree NonZero union passes;
- `ClipperUtils.cpp:169-205` for ordered PolyTree-to-ExPolygon ownership and
  nested-island traversal;
- `Polygon.hpp:274-280` and `libslic3r.h:143-163` for move conversion and
  contiguous result append order.

The required Clipper implementation closure is:

- `deps_src/clipper/clipper.hpp:130-140,274-313,331-379,420-480,515-528` for
  the strict option bit, output records, input containers, public options,
  maxima storage, and execution state;
- `deps_src/clipper/clipper.cpp:255-308` for exact containment predicates;
- `deps_src/clipper/clipper.cpp:756-856` for closed-path input cleanup;
- `clipper.cpp:1042-1068` for construction and execution reset;
- `clipper.cpp:1072-1103` for Paths-versus-PolyTree execution state;
- `clipper.cpp:1103-1164` for execution success, join/fixup ordering, and the
  strict guard around `DoSimplePolygons`;
- `clipper.cpp:1987-1999` for ordered output-record creation;
- `clipper.cpp:2214-2404` for horizontal direction, chain bounds, maxima
  cursor initialization/consumption, crossing order, consecutive-edge
  promotion, and touching vertex insertion;
- `clipper.cpp:2587-2654` for non-horizontal maxima classification,
  horizontal-pair exclusion, promoted-edge context, maxima collection/sort,
  strict top-edge touch joins, and cleanup;
- `clipper.cpp:2721-2760` for strict-aware collinear output fixup;
- `clipper.cpp:2779-2853` for ordered Paths and PolyTree output;
- `clipper.cpp:2885-2895` for output-point ownership-index updates;
- `clipper.cpp:3196-3238` for the two conditional dependent `FirstLeft`
  repair helpers reached by simple-polygon splitting;
- `clipper.cpp:3787-3851` for `DoSimplePolygons` duplicate-point splits and
  ownership repair.

These ranges form one observable closure. Implementing only Douglas-Peucker,
only `DoSimplePolygons`, or only one Boolean union is incomplete.

## Stage order and Option ownership

Task 22I runs once after Task 22H largest-contour selection and before any
future volume combination. It applies to every retained `Regular`, `EvenOdd`,
`Positive`, and `PositiveLargestContour` layer. The simplifier itself has no
mode branch.

The only external input is
`resolved.views.full.process.print.resolution.0`, already parsed and normalized
from the 3MF project settings. No Task 22I parser or default is added.

The mapping is exact:

1. `resolution <= 0.001` disables the entire stage.
2. `resolution > 0.001` selects fixed `0.0025 mm`, regardless of how much
   larger the raw Option is.
3. Use the same factor from the `CoordinateScale` already selected for project
   intersection and closing, but preserve upstream types exactly: evaluate
   `0.0025_f64 / scale.factor()` as `f64`, cast that quotient to `f32`, then
   promote it to `f64` for simplification. Do not call integer-returning
   `CoordinateScale::checked_scale`.
4. Normal scale therefore yields exactly `2500.0`; LargeBed scale yields
   exactly `250.0`, not the truncated integer value `249`.

Therefore `0`, `0.001`, and the normalized minimum are equivalent no-ops;
`0.0011`, `0.002`, `0.012`, and `1.0` are equivalent enabled values at this
stage. A disabled stage must not parse/re-encode geometry or run a tolerance
zero Boolean repair. It preserves every Task 22H body byte.

## Closed-loop Douglas-Peucker contract

Each input `ExPolygon` is processed independently. Its contour is simplified
first, followed by holes in source order.

For each nonempty polygon:

1. Clone or move its point sequence without rotating it.
2. Append its first point to make an open sequence with equal endpoints.
3. Append the first point to the result. Initialize anchor to index zero,
   floater to the final index, and an endpoint LIFO stack containing only that
   final index.
4. For each candidate range, visit intermediate points from first to last and
   compute squared distance to the finite anchor-floater segment as `f64`.
5. Replace the farthest candidate only for strict `distance > maximum`; equal
   distances retain the first visited candidate.
6. If `maximum <= tolerance_squared`, emit only the range endpoint. Equality
   to the tolerance is removed.
7. Otherwise set floater to the selected index and push that same endpoint.
8. When a range is accepted, emit floater, move anchor to it, pop the matching
   endpoint, and resume with the new stack top until the stack is empty.
9. Remove the final repeated first point after traversal.

Finite-segment distance is exact:

- a zero-length segment returns squared distance to that endpoint;
- subtract every X/Y coordinate pair as `i64` first and only then cast each
  delta to `f64`; casting absolute coordinates before subtraction is forbidden;
- projection parameter `t <= 0` uses the first endpoint;
- `t >= 1` uses the second endpoint;
- otherwise use squared distance to the interior projection;
- compute `tolerance_squared = tolerance * tolerance` in `f64`.

The required large-coordinate KAT uses
`a=(4_000_000_000_000_000_000,0)`,
`b=(4_000_000_000_000_000_001,1)`, and
`p=(4_000_000_000_000_000_000,1)`. Fixed subtract-before-cast finite-segment
distance is exactly `0.5`; cast-before-subtract incorrectly loses the one-unit
X delta. Translated equality and first-tie vectors must remain identical to
their origin-near forms.

The algorithm is intentionally start-point dependent. No cyclic
canonicalization, farthest-pair anchor, recursive rewrite, epsilon comparison,
parallel maximum, or last-tie selection is allowed. A ring may simplify below
three points; the following Clipper input stage drops it. No fallback triangle
or original-ring recovery is added.

## Exact three-union repair

After simplifying one contour and its holes, that one ExPolygon enters a
repair pipeline with two mandatory and one conditional ordered NonZero union:

1. StrictlySimple `execute_paths(Union, NonZero, NonZero)` over the simplified
   contour and holes.
2. Non-strict `execute_paths(Union, NonZero, NonZero)` over pass-one Paths.
3. If pass two is nonempty, non-strict
   `execute_polytree(Union, NonZero, NonZero)` and ordered conversion to
   ExPolygons.

Passes two and three reuse released Ares `union_ex`. Pass one is the new
source-cited `simplify_polygons` wrapper. Every enabled ExPolygon calls pass
one and then `union_ex`, even when Douglas-Peucker removes no point.
`union_ex` always executes pass two, but preserves its released early return:
pass three runs only when pass-two Paths are nonempty.

Different input ExPolygons in one layer are never fed to the same union. One
input may produce zero, one, or several outputs. Append all of its outputs
contiguously before processing the next input. Do not rerun largest-contour,
sort, canonicalize ring starts, or merge siblings after repair.

## StrictlySimple contract

`ClipperOptions` gains `strictly_simple: bool` with default `false`. Every
released non-strict caller must retain identical behavior and bytes.

For the strict pass only:

- collect the X coordinate of every non-horizontal local maximum at the top of
  a scanbeam, in active-edge order, then sort ascending;
- for every horizontal chain, regardless of its current output assignment,
  initialize and advance the maxima cursor; for left-to-right, start at the
  first maximum strictly
  greater than the first bottom X and disable the cursor if it is greater than
  or equal to the final top X; for right-to-left, start at the first reverse
  maximum less than or equal to the first bottom X and disable it if it is less
  than or equal to the final top X;
- before each active-edge range break, consume left-to-right maxima strictly
  less than the crossing X or right-to-left maxima strictly greater than it,
  and insert `(maxima_x, horizontal.bottom.y)` only when the horizontal is
  assigned at that instant and has nonzero winding; a chain that becomes
  assigned during crossings must retain the already-advanced cursor;
- create the upstream type-3 touch join when a promoted/non-maxima top edge
  shares current X with its previous active edge, both outputs are assigned,
  and both winding deltas are nonzero;
- during output fixup, preserve a collinear middle point that lies between its
  neighbours when either `preserve_collinear` or `strictly_simple` is true;
- after common-edge joins and output fixup, repeatedly find exact equal,
  non-successive output points and split the ring;
- append each new output record in discovery order and continue the outer
  index loop over newly appended records; never rerun fixup or orientation on
  a split ring;
- when the new ring is inside the old ring, set its hole state opposite the
  old ring and `first_left` to the old ring; when the old ring is inside the
  new ring, transfer the old hole state/parent to the new ring and make the old
  ring its opposite-state child; when disjoint, copy the old hole state and
  parent to the new ring;
- only during PolyTree execution, update dependent records with
  `fixup_first_lefts2(new, old)`, `fixup_first_lefts2(old, new)`, or
  `fixup_first_lefts1(old, new)` for those three cases respectively;
- never call `fixup_first_lefts3` from `DoSimplePolygons`;
- build Paths and PolyTree in existing output-record order.

No epsilon, hash-set reordering, coordinate sort, post-output canonicalization,
or generic polygon library may replace this state-machine behavior.

## Ares destination boundary

Task 22I remains private, byte-oriented, and platform-neutral:

- `geometry/simplification.rs` owns finite-segment distance, iterative closed
  Douglas-Peucker, and per-ExPolygon simplify orchestration;
- `geometry/clipper/simplify.rs` owns the strict NonZero Paths wrapper;
- `geometry/clipper/strictly_simple.rs` owns strict top-edge/maxima helpers;
- `geometry/clipper/output/simple.rs` owns duplicate-point splitting and
  ownership repair;
- `geometry/clipper/intersections/top.rs` is a real split of the existing
  399-LOC top-of-scanbeam code and hosts its strict call site;
- `geometry/clipper/horizontals.rs`, `engine.rs`, `minima.rs`, and
  `output/fixup.rs` receive only the required state-machine integration;
- `project_slice/simplification.rs` owns Option mapping, scaled tolerance, and
  independent per-ExPolygon traversal;
- `project_slice.rs` carries the already-selected scale through Task 22H,
  invokes Task 22I, and feeds the public incomplete lifecycle;
- `project_slice/task22i_oracle.rs` wraps the released complete encoder with
  `ARES22I\0` magic only after simplification.

The non-default browser feature becomes `task22i-browser-oracle`. It exposes
exactly a post-H input checkpoint and post-I output checkpoint through
`ares-core` and `ares-wasm`. The Task 22H browser feature and exports are
removed without aliases; native Task 22H test helpers remain under `cfg(test)`.
The feature controls visibility only, never algorithms or Options.

## Invariants and errors

Task 22I adds no public error variant. Its project module privately maps
`ClipperError::CoordinateOutOfRange` to
`SliceError::InvalidInput("project simplification polygon coordinate is outside the supported Clipper range")`.
It does not reuse the sibling-private pre-closing or closing mappers and does
not mislabel this stage as either predecessor.
Internal output-ring and ownership states remain trusted invariants and use
private assertions where required.

The stage validates no new internal `None`, empty, or type case. Empty layers
and an empty ExPolygon result are normal geometry outcomes. External 3MF
parsing and typed Option validation remain owned by existing project code.

## Fixed-source oracle protocol

The ignored C++20 probe consumes the complete released `ARES22H` ownership
stream, validates exact EOF, applies the fixed three-union closure to each
ExPolygon, and emits `ARES22I\0`. It never reads a 3MF Option; tracked full-path
tests are responsible for deriving enabled/disabled behavior from complete
3MF bytes.

The approved probe is 398 physical LOC and 13,431 bytes with SHA-256
`63548a15da7fe2beaa39d812a017f82b568ba99f1ac577f69383e05c67231594`.
MSVC 19.44 built it with `/std:c++20 /EHsc /O2 /fp:precise /W4 /WX
/wd4244 /DNDEBUG`; the executable is 389,632 bytes and its SHA-256 is
`2e7c278ccb73e2f640ff9d7239997ab28875848bce93074e3e3dfde940b545d4`.

Its executable KATs assert the DP equality-removal vector, exact
double-division-then-float Normal/LargeBed scaling, and both ordered outputs of
one touching-ring union. The exact ordered input Path is
`[(0,0),(10,0),(10,10),(20,10),(20,20),(10,20),(10,10),(0,10)]`.
Non-strict output is one Path:
`[(10,10),(20,10),(20,20),(10,20),(10,10),(0,10),(0,0),(10,0)]`.
Strict output is two Paths, in order:
`[(20,10),(20,20),(10,20),(10,10)]`, then
`[(0,10),(0,0),(10,0),(10,10)]`. KSR itself does not distinguish the strict
flag, so complete KSR equality cannot replace this exact synthetic KAT in
tracked Rust tests. Five runs of the strengthened executable retain the
approved KSR SHA-256
`0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef`.

The probe, fixed checkout, generated archives, and outputs are ignored evidence
only. Tracked tests encode and parse Ares bytes independently and never invoke
the probe or inspect source paths, line numbers, commits, or hashes.

## KSR acceptance at this boundary

The committed project and reference G-code fixture hashes remain respectively
`698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`
and `10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3`.
The reference G-code hash is integrity evidence only; Task 22I code and tests
do not open that file.

The committed `resolution=0.012` Task 22I stream is 999,721 bytes with SHA-256
`0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef`.
It contains 1 object, 1 volume, 460 Regular layers, 2,890 contours, 395 holes,
and 58,902 points. It removes 40,310 points from Task 22H. Exactly 260 layer
records change, slots 0 through 259. Encode the vector as base-10 ASCII
integers joined by one comma, with no spaces and no trailing comma or newline:
`0,1,2,...,259`. That exact byte string's SHA-256 is
`7377acff6b3bea897ad32249b320eeba2bc48091b9618be54d2f3ad44d269514`.
Five fixed-source runs are byte-identical and reach exact EOF.

Representative output records are:

- slot 0: 11,681 bytes, SHA-256
  `a9320cf7f76a8a4dc24d394033ae1e53b5245eec5d808d8df26a35a5ac49bc9c`;
- slot 46: 24,217 bytes, SHA-256
  `0e515d5ebb34e7f06e886956f62b955cc83a7e58e49f2b28ab37374b26f58291`;
- slot 49: 23,513 bytes, SHA-256
  `c020b4558012a485af5ec1bcc01da9b3785fb448e24e37ee4adcd307deaf0ea8`;
- slot 459: unchanged 737 bytes, SHA-256
  `c8822b67958531cb4b043d338b53f7329e0b00cb4f08108306763e763cd52f80`.

A complete 3MF mutation changing only `resolution: 0.012 -> 0.001` must produce
an exact marker-only H-to-I identity: 1,644,681 bytes, SHA-256
`572688f416497a276540adc57df50742561363a7d0470124ea21759eced591ff`.
A second complete mutation changing only `0.012 -> 0.0011` must match the
committed enabled I output exactly. Both mutations are rebuilt inside tests and
passed through the complete public project preparation path.

The released Task 22H three-Option archive must simplify to 275,433 bytes,
SHA-256
`022cc958a38d5654e0a5fc4e2ca44d5e5ef068b7e57b271cb14151b11005343e`,
with modes `2/0/0/458`, 470 contours, 13 holes, and 16,245 points. This proves
Task 22I also runs after behavior-bearing largest-contour selection. The
threshold-21 output is supplementary regression evidence: 416,217 bytes,
SHA-256
`185118681aad5de780a93d6f71f22f497dc7dc7dd82e038ec1feaf32b0f91294`.

Superseded two-pass outputs and tolerance-zero re-encodes are prohibited as
tracked constants.

## Planned test inventory

Geometry tests cover finite-segment endpoint/interior/degenerate cases,
subtract-before-cast coordinates above `2^53`, Normal/LargeBed scaled-float
values,
iterative stack ordering, exact tolerance equality removal, strict retention,
first-farthest ties, preserved start-point dependence, empty/single/two-point
chains, contour-before-holes ordering, dropped invalid rings, split outputs,
and contiguous per-input append order.

Clipper tests cover default-false byte identity, the fixed strict touching KAT,
type-3 top joins, horizontal maxima insertion in both directions, strict
collinear retention, non-successive duplicate splits, disjoint and contained
ownership, dependent `FirstLeft` repair, repeated splits, output ordering, and
unchanged released non-strict Boolean/offset checkpoints.

Project tests cover exact threshold mapping at `0.001`, all four slicing modes,
multiple objects/volumes/layers, preserved plans and metadata, independent
ExPolygon scope, empty results, fixed-scale propagation, complete committed and
mutated 3MF checkpoints, EOF/counts/representatives, repeatability, unchanged
fixtures, and the public incomplete lifecycle.

Browser tests build fresh default and Task 22I feature bindings, audit exact
exports, run parser and WebCrypto KATs, execute the committed and both
resolution-mutated complete 3MFs, compare H/I ownership streams, and freeze
exact hashes/counts/repeatability in Chromium.

## Included behavior

- Global 3MF `resolution` threshold mapping at this mesh-slice stage.
- Exact closed-loop iterative Douglas-Peucker behavior.
- Exact per-ExPolygon three-union repair.
- Required Clipper StrictlySimple state-machine closure.
- Ordered output and ownership reconstruction.
- Complete native and WASM/browser conformance checkpoints.

## Explicitly deferred behavior

- Raw `resolution` consumers in brim, fill, perimeter, arc fitting, and G-code.
- Orca GUI's global large-bed scale mutation in
  `src/slic3r/GUI/Plater.cpp:11361-11366`; Task 22I reuses the already selected
  Ares `CoordinateScale` and makes no new GUI or global scaling claim.
- Cross-ExPolygon, cross-volume, NegativeVolume, and modifier composition.
- Regions, surfaces, perimeters, fill, supports, toolpaths, G-code assembly,
  metadata, post-processing, and normalized reference-G-code equality.
- Generic upstream simplify APIs not reached by the cited call graph.

## Structural, hardcoding, and platform constraints

- Every Rust production and test file remains below 400 physical LOC; split
  before reaching the limit.
- `intersections.rs` must be genuinely split before new top-edge behavior; no
  line-count compression or macro split substitutes for a module boundary.
- Tests live in separate real `mod` files. `include!` and `include_bytes!` may
  not split Rust source or test modules.
- No unsafe, FFI, filesystem, process, thread, UI, terminal, OpenGL, native
  dependency, platform branch, or second geometry engine enters `ares-core`.
- No production fixture name/hash/count, reference-G-code read, Option literal
  override, coordinate table, stage bypass, or KSR-specific branch is allowed.
- Existing obsolete executable Orca source-pinning tests remain deleted; no
  source-path/line/hash test is added.
- No legacy browser feature alias, fallback simplifier, or compatibility shell
  is retained.
- Tier-1 remains WASM browser, Windows, macOS, and Linux.

## Verification and review exit criteria

Implementation follows strict RED-GREEN-REFACTOR packages. Exact complete
checkpoint assertions and synthetic strict vectors are registered before the
corresponding production behavior. Expected oracle constants cannot change
without new fixed-source evidence and independent approval.

Before release, all focused and predecessor Task 22 tests, workspace nextest,
workspace all-target/all-feature Clippy with warnings denied, rustfmt, native
checks, both wasm32 checks, isolated feature-export audits, and fresh Playwright
Chromium tests must pass. Structural and hardcoding audits must pass on the
exact candidate manifest.

One independent read-only reviewer must assess requirement completeness,
logical correctness, boundary cases, code quality, test coverage, and actual
execution. It returns a repair list to the main thread and makes no edits. The
main thread fixes every finding, reruns affected verification, and sends the
same reviewer the new exact candidate. This loop continues until literal
approval with no unresolved P0-P3 finding.

After six-axis approval, independent specification, quality, default-model,
and documentation reviews must approve the same bytes. The exact commit is
pushed normally and its Tier-1 run must pass all five jobs. Only then may Task
22J begin. Task 22I approval does not claim complete G-code parity or complete
the persistent user goal.
