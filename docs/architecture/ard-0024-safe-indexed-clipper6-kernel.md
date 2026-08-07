# ARD-0024: Safe indexed Clipper 6 kernel

## Status

Accepted

## Context

The project-to-G-code parity path now produces ordered closed integer polygons
for every planned layer and has applied the source slicing-mode raw-polygon
policy. The next upstream operation used by
`TriangleMeshSlicer.cpp::make_expolygons` is Clipper 6 polygon combination and
PolyTree ownership. The committed KSR project subsequently requires a
0.049 mm closing operation, but that operation itself depends on the same
Boolean kernel.

OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`
contains a modified bundled Clipper implementation. Its files identify the
algorithm as Clipper 6.4.2, while the retained `CLIPPER_VERSION` macro says
6.2.6. Observable behavior is therefore tied to the fixed commit, tree, and
blob identities rather than to either textual version label alone.

The C++ implementation represents edge, output-ring, join, and tree identity
with raw pointers and mutable linked records. A direct pointer translation
would require unsafe or self-referential Rust. A replacement geometry library,
Clipper2, or a C++ binding would introduce different ordering, fill, range, or
platform behavior and would violate the pure-Rust browser boundary in
ARD-0023.

The fixed Clipper source is distributed under the Boost Software License 1.0
and names Angus Johnson, copyright 2010-2017. Ares does not currently have a
repository-wide license or third-party notice convention. This decision
addresses the directly ported component only and does not infer a license for
the rest of the repository.

The fixed Windows workflow gives its x64 matrix a `windows-latest` default but
selects the self-hosted `orca-win-server` when `vars.SELF_HOSTED` is set. Its
reusable workflow chain invokes `microsoft/setup-msbuild@v3`, but does not pin
one MSVC STL toolset. It therefore establishes the Windows/MSBuild release
path, not the exact library version used by every release build. Separately, a
fixed pre-implementation oracle was compiled with audited MSVC STL 14.44
headers. Its 35 minima and 36 intersection nodes proved that this compatibility
target reorders equal keys after the 32-element insertion-sort threshold and
changes final sibling order. The previously proposed stable source-sequence
normalization is therefore rejected before production implementation.

## Decision

### One exact engine

`ares-core` will own one pure-Rust closed-path Clipper 6 engine. The engine is
the source-cited basis for union, intersection, difference, XOR, offset cleanup,
and later polygon wrappers. There will be no second polygon Boolean engine,
legacy contour fallback, native C++ runtime, FFI, Clipper2 substitution, or
generic geometry-crate approximation.

The first implementation package includes the complete strongly connected
closed-path Boolean state machine: input normalization, local minima,
scanbeam, active and sorted edge lists, winding and contribution, horizontal
processing, intersection ordering, output rings, common-edge joins, ownership
repair, ordered Paths, and PolyTree. Open paths, the optional strictly-simple
post-pass, and the offset engine remain separate packages.

### Stable identity without unsafe

C++ pointer identity becomes typed stable indices into deterministic `Vec`
arenas:

- `EdgeId` for input and active edges;
- `OutRecId` for output records and redirections;
- `OutPointId` for circular output-point lists;
- `PolyNodeId` for tree nodes.

All links are typed IDs or `Option<ID>`. Edge slots removed while normalizing
an otherwise accepted path remain tombstones so later edge identities retain
the fixed allocation sequence; slots from an entirely rejected path are
rolled back. Output-point free-list reuse and allocation order are likewise
preserved because they can affect subsequent ring and tree order. Simultaneous
mutable access to two distinct arena entries uses a small safe `split_at_mut`
helper. The engine contains no raw pointers,
`unsafe`, `Rc<RefCell<_>>`, self-referential structs, hash-order-dependent
stores, mutable globals, or exposed arena identities.

The C++ `OutIdx` sentinels become an exhaustive enum rather than integer
sentinels. Redirected output records retain explicit root identity; they are
not replaced by copied polygons.

### Numeric contract

Coordinates remain signed 64-bit integers. The engine preserves:

- `loRange = 0x3fffffff`;
- `hiRange = 0x3fffffffffffffff`;
- rejection of any retained candidate coordinate outside
  `[-hiRange, hiRange]`, after initial cardinality trimming;
- exact full-range slope-equality signs where the fixed source calls its
  filtered Int128 predicate;
- fixed `double` area and point-in-polygon cross products where the source
  deliberately uses floating arithmetic;
- the fixed floating-point `Dx`, intersection, and `TopX` formulas;
- Clipper's `floor(value + 0.5)` rounding and its
  `0.49999999999999994 -> 0` special case.

Rust `i128` products implement the fixed full-range slope-equality predicate.
A low-range fast path is permitted only if behavioral tests prove it
identical. Predicates implemented with `double` in the fixed source remain
`f64`; they are not silently strengthened and thereby changed at range edges.
Rust's `f64::round()` is not substituted for the fixed rounding rule.
Platform-sized integers are not used for coordinates or arithmetic decisions.

Range validation occurs once in the crate-private polygon-input routine, after
the fixed terminal-point trimming has rejected a closed path with fewer than
three candidates. This ordering is observable: coordinates belonging only to
such an initially degenerate path are ignored, while every retained candidate
coordinate is range checked. Internal engine methods trust normalized arena
state and do not duplicate impossible-state defenses.

### Observable ordering and ownership

All four clipping operations and all four fill rules are implemented in the
closed engine even though the first production consumer exposes only
`union_ex`. The engine preserves fixed-source allocation, traversal,
intersection, output-record, sibling, and point-start order.

The Orca `union_ex` wrapper keeps its overlap workaround:

1. execute union to ordered Paths;
2. if the result is nonempty, execute union again into PolyTree;
3. convert each root contour and its immediate hole children to one
   `ExPolygon`, then recursively emit islands nested inside holes.

No area sort, coordinate canonicalization, ring rotation, unordered set, or
post-hoc containment heuristic may replace that sequence.

Fixed Clipper uses non-stable `std::sort` with comparators that inspect only Y.
Ares makes the resulting Windows dependency explicit by porting the relevant
MSVC STL 14.44 sort control flow into one platform-neutral Rust module:

- insertion sort is used for ranges of at most 32 elements;
- larger ranges use the fixed median guess, three-way partition, recursion
  budget, and heap fallback;
- minima retain the fixed ascending-Y comparator and are consumed from the
  back;
- intersections retain the fixed descending-Y comparator before adjacency
  repair;
- comparator-equivalent values receive no invented coordinate, edge-ID, or
  source-sequence tie breaker.

The implementation is deterministic because every Ares target executes that
same Rust control flow; it does not call the host platform's sort. The audited
compatibility target is pinned by `_MSVC_STL_VERSION=143`,
`_MSVC_STL_UPDATE=202503L`, toolset directory `14.44.35207`, SHA-256
`e4cfb31da8ec07af89834d829ea72b20c7e3202476af3b0641cfe8d6ebb245d7`
for `algorithm`, and SHA-256
`56c6be67b7c0ff9b3ffb7d48943c1ec01728f41f0663dca2c49c296f492bf619`
for `__msvc_heap_algorithms.hpp`. The exact control-flow closure is
`algorithm:233-237,7147-7152,8242-8404` plus
`__msvc_heap_algorithms.hpp:21-136`. Rust preserves the insertion helper's
backward element-move order without copying its unrelated bitcopy
optimization. Large equal-key Windows and KSR oracles must match before
release. This is a compatibility dependency, not a fixture-specific reorder
or platform branch.

Ordering-helper tests cover each source branch independently: insertion at
`<=32`, median-of-three partition at `33..=41`, Tukey ninther at `>=42`, and an
adversarial range that reaches the introsort heap fallback with test-only
branch evidence. The threshold is 42 because `_Guess_median_unchecked`
receives an inclusive last iterator and compares `40 < N - 1`. The minima and
intersection paths may not call any Rust host slice sort API; only this fixed
helper owns their ordering.

Within each project object, Ares uses the released `VolumeOrdinal` as the
portable normalized equivalent of ascending Orca runtime
`ModelVolume::id()`. Numeric 3MF leaf IDs remain provenance only. Absolute
Orca `ObjectID` values depend on process-global allocation history and are not
reproduced.

### Geometry-private provenance extension

Task 22O.28 extends this same indexed engine with optional per-vertex `i64` Z
metadata for pinned ClipperZ operations. Z is provenance, not a third geometry
axis: ordinary point equality and every clipping, cleanup, winding, ownership,
and containment predicate remain XY-only. Existing 2-D adapters create
`z = 0`, existing ordered 2-D outputs discard Z, and the public `Point`,
`Polygon`, and `Polyline` contracts do not change.

The private Z execution path reuses the existing edge, output-point, join,
free-list, and PolyTree arenas. It applies the pinned endpoint-priority `SetZ`
rules, owns one execution-local intersection-label collector, and stores an
optional Z sidecar parallel to a PolyTree contour. The collector is absent for
ordinary clipping and must be absent before and after each Z execution.
`KernelPoint`, Z paths, sidecars, and Z adapters are visible only inside the
geometry module; they are not a second clipping engine or a public 3-D API.

Region-expansion seed discovery may widen the fixed MSVC-sort helper only to
geometry-private visibility and sorts a `Vec<usize>` permutation. The accepted
MSVC STL 14.44 control flow, threshold, comparators, and lack of tie-breakers
remain unchanged.

### Platform and license boundary

The implementation stays filesystem-free and platform-neutral in `ares-core`
and must compile on Windows, macOS, Linux, and `wasm32-unknown-unknown`. It may
not introduce native allocators, native threads, TBB, or platform branches.

The repository will carry the unmodified BSL-1.0 text, the Apache-2.0 WITH
LLVM-exception text applicable to the audited MSVC STL sort control flow, and
a concise third-party notice for both directly rewritten components. The
Clipper and fixed-sort module roots retain source and license provenance. This
does not set the license of the whole crate and is not a legal conclusion
about unrelated Orca-derived work.

## Consequences

- The implementation is larger than a convex, rectangle, or union-only
  algorithm because correctness for touching, coincident, nested, and
  self-touching paths crosses the whole state machine.
- Safe indices make ownership explicit and portable, at the cost of more
  verbose arena mutation than the C++ pointer representation.
- Determinism is a first-class compatibility contract; geometrically equal but
  differently ordered output is a failure.
- Exact Windows ordering requires a small source-cited Rust rewrite of the
  release toolchain's sort control flow; relying on Rust, libc++, or libstdc++
  unstable sort would reproduce a different hidden dependency.
- The first production stage is explicitly named pre-closing. It is not the
  final `slice_mesh_ex` result for KSR because closing, largest-contour
  selection, and simplification follow in later source-cited packages.
- Offset and later Boolean consumers must extend and reuse this kernel rather
  than introduce another engine.
- Optional Z provenance increases internal record width during geometry-private
  ClipperZ execution, but does not alter ordinary 2-D predicates, ordering, or
  public geometry semantics.

## Rejected alternatives

- Bind or invoke Orca/Clipper C++ | Breaks WASM and does not implement Ares.
- Use Clipper2 or a generic polygon crate | Fill, rounding, range, and output
  order are not the fixed Clipper 6 contract.
- Publish a rectangle, convex, nonintersecting, or containment-only union |
  Cannot cover the strongly connected edge/join/ownership behavior.
- Use raw pointers or `unsafe` to mimic C++ | Unnecessary; typed stable arenas
  preserve identity safely.
- Use `Rc<RefCell<_>>` graph nodes | Adds runtime borrow and allocation costs
  and obscures deterministic free-list identity.
- Sort or canonicalize final polygons | Changes observable point, contour, and
  sibling order needed by downstream exact G-code parity.
- Sort volumes by the numeric 3MF leaf ID | That field is not Orca's runtime
  `ModelVolume::id()` and released tests deliberately distinguish them.
