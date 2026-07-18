# Task 22F: Clipper 6 Closed Boolean, PolyTree, and Pre-Closing Union

## Status and objective

This specification is a draft. Production or test implementation may begin
only after independent fixed-source/spec, independent Ares/plan, and direct
default-model reviewers approve the exact frozen specification, plan, and
ARD-0024 bytes.

Task 22F is the next bounded source-rewrite package in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Task 22E commit
`645f5cb9e193750b8ffdbdf6e06e8829c7c210f4` produces, for every object,
volume, and planned layer, ordered closed integer polygons plus the original
internal slicing mode. Task 22F ports the complete closed-path Clipper 6
Boolean/PolyTree kernel required by Orca's `union_ex` wrapper, defines owned
`ExPolygon` output, and connects the exact fill-rule-sensitive result as an
explicit per-volume, per-layer pre-closing project stage.

The committed KSR project has `slice_closing_radius=0.049` and
`resolution=0.012`. Therefore Task 22F output is deliberately not the final
`slice_mesh_ex` output. It precedes the required expand/shrink closing,
post-union largest-contour selection, and simplification packages. The public
project API still traverses the owned intermediate result and returns
`SliceError::ProjectSlicingIncomplete`.

No implementation may recognize the fixture, read reference G-code, invoke
Orca, substitute a different polygon engine, or infer missing behavior from
the expected output.

## Fixed upstream identity

All citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored Orca checkout may
have another HEAD. Source evidence is read with `git show <sha>:<path>` without
changing that checkout.

Primary fixed blobs:

- `deps_src/clipper/clipper.hpp`, blob
  `06637effce040fa7d87c368437cb32398f19ee92`;
- `deps_src/clipper/clipper.cpp`, blob
  `1f16446ac8da1f0b9c802d8a9dee33f766919f6b`;
- `src/libslic3r/ClipperUtils.hpp`, blob
  `9c2fa239263c0cb097a4b4c3db823821615bd7c7`;
- `src/libslic3r/ClipperUtils.cpp`, blob
  `2f97e08f536e93c5fd27b4614980072285d2ce22`;
- `src/libslic3r/TriangleMeshSlicer.cpp`, blob
  `2c1c0da23fe569c93b5d243a14494792956533d0`;
- `src/libslic3r/ExPolygon.cpp`, blob
  `185e92508449a425064b26690e3d74d06a16fda8`;
- `src/libslic3r/Model.hpp`, blob
  `d8697adb41307ac2cdb018c440f1afac75f01356`;
- `src/libslic3r/ObjectID.hpp`, fixed commit path;
- `src/libslic3r/Int128.hpp`, fixed commit path.

The fixed Windows workflow path is recorded because equal-key ordering is
observable, but the workflow does not itself pin one MSVC STL toolset:

- `.github/workflows/build_all.yml`, blob
  `4792c59fb746b24509f7cf548b29f041a84b32be`, gives x64 a
  `windows-latest` default and selects `orca-win-server` when
  `vars.SELF_HOSTED` is set;
- `.github/workflows/build_check_cache.yml`, blob
  `646a02dbf0cf9cf7eb2b6fb142c42f26b5545ea6`, forwards the chosen runner to
  the dependency workflow;
- `.github/workflows/build_deps.yml`, blob
  `8b476c9290ac16841fff0c34415596789cc5d1e3`, invokes
  `microsoft/setup-msbuild@v3` and then the Orca build workflow;
- `.github/workflows/build_orca.yml`, blob
  `0bc2c37a20420bd5e7eee01693c4ebad990708fe`, also invokes
  `microsoft/setup-msbuild@v3` before the Windows build;
- those workflows establish the Windows/MSBuild path but do not select the
  exact `14.44` toolset below;
- the separately audited fixed Windows oracle used MSVC STL toolset
  `14.44.35207`,
  `_MSVC_STL_VERSION=143`, `_MSVC_STL_UPDATE=202503L`;
- its `algorithm` header is SHA-256
  `e4cfb31da8ec07af89834d829ea72b20c7e3202476af3b0641cfe8d6ebb245d7`;
- its `__msvc_heap_algorithms.hpp` is SHA-256
  `56c6be67b7c0ff9b3ffb7d48943c1ec01728f41f0663dca2c49c296f492bf619`.

Those standard-library headers are Apache-2.0 WITH LLVM-exception. They are
used as a source-cited compatibility boundary for the pure-Rust sort-control
rewrite only; no MSVC code or runtime is linked into Ares.

The bundled Clipper file header says version 6.4.2 while its version macro says
6.2.6. This package is pinned to the fixed source identity above, not to a
floating Clipper version claim.

## Exact upstream rewrite boundary

The closed Boolean dependency closure is:

- `clipper.hpp:75-81,88-100,121-123,137,141-223,225-535` for clipping and
  fill enums, coordinate ranges, path/tree vocabulary, edge/output records,
  and the closed Clipper interface;
- `clipper.cpp:67-72,78-161,167-426` for direction/sentinels, the horizontal
  marker, rounding, area, point-in-polygon, exact slopes, intersection points,
  and linked-ring helpers;
- `clipper.cpp:429-1614` for closed-path normalization, edge construction,
  local-minima/LML construction, reset, execute, winding, contribution,
  output creation, and AEL insertion;
- `clipper.cpp:1630-3340` for closed-edge intersection, output append,
  horizontal handling, SEL/AEL ordering, top-of-scanbeam processing, output
  cleanup, common-edge joins, ownership repair, ordered Paths, and PolyTree;
- `Int128.hpp:234-277` for the full-range slope-equality sign contract,
  expressed in Rust with exact `i128` products;
- `ClipperUtils.cpp:303-350` for direct execution and union wrappers;
- `ClipperUtils.cpp:634-668` for the two-pass Paths-then-PolyTree overlap
  workaround;
- `ClipperUtils.cpp:169-204` for exact PolyTree-to-ExPolygon ownership and
  order;
- `ClipperUtils.cpp:737-740,812-814` for `_clipper_ex` and
  `union_ex(Polygons, fill_type)`;
- `ClipperUtils.hpp:42-155` only where it fixes closed path provider expansion
  and input order;
- `TriangleMeshSlicer.cpp:1738-1823,2003-2049` only through the initial
  `union_ex` result before offset, largest-contour, or simplify behavior;
- `PrintObjectSlice.cpp:166-177` as deferred-consumer context, including the
  configured `resolution > 0.001` to fixed `0.0025 mm` simplification mapping;
- `Model.hpp:1227-1230`, `ObjectID.hpp:20-87`, and the released Task 22B
  import/creation-order boundary for portable volume ordering.
- fixed Windows workflow blobs above for the runner/MSBuild call chain, plus
  MSVC STL `algorithm:233-237,7147-7152,8242-8404` and
  `__msvc_heap_algorithms.hpp:21-136` for the insertion threshold,
  insertion and backward-move order, median partition, introsort budget, heap
  construction, pop/push, and heap-sort fallback used by the Y-only Clipper
  comparators. The Rust rewrite preserves backward element-move order but not
  unrelated iterator/debug/bitcopy optimizations.

Open-path branches inside overlapping source functions are excluded. Offset,
cleaning, simplification, and project combination after pre-closing union are
excluded and listed explicitly below.

## Architectural contract

ARD-0024 is normative for this package. The implementation uses safe typed
arena indices, exact fixed-range predicates, deterministic vectors, and one
pure-Rust kernel. It contains no C++ binding, native allocator, unsafe block,
raw-pointer graph, `Rc<RefCell<_>>`, unordered output store, or alternate
polygon engine.

The same pure-Rust kernel owns one generic fixed-order helper that rewrites the
audited MSVC sort control flow. Minima and intersection nodes call that helper
with their fixed Y-only comparators. Those paths may not call any host slice
sorting API: `.sort`, `.sort_by`, `.sort_by_key`, `.sort_unstable`,
`.sort_unstable_by`, and `.sort_unstable_by_key` are all forbidden. No
platform branch, coordinate tie-breaker, edge-ID tie-breaker, or
source-sequence tie-breaker is permitted.

The fixed Clipper source is BSL-1.0 and the audited MSVC STL sort control flow
is Apache-2.0 WITH LLVM-exception. The package carries both unmodified license
texts, a concise third-party notice, and module-root provenance. It does not
set or infer a repository-wide license.

## Closed polygon domain

### `Polygon` and `ExPolygon`

The released `geometry::Polygon` remains an ordered `Vec<Point>` with no
duplicated closing point required by its domain. Task 22F may consume its point
vector but may not rotate, sort, deduplicate, or normalize it before the
Clipper input routine applies the fixed rules.

The new private domain type is:

```text
ExPolygon
  contour: Polygon
  holes: Vec<Polygon>
```

Contour, hole, and point order are observable. An ExPolygon is not equal to an
unordered set of rings. Root contours must be positive-area/CCW under the
fixed output convention and holes negative-area/CW. Nested islands become new
ExPolygons; they are not inserted into their grandparent's hole vector.

### Operations and fill rules

The closed engine implements exhaustively:

```text
ClipOperation = Intersection | Union | Difference | Xor
FillRule      = EvenOdd | NonZero | Positive | Negative
```

It accepts independent subject and clip fill rules. Task 22F production uses
subject-only Union, but all closed operation branches are part of the minimum
state-machine port and must have behavioral tests. No public or project-facing
general polygon API is added.

The fixed options are represented explicitly:

- `reverse_solution`, default false;
- `preserve_collinear`, default false.

Task 22F project union uses both defaults. Tests cover both values of each
included option. Open paths and the default-false StrictlySimple post-pass are
not accepted or implemented in this package.

## Input normalization and range semantics

For each closed path, the engine preserves the fixed sequence:

1. remove repeated terminal points equal to the first point;
2. remove repeated consecutive terminal points;
3. reject fewer than three remaining points;
4. validate each retained coordinate against `hiRange` while determining
   whether full-range predicates are needed;
5. build the circular edge sequence;
6. remove adjacent duplicate vertices;
7. remove eligible collinear intermediate vertices unless
   `preserve_collinear` retains the between-point case;
8. reject flat paths and any path reduced below three effective vertices;
9. form local minima, left/right bounds, horizontal direction, and LML links in
   fixed traversal order.

The safe arena preserves the fixed allocation effects: vertices removed from
an otherwise accepted path leave typed edge tombstones, while all edge slots
for an entirely rejected path are rolled back before the next path. Once any
step-4 candidate crosses `loRange`, full-range predicate mode remains enabled
for the request even if that path is later rejected as flat or collinear.

Degenerate paths are ignored without discarding valid sibling paths. The fixed
initial cardinality rejection precedes range validation: an out-of-range point
that belongs only to a path reduced below three candidates by steps 1-3 is
therefore ignored. Any coordinate retained for step 4 outside
`[-0x3fffffffffffffff, 0x3fffffffffffffff]` produces one deterministic geometry
error that the project adapter maps to `SliceError::InvalidInput`; it never
panics, wraps, clamps, or falls back. Tests distinguish a two-candidate path
containing `i64::MAX`, which is ignored, from a three-candidate path containing
the same coordinate, which errors.

After successful input addition, fixed `ExecuteInternal` reports a Boolean
success value. The fixed ClipperUtils wrappers ignore `false`, so execution
failure yields the same empty Paths/PolyTree result as a successful empty
operation after output state is disposed. Ares preserves that observable
wrapper behavior. It does not invent a second project error for a valid
normalized request, and it does not fall back to raw polygons or another
engine. Out-of-range input remains the distinct pre-execution error above.

Exact `i128` determinant products are used for full-range slope equality,
matching the fixed `Int128` call. Fixed Clipper area and point-in-polygon
cross products use `double` in the 64-bit configuration and remain `f64`;
they are not replaced with a mathematically stronger predicate. Input and
output coordinates remain `i64`.

## Scanbeam and edge-state semantics

The engine ports the complete closed state transition:

```text
normalized edges and local minima
  -> descending scanbeam
  -> local minima inserted into AEL
  -> winding counts and contribution
  -> horizontals and ghost joins
  -> intersection list built through SEL
  -> adjacency-correct intersection order
  -> top-of-scanbeam maxima and edge promotion
  -> output-ring cleanup and joins
```

Edge and output identity are stable typed indices. Mutation helpers may change
Rust ownership mechanics but not allocation, free-list reuse, adjacency,
redirection, or traversal order. AEL and SEL swaps must update every involved
neighbor exactly once. Output records retain redirect/root identity after
append or split.

Horizontal minima/maxima, coincident edges, partial collinearity, touching
vertices, shared edges, multiple intersections in one scan band, and
self-touching output are not special-cased after the fact; they flow through
the fixed state machine.

The fixed intersection rounding is:

```text
FRound(0.49999999999999994) = 0
FRound(x)                    = floor(x + 0.5)
```

Rust `round()` is observably different for negative half values and is
forbidden here.

### Equal-key fixed Windows sort

The fixed source uses non-stable `std::sort` with Y-only comparators for local
minima and intersection nodes. C++ leaves comparator-equivalent order
unspecified. The initial proposal added source construction sequence as a
stable secondary key. Before production implementation, a fixed Windows
oracle with 35 minima and 36 intersection nodes crossed MSVC's 32-element
insertion threshold and disproved that proposal:

- the 33 equal-Y minima within the 35-minima vector were reordered from creation
  order; destructive
  `back()` consumption changed complete sibling Paths order;
- the 36-node intersection vector contained a mixed primary-Y pair and a
  34-node equal-Y subgroup; fixed sorting reordered both groups before the
  adjacency pass;
- provider order, X order, and EdgeId order were deliberately shuffled, so
  none is a valid substitute tie-breaker.

Ares therefore ports the audited MSVC STL 14.44 control flow exactly:

1. ranges of at most 32 elements use its move-based insertion sort;
2. larger ranges use its median-of-three or Tukey-ninther guess and
   three-way partition;
3. its `ideal = (ideal >> 1) + (ideal >> 2)` recursion budget and heap fallback
   are preserved;
4. minima use only the fixed ascending-Y comparator and are consumed from the
   back;
5. intersections use only the fixed descending-Y comparator, then run the
   unchanged adjacency repair;
6. comparator-equivalent items receive no additional key.

One Rust implementation of that control flow runs on every Tier-1 target, so
Windows, macOS, Linux, and WASM receive the same fixed Windows order without a
native dependency or platform branch. Complete large-vector Paths and the KSR
pre-closing stream must match the fixed Windows oracle. A disagreement stops
the package for source/toolchain tracing; production does not gain a
fixture-specific reorder.

## Winding and contribution

Winding delta, subject winding count, and alternate winding count preserve the
fixed operation/fill-rule branches. EvenOdd toggles parity. NonZero uses a
nonzero signed count. Positive and Negative test signed count direction.

Contribution is derived during the sweep; it may not be reproduced later with
point containment, area sign, bounding boxes, or ring count. The same input
must distinguish:

- duplicated same-winding rings under EvenOdd versus NonZero;
- opposite-winding nested rings under NonZero/Positive/Negative;
- overlap under Union versus XOR;
- subject/clip overlap under Intersection and Difference.

## Output rings, joins, and ownership

The port preserves:

- local-minimum and local-maximum output creation;
- edge-side insertion at the front or back of circular rings;
- duplicate output-point suppression;
- OutRec append/redirection semantics;
- ghost and common-edge joins;
- horizontal and nonhorizontal join decisions;
- containment and `FirstLeft` repair after split/merge;
- removal of duplicate and collinear output points;
- fixed orientation reversal based on hole/output policy;
- ordered Paths traversal;
- PolyTree parent and child insertion order.

No global canonicalization occurs. A ring's start point is whichever point the
fixed BuildResult traversal emits.

## Exact `union_ex` two-pass wrapper

The production `union_ex(polygons, fill_rule)` wrapper performs exactly:

1. add the polygons as closed subject paths in their existing order;
2. execute Union to ordered Paths with the same fill rule for subject and
   clip;
3. return an empty ExPolygon vector if the first pass is empty;
4. otherwise create a fresh engine, add the first-pass Paths as closed subject
   paths in their emitted order, and execute Union to PolyTree;
5. convert PolyTree to ExPolygons in child order.

The first pass is not an optimization that tests may omit. It is Orca's
overlap-performance workaround and affects the exact ownership/output seam.

PolyTree conversion visits each root child in order. One root contour becomes
one ExPolygon. Its immediate children become holes in child order. Each child
of a hole is recursively emitted as a later ExPolygon, with the same rule
applied at every depth.

## Project pre-closing stage

### Portable volume ordering

Orca sorts each object's selected `ModelVolume*` values by ascending runtime
`ObjectBase::id()`. Those IDs are strictly positive and monotonically allocated
but include unrelated process allocation history. They are not the numeric 3MF
leaf ID.

Released Task 22B therefore established `VolumeOrdinal`: a one-based nonempty
BFS occurrence ordinal assigned before type filtering, with support
blocker/enforcer gaps retained. Task 22F explicitly sorts the Task 22E volume
states by this ordinal before union. It does not sort by `ProjectVolume::id()`,
`source_volume_index`, volume type, geometry, area, or content hash. Duplicate
ordinals are an internal invariant violation, not a new external fallback.

### Fill-rule projection

Each Task 22E `SlicingModeLayer` maps exhaustively:

```text
Regular                -> NonZero
EvenOdd                -> EvenOdd
Positive               -> NonZero
PositiveLargestContour -> Positive
```

`Positive` represents external CloseHoles only after Task 22E has made every
raw loop CCW; it still uses NonZero at this stage. PositiveLargestContour uses
Positive for union but retains its original mode for the later post-union
largest selection.

### Owned result

The new stage retains:

- planned object identity and all planned layer slots;
- source volume index, volume ordinal, and volume type;
- original slicing mode per layer;
- ordered `Vec<ExPolygon>` per layer, including empty vectors.

No empty layer slot is deleted. No volume is combined with another volume.
Negative and modifier volumes receive their independent pre-closing union just
like model parts; their later cross-volume Boolean semantics remain deferred.

The stage is named `PreClosing*` in code and tests. It may not be named final,
combined, sliced, or completed ExPolygon output.

## Normative behavioral vectors

Committed Rust tests freeze complete ordered coordinates, not only area/count.
The minimum vectors are:

1. Empty, one-point, two-point, flat, repeated-endpoint, consecutive-duplicate,
   collinear, and mixed valid/invalid input.
2. Fixed Orca overlapping/nested square union from
   `test_clipper_utils.cpp:194-203`, with exact output
   `[(40,40),(0,40),(0,0),(40,0)]`.
3. One outer CCW square with same-winding and opposite-winding inner squares,
   exercised under all four fill rules.
4. The same nonconvex subject/clip pair under all four ClipOperations, with
   exact ordered Paths generated once by a fixed-source oracle.
5. Duplicate identical rings, reversed identical rings, partial shared edges,
   a complete shared edge, a T-junction, vertex-only contact, horizontal
   minima/maxima, and multiple crossings within one scan band.
6. Contour -> hole -> island -> hole nesting, asserting complete node parent,
   child, sibling, contour, hole, and recursive ExPolygon order.
7. PreserveCollinear false/true and ReverseSolution false/true on
   branch-distinguishing paths.
8. Coordinates at `loRange`, just beyond it, at positive/negative `hiRange`,
   and one unit outside each allowed bound; the out-of-range cases distinguish
   an initially two-candidate path (ignored before range checking) from an
   initially three-candidate path (error). A near-collinear full-range case
   proves exact slope-equality determinant signs, while a containment case
   freezes the source's floating point-in-polygon behavior. An accepted path
   with removed vertices followed by another path freezes tombstone identity,
   and a rejected full-range flat path followed by a low-range path proves that
   full-range mode is monotonic.
9. Repeated execution proves deterministic bytes and no leaked state between
   the first Paths pass and fresh second PolyTree engine. A single fixed
   engine consumes its minima: a second Execute without re-adding input is
   empty, while Clear plus re-add reproduces the first output.
10. A shuffled 35-path minima vector crosses the MSVC insertion threshold and
    freezes complete sibling order. A shuffled 36-node intersection vector
    contains mixed primary Y values plus 34 equal-Y nodes and freezes complete
    output and the pre-adjacency permutation.
11. Reviewed ordering-helper vectors separately freeze an `>=42`
    Tukey-ninther permutation with test-only branch evidence and an adversarial
    public-sort input that exhausts the
    introsort budget. Test-only branch evidence must prove that the latter
    reaches heap fallback; expected permutations come from the fixed MSVC
    oracle, never from the Rust implementation.
12. Project volumes supplied in shuffled ordinal order `[5,2,3]` emit
    `[2,3,5]`; deliberately unrelated leaf IDs/source indices prove they do not
    select the order.
13. Every project slicing mode maps to the stated fill rule, including
    PositiveLargestContour retaining all pre-closing ExPolygons.

Oracle generation is a development action only. Complete ordered expected
values are frozen as Rust literals. Executable tests may not open, parse, grep,
hash, compile, or invoke Orca source or binaries.

## Canonical pre-closing oracle encoding

The real-fixture digest uses SHA-256 over one versioned little-endian byte
stream. No serde format, platform `usize`, debug text, native struct layout, or
map iteration participates.

The byte stream is exactly:

```text
8 bytes  ASCII "ARES22F\0"
u64      print-object count
for each print object in production order:
  u64    source_object_index
  u64    transform_index
  u64    planned layer count
  u64    volume count
  for each volume in normalized VolumeOrdinal order:
    u64  source_volume_index
    u32  volume_ordinal
    u8   volume_type: ModelPart=0, NegativeVolume=1, ParameterModifier=2
    u64  layer count
    for each layer slot in ascending zero-based index:
      u64 layer_slot_index
      u8  mode: Regular=0, EvenOdd=1, Positive=2,
                PositiveLargestContour=3
      u64 ExPolygon count
      for each ExPolygon in emitted order:
        polygon contour
        u64 hole count
        polygon for each hole in emitted order

polygon:
  u64 point count
  for each point in emitted order:
    i64 x
    i64 y
```

Every integer is two's-complement little-endian at the stated width. Counts
frame empty objects, volumes, layers, ExPolygon lists, holes, and polygons;
there are no separators or trailing bytes. `usize` values are checked into
`u64` in test code. Support blocker/enforcer tags cannot occur because they are
filtered before raw slicing. The C++ oracle writer and Rust test encoder are
independently implemented from this table, then compared on hand-written
nested/empty vectors before either processes KSR.

## KSR acceptance at this boundary

Before project production wiring is implemented, a one-time ignored probe
against the fixed Clipper source must freeze the complete KSR pre-closing
oracle. The oracle input is the already frozen Task 22E ordered raw polygon
stream, not the reference G-code. The committed test stores only reviewed
facts or a deterministic stage digest.

The KSR acceptance must prove:

- one print object and one nonempty model-part volume;
- 460 layer slots retained in order;
- every layer maps Regular -> NonZero;
- complete contour/hole/point totals and an ordered binary encoding digest
  match the fixed-source oracle;
- representative first, first-with-hole, maximum-loop-count, and final layers
  match exact ordered coordinates or per-layer digests;
- Task 22E raw polygon/config hashes and both fixture files remain unchanged;
- the public result remains `ProjectSlicingIncomplete`.

This oracle is explicitly pre-closing. It must not be compared with or named
the final KSR `slice_mesh_ex` result.

## Included behavior

Task 22F includes only:

- safe typed-index closed Clipper 6 Boolean/PolyTree state;
- fixed MSVC STL 14.44 equal-key sort-control rewrite shared by minima and
  intersections;
- all closed ClipOperations, fill rules, and fixed options;
- exact range, predicate, rounding, winding, intersection, join, output, and
  tree behavior;
- ordered Paths and PolyTree outputs;
- two-pass `union_ex` and PolyTree-to-ExPolygon conversion;
- `ExPolygon` ownership;
- explicit ascending `VolumeOrdinal` normalization;
- slicing-mode-to-fill-rule projection;
- per-volume/per-layer pre-closing project state;
- synthetic and real KSR pre-closing behavioral acceptance;
- BSL-1.0 and Apache-2.0 WITH LLVM-exception license/attribution artifacts.

## Explicitly deferred behavior

The following are not Task 22F behavior:

- open paths and PolyTree open nodes;
- the default-false StrictlySimple post-pass;
- ClipperOffset, join/end types for offset, shortest-edge removal, and negative
  offset outer-wrapper cleanup;
- `offset_ex`, `offset2_ex`, safety offset, opening, and closing wrappers;
- consumption of `slice_closing_radius`, extra XY offset, or
  `xy_contour_compensation` in the project stage;
- `keep_largest_contour_only` after PositiveLargestContour union;
- Douglas-Peucker/ExPolygon simplification and resolution-threshold mapping;
- cross-volume negative subtraction, modifier intersection, multipart union,
  or region merge;
- layer ranges, painted/MM segmentation, slab slicing, regions, surfaces,
  perimeters, infill, supports, path ordering, G-code, metadata,
  post-processing, and normalized golden parity.

The immediately following package starts at ClipperOffset and
`ClipperUtils.cpp` offset/offset2 wrappers, then consumes the KSR
`slice_closing_radius=0.049`. It must reuse this engine.

## Structural and platform constraints

- Every Rust source and Rust test file remains below 400 physical lines; split
  with real `mod` files before reaching 400.
- Tests live in separate test modules.
- No `include!`, `include_bytes!`, or equivalent macro may split Rust source.
- The new path is platform-neutral, WASM-safe, and contains no filesystem,
  terminal/UI, native thread, Rayon, TBB, C/C++ runtime, platform branch,
  unsafe code, mutable global, or native polygon dependency.
- No legacy fallback, feature flag, compatibility shim, geometry
  canonicalization, fixture branch, reference-G-code read, or executable
  Orca source-pinning test is introduced.
- Production changes remain limited to the approved manifest.

## Verification and review exit criteria

Task 22F is implemented only when:

1. genuine RED/GREEN evidence exists for domain/range/input, sweep/winding,
   intersections/horizontals, rings/joins/ownership, PolyTree/two-pass union,
   project ordering/fill mapping, and KSR acceptance;
2. every new test is named `task22f_*` and passes under Cargo Nextest;
3. all four closed operations, all four fill rules, and both included option
   toggles have branch-distinguishing ordered-coordinate tests;
4. the fixed-source KSR pre-closing oracle matches without reading Orca at
   test runtime;
5. Task 22A-E suites and mesh/project regressions remain green;
6. full workspace Nextest, rustfmt, warning-denying Clippy, native checks,
   core/adapter WASM checks, release WASM build, and real-3MF browser gate pass;
7. audits prove every Rust file is below 400 LOC and no prohibited mechanism,
   hardcoding, pinning, fixture mutation, or manifest drift exists;
8. one independent reviewer validates requirement completeness, logic,
   boundary cases, code quality, test coverage, and actual execution; the main
   thread fixes its list and the same reviewer rechecks until all six pass;
9. fresh whole-spec, whole-quality, and direct default-model reviews approve
   the exact candidate;
10. architecture and roadmap documentation record actual behavior and the
    next offset/closing boundary;
11. one conventional commit is pushed normally and its exact SHA passes all
    five Tier-1 jobs.

Passing Task 22F is not completion of the original user-visible goal. The goal
remains active until Ares emits the complete normalized reference G-code.

**Status: DRAFT — implementation is forbidden until fresh independent and
default-model reviewers approve the exact specification, plan, and ARD bytes.**
