# B-T isolated RED: failure modes before tests

Status: failure-mode inventory written before any B-T test/oracle/production
code. Coordinator approved isolated RED for this geometry sub-slice; public
S45 end-to-end output proof belongs to the later Support preparation slice.
Base29f4a49c18139925fec238d31d2fdb6b790fbc30, approved SPEC3d863ce63
rebased456fd2a2b with range-diff1/1 `=`. No production implementation here.

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

Ares geometry is crate-private in geometry.rs. With the coordinator's
isolated-test ruling, B-T tests live under geometry/tessellation/tests.rs;
geometry/tessellation.rs contains test registration only in RED, and
geometry.rs registers the module. No new public API is needed for this RED.
The desired private source operation is triangulate_expolygon_3d:
ExPolygon + z + flip + existing CoordinateScale -> ordered Vec<[f64;3]>.
CoordinateScale is Ares's temporary coordinate representation adapter;
Normal reproduces upstream unscale<double>. No scale-policy change here.

The first isolated Rust RED is compilation failure for the missing operation,
explicitly NOT a Native-runtime failure or parity PASS. Native source/oracle
admission and expected ordering are collected independently first. No stub,
empty result, Unsupported guard, ignore/cfg skip or normalized tolerance is
introduced to fabricate a runtime failure. RED cannot merge without GREEN.

Native reference is a CPU12 test-only executable using the original vendored
C files and wrapper-equivalent contour submission/callback expansion, outside
Git. Kernel/source hashes and complete ordered vertex identities/coordinates
are recorded. Rust assertions reference authored input vertices and ordered
identities, not committed generated Native meshes/G-code/output files.
Oracle traces/raw results remain outside Git. Native S45 function observations
remain provenance/hand-off, not a standalone B-T whole-slice PASS claim.

## Failure modes and planned Native-backed coverage

| ID | Failure the source port could have | Isolated check |
|---|---|---|
| F1 | Rectangle diagonal or callback order differs | Convex quad complete ordered triples |
| F2 | Coordinate axes or contour start changed | Translated/asymmetric quad, preserve x/y order |
| F3 | Auto normal treats clockwise input differently | Reverse contour; freeze actual Native order |
| F4 | Concavity incorrectly filled or emission reordered | L polygon and longer concave polygon |
| F5 | Interior hole filled; winding changed | Outer CCW/single CW hole, complete triangles |
| F6 | Multiple contours merged in wrong order | Two holes in authored order; hole order control |
| F7 | Fan/strip expansion parity differs | Larger convex/concave outlines, actual Native primitive trace |
| F8 | NORMALS_DOWN reverses wrong pair/order | Flip control preserves triple sequence and z |
| F9 | Stored z rounded or ignored | Exact nonintegral z and source-widened f32 z |
| F10 | Integer-to-mm scaling/negative values wrong | Authored fractional and negative integer coordinates |
| F11 | Repeated/collinear edges create spurious faces | Native-admitted duplicate/collinear contour controls |
| F12 | Degenerate contour creates fake triangles | Native-admitted empty/point/line controls |
| F13 | Intersection combine loses generated vertex/order | Native-admitted crossing contour; exact constructed crossing |
| F14 | Per-call state leaks | A→B→A results byte/value-identical |
| F15 | Cached convex and sweep paths differ | Native callback primitive + ordered triples per case |

Run Native preflight before freezing the cases; remove no Native-accepted
failure mode without coordinator ruling. A rejected input is explicitly
not a parity RED. Compare ordered coordinates exactly; area/count/orientation
may supplement but never replace ordered equality. Generated vertices need
source-derived identities distinct from submitted vertices, not invented
nearest-coordinate matching or sorting. Do not weaken any existing assertion.

RED commit follows this inventory commit. Report Native outcomes and precise
Rust compile RED with exact hash; no Rust algorithm GREEN until reviewed.
All compile/Native/nextest processes CPU12 outer taskset, jobs1/threads2,
fresh run root; no full/workspace allocation or push. Freeze overrides runs.
