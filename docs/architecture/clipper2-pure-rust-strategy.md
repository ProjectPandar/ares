# Pure-Rust Clipper2 boundary strategy

Status: **proposed; review required before implementation**. Target: OrcaSlicer
v2.5.0-beta, `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This is a source-rewrite strategy, not a parity or platform PASS claim.

## Boundary and non-negotiable decisions

Replace the Clipper 6.4.2 engine behind `ares-core/src/geometry/clipper.rs`
with an in-tree, safe Rust port of the pinned `deps_src/clipper2/Clipper2Lib`:
`clipper.core.h`, `clipper.engine.h/.cpp`, and `clipper.offset.h/.cpp`.
The dependency identifies itself as **2.0.1** (`clipper.version.h:4`); its pinned
Git tree is `09f1baceae892df95ea07250dbb6b6d3dedd3d2b`.
`src/libslic3r/ClipperUtils.cpp/.hpp` and `ClipperZUtils.cpp:33–64` own the
application wrappers; dependency-version equivalence alone is insufficient.

Keep the existing geometry entry signatures only as temporary adapter shells
around these upstream operations. Replace their engine, offset, tree and Z
implementations together; do not retain a reachable Clipper1 backend, dual-engine
selector, native C++ dependency, or version/platform fallback. Keep implementation
inside `ares-core`, split by upstream responsibility before files exceed 400 lines.
Use indexed arenas for the upstream linked active/output records, retaining their
mutation, scanbeam, intersection and output traversal order. No new sort keys,
ring-start rotation or result canonicalization.

The pure-Rust candidate `larsbrubaker/clipper2-rust`, inspected at
`2f5e9d4249bc4d624773094ff6164335d9ff6ae7`, declares upstream **1.5.4** in
`src/version.rs`; it is not selected as an unchanged dependency. Reference code
reuse requires function-by-function reconciliation to the pinned 2.0.1 sources
and retained BSL-1.0 notices. The similarly named `clipper2`/`clipper2c-sys` FFI
route is excluded. The pinned Orca source, not another port's tests, is authority.

## Numeric contract

Copy scaled XY directly as `i64`; no rescaling through floating-point paths.
Port the declared `±(i64::MAX >> 2)` coordinate domain and upstream boundary error
behavior. Use Rust `i128` products for `ProductsAreEqual`/`CrossProductSign`
(`clipper.core.h:730–801`), preserving exact signs on browser WASM and native
platforms without compiler-specific intrinsics. Do not replace the engine's
floating arithmetic with a purportedly more exact algorithm: areas, slopes,
intersection calculations and offset normals retain upstream `f64` evaluation
order. Preserve each conversion separately: `Point64` construction uses
`std::round` semantics, while default `GetLineIntersectPt` interior results cast
with truncation (`clipper.core.h:117–165,956–979`). No global rounding policy,
FMA substitution, epsilon tuning or saturating/wrapping arithmetic workaround.

## Wrapper policies to port literally

- **Stray holes:** `ClipperUtils.cpp:283–390` classifies by signed area, not tree
  depth alone. Recurse through misplaced contours/islands; collect clockwise
  contours encountered in an outer position. Reassign each stray to the smallest
  enclosing contour with strictly greater area, using the upstream bbox candidate
  pruning and ascending-area search. `path_encloses` ignores vertices within
  **3 scaled units** of the boundary, then uses the first clear vertex; if all
  touch, use the exact `interior_point` band/crossing algorithm
  (`:930–971`). An unresolvable sliver is not enclosed; an unattached hole remains
  the original clockwise contour. Do not reverse or discard it. The tolerance is
  source behavior, not permission to relax a comparator.
- **Collinear points:** ordinary closed/open wrappers explicitly set
  `PreserveCollinear(false)` (`:393–416`), despite Clipper2's different default.
  Z clipping uses its caller's flag. Preserve the engine's unconditional duplicate
  and 180-degree-spike removal and self-intersection repair
  (`clipper.engine.cpp:1551–1586`); retaining collinear points does not retain spikes.
- **Offset setup:** `ClipperUtils.cpp:419–430` uses `float` delta at the wrapper
  boundary. For round joins, positive `miterLimit` is arc tolerance clamped to
  `abs(delta)*0.25`; nonpositive values select **0.25**, without that clamp.
  Otherwise miter limit is at least **2**, and arc tolerance is
  `min(capTolerance > 0 ? capTolerance : 0.25, abs(delta)*0.25)`.
  Port the pinned offset engine's arc subdivision, rounding, orientation and final
  union (`clipper.offset.cpp:262–307,480–510,599–680`), not Clipper1's formulas.
- **Shortest edges and winding:** use `abs(delta*0.005)`
  (`ClipperUtils.hpp:47`, `.cpp:235–264,479–572`). Drop points only at squared
  distance **strictly less** than the threshold from the last retained point;
  zero threshold removes exact duplicates. Trim the closed trailing seam first;
  reject polygon paths with fewer than three retained points. Preserve the strict
  bbox group test, isolated-CW delta reversal, degenerate negative-offset handling,
  fallback union fill rule, and ExPolygon contour/hole winding preparation
  (`:433–525,582–598`). These are beta wrapper behavior, not an old-engine fallback.
- **Z paths:** `ClipperZUtils.cpp:33–64` executes NonZero clipping, distinguishes
  open subjects, and appends closed outputs before open outputs. Port engine
  endpoint-Z precedence (`clipper.engine.cpp:831–868`); the wrapper suppresses the
  user fill callback whenever the intersection XY coincides with any input endpoint.
  Preserve subject-before-clip endpoint priority and callback argument order.
  Otherwise pass the four original endpoints and copy back only the callback's Z.

## Scope and verification gate

Include closed booleans with all wrapper fill rules, PolyTree-to-ExPolygon output,
open clipping, Z clipping, offset joins/caps, safety offsets and sequential offsets
used by existing geometry consumers. Existing higher-level variable-offset and
simplification adapters must call this single engine and retain only behavior
still owned by the beta wrappers. Remove orphaned Clipper1 state/repair/round-policy
machinery rather than emulating its deleted options.

Defer TriangleMeshSlicer canonical ordering/coplanar handling to the separate
subsequent slice. Do not bundle walls, fill, support, configuration, GUI, file I/O,
ClipperD convenience APIs, rectangle clipping or triangulation not needed by this
wrapper boundary. No new public geometry API solely for testing.

After spec approval, first commit a targeted **public RED** witness through an
existing public slicing/path API or CLI parity case, with immutable beta-oracle
bytes or independent literal expectations. Select explicit nested/touching-hole,
collinear/spike, offset-threshold/arc-cap, and coincident/new-intersection Z cases;
new tests contain no private-module calls, mocks, loops or computed expectations.
Source-only diagnostic probes may localize failures but cannot replace the public
RED or claim official application parity. New case/function/fixture names describe
behavior; existing names are not renamed by this slice. Any required instrumented producer must
be separately identified and approved; old producer outputs cannot become beta
expectations by relabeling.

Then port the source boundary, run only the selected RED/GREEN filter,
`cargo fmt --all -- --check` and
`cargo clippy --workspace --all-targets -- -D warnings`, and submit the
implementation REVIEW-REQUEST. Preserve ordered outputs and exact assertions;
unresolved unrelated migrations are reported, not hidden by skips or tolerances.
Browser WASM/native integration and FULL gates belong to the integrator after
cutover. This document authorizes neither a FULL run nor implementation before
strategy review.
