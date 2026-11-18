# B-T isolated RED: failure modes before tests

Failure-mode inventory written before any B-T test/oracle/production code.
This geometry sub-slice uses an isolated RED; public S45 end-to-end output
proof belongs to the later Support preparation slice. No production
implementation here.

## Source supplier audit

Tesselate.cpp:5 includes <glu-libtess.h>, NOT system GLU.
OrcaSlicer/src/libslic3r/CMakeLists.txt:594 links glu-libtess.
OrcaSlicer/deps_src/glu-libtess/CMakeLists.txt:5–34 builds the vendored
C kernel statically (only math on Unix). Its README:6 explicitly identifies
this standalone library as having no OpenGL dependencies.
include/glu-libtess.h:36–49 maps entry symbols to mglu*.
The Native wrapper remains libslic3r/Tesselate.cpp:23–56/100–170/192–195;
its default is NORMALS_UP=false (Tesselate.hpp:13–16).

The owning algorithm is now available locally and source-citable:
- tess.c:415–457 vertex insertion and :500–579 polygon end pipeline;
- normal.c::__gl_projectPolygon determines projection/orientation;
- sweep.c::__gl_computeInterior:1301 and its ordered sweep events;
- mesh.c topology operations and priorityq*.c event order;
- tessmono.c::__gl_meshTessellateInterior:133 monotone triangulation;
- render.c::__gl_renderCache/__gl_renderMesh chooses ordered triangle,
  fan/strip emission; Tesselate.cpp callbacks expand those to Vec3d triples.
Retain SGI Free Software License B notices in any later Rust algorithm port.
No alternate earclip, triangulator crate, TS, native FFI/runtime library,
fixture branch or guessed emission reorder is permitted for GREEN.

## Rust seam and compile RED

Ares geometry is crate-private in geometry.rs. As isolated tests, B-T tests
live under geometry/tessellation/tests.rs;
geometry/tessellation.rs contains test registration only in RED, and
geometry.rs registers the module. No new public API is needed for this RED.
The desired private source operation is triangulate_expolygon_3d:
ExPolygon + z + flip + existing CoordinateScale -> ordered Vec<[f64;3]>.
CoordinateScale is Ares's temporary coordinate representation adapter;
Normal reproduces upstream unscale<double>. No scale-policy change here.

The first isolated Rust RED is compilation failure for the missing operation,
explicitly NOT a Native-runtime failure or parity PASS. Native source/oracle
admission is collected independently; expectations are generated at test time. No stub,
empty result, Unsupported guard, ignore/cfg skip or normalized tolerance is
introduced to fabricate a runtime failure. RED cannot merge without GREEN.

## Runtime Native oracle (replaces a rejected frozen-order oracle)

Every enabled test compiles tests/parity/glu-tessellation/native_oracle.cpp
against the original OrcaSlicer/deps_src/glu-libtess sources at runtime.
The ten C translation units match its CMakeLists.txt; priorityq-heap.c is
included by priorityq.c and is not compiled twice. No prebuilt oracle or
committed Native index orders/coordinates serve as expected values.
Authored input JSON, generator and failure-mode inventory remain committed.

The test-time oracle uses the shared Native availability policy. Optional
unconfigured runs emit an explicit skip, never a parity claim. With
`ARES_REQUIRE_ORCA=1`, missing Native binary, resources, or artifact root
fail the test. Once configured, missing vendor sources or compiler failures
also fail the test.

Each oracle gets a fresh directory below ARES_PARITY_ARTIFACT_ROOT. Source
hashes, compiler commands/status/stdout/stderr, submitted input, Native raw
output and Rust coordinate bits are retained there. Native emits runtime
IEEE-754 uint64 coordinate bits, preserving ordered triples exactly without
decimal-parser rounding, sorting, tolerance or coordinate reconstruction.
The separate oracle-admission test builds the same helper and runs all authored
inputs, proving test-time compilation independently of the missing Rust seam;
it is Native admission evidence, not a Rust parity PASS.
Native S45 observations remain provenance/hand-off, not a B-T slice PASS.

## Failure modes and planned Native-backed coverage

| ID | Failure the source port could have | Isolated check |
|---|---|---|
| F1 | Rectangle diagonal or callback order differs | Convex quad complete ordered triples |
| F2 | Coordinate axes or contour start changed | Translated/asymmetric quad, preserve x/y order |
| F3 | Auto normal treats clockwise input differently | Reverse contour; compare live Native order |
| F4 | Concavity incorrectly filled or emission reordered | L polygon and longer concave polygon |
| F5 | Interior hole filled; winding changed | Outer CCW/single CW hole, complete triangles |
| F6 | Multiple contours merged in wrong order | Two holes in authored order; hole order control |
| F7 | Fan/strip expansion parity differs | Larger convex/concave outlines, actual Native primitive trace |
| F8 | NORMALS_DOWN reverses wrong pair/order | Flip control preserves triple sequence and z |
| F9 | Stored z rounded or ignored | Exact nonintegral z and source-widened f32 z |
| F10 | Integer-to-mm scaling/negative values wrong | Authored fractional and negative integer coordinates |
| F11 | Repeated/collinear edges create spurious faces | Native-admitted duplicate/collinear contour controls |
| F12 | Degenerate contour creates fake triangles | Native-admitted empty/point/line controls |
| F13 | Intersection combine loses generated vertex/order | Native-admitted crossing contour; live Native combined coordinate bits |
| F14 | Per-call state leaks | A→B→A results byte/value-identical |
| F15 | Cached convex and sweep paths differ | Native callback primitive + ordered triples per case |

Run Native preflight before finalizing the authored cases; remove no Native-accepted
failure mode without a documented reason. A rejected input is explicitly
not a parity RED. Compare ordered coordinates exactly; area/count/orientation
may supplement but never replace ordered equality. Generated vertices need
source-derived identities distinct from submitted vertices, not invented
nearest-coordinate matching or sorting. Do not weaken any existing assertion.

RED commit follows this inventory commit. Report Native outcomes and precise
Rust compile RED.

## Revised RED verification summary

The runtime admission test compiled the original supplier independently under
both canonical official/seeded prerequisites: 18 authored cases per mode,
all Native exit0/no GLU errors (36 accepted runs, no Rust parity PASS).
Rust nextest --lib retains exactly one E0432 for the absent
triangulate_expolygon_3d operation: 19 comparisons planned, zero executed.
Fmt and strict clippy of the independent oracle test passed. Required-gate
negative checks for missing profile-entry flag, artifact root and Native
binary each failed (nextest100), without a SKIPPED result.
The initial SHA-format helper compilation error is excluded from RED evidence.
The original rejected frozen-order commit is replaced, not retained; no
authored input was changed. Receipts, compiler/source fingerprints, gate logs
and runtime Native products are not committed. Only this outcome summary is
committed; no Native output coordinates/indices
are copied into test expectations or repository artifacts.
