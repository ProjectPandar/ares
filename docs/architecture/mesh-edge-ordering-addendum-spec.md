# Mesh-edge addendum: Native rejection attribution and accepted-input RED

Docs-only SPEC supplement. No public RED, production or existing assertion change.
This supersedes the earlier speculative standalone sort-replay proposal;
no isolated replay/compiler experiment is requested by this addendum.

## Native observations and retained input revisions

First preflight:
Two closed prisms sharing a vertical indexed edge ->4 uses; identical world
geometry with split indices; shared faces original/reversed/interleaved/rotated.
Official/seeded each5/5 exit0/result0. All ten outputs share generator-only SHA
6480e4c1a1ec8b2cc275cd1b090bcf415d6a7ddfc5957f265264aed0b0927aec.

Second preflight:
The planned coverage risks motivated explicit input revisions:

- Cube control: one closed prism,2 uses per ordinary edge.
- Odd inputs: one cube plus a nonzero-area face on its vertical edge ->3 uses;
  original/reverse/rotate have identical vertices and oriented triangle multiset.
  A triangle-multiset audit confirms all SHA330216f23d9abd53fc505c21b68756767131e237e750233fcf69dd12901e6e74.
- Six-use: duplicate the first prism's faces in the first tracer, with
  original/reverse/rotate order, preserving world coordinates.
- Collapsed diagnostic: append (0,0,0) to the first tracer, preserving its body.

Each mode6 exit0 plus2 failures: odd-original is admitted, generator-only SHA
92de92462c3bde6deacb42f3c6e78d4f3512f6963f24905a33ec299820945e31;
odd-reverse/odd-rotated return result-100, Unix process156, no G-code. Six-use
and collapsed cases match the first tracer's SHA. Collapsed remains diagnostic.
No data was overwritten, and no command was repeated to seek a PASS variant.

## Exact -100 cause: empty first extrusion layer at G-code export

Original stdout: `run found error, return -100, exit...`; stderr: `Errors`.
The same channels/results persist with existing Native logging enabled:

- A run that changes only
  --debug3; without a log sink it prints no additional exception detail.
- A run that changes only --debug5
  plus --logfile to a fresh per-case native-trace.log. Original input SHA is
  unchanged, fresh cache/artifacts each run; this is attribution, not retry-to-PASS.

Trace for all four failures (both modes/two face orders) establishes:
`got 1 printable istances`, `volume triangle count 13, total 13`,
`start Print::process`, `slice_mesh to polygons`, perimeter generation including
layer0, `Filling layers in parallel - end`, followed by
`found slicing or export error for partplate 1` and saving result.json.
Thus file import/config/print-volume admission succeeded. It is not a native
blanket unsupported-mesh or high-valence parse rejection.

The inner exception text is NOT printed verbatim by the CLI. Its exact source
chain is unambiguous under the pinned upstream boundary:

1. GCode.cpp::collect_layers_to_print(const PrintObject&):1718-1830 has one
   SlicingError throw, at1774-1778: the first layer has no object/support
   extrusions. Message: "One object has an empty first layer and can't be
   printed. Please Cut the bottom or enable supports."
2. GCode.cpp::collect_layers_to_print(const Print&):1847-1866 catches that
   SlicingError, collects it and throws SlicingErrors(errors).
3. Exception.hpp:40-46 constructs this aggregate with Exception("Errors").
   The observed stderr `Errors` therefore identifies this aggregate, not the
   longer inner text. Repository source has no other throw of this aggregate.
4. OrcaSlicer.cpp:6286-6291 catches std::exception, prints ex.what(), records
   CLI_SLICING_ERROR with its generic result text (-100), then exits.

This source+log attribution distinguishes the catch path from the warning path
at OrcaSlicer.cpp:6188-6194, which does not print the aggregate exception.
Do not misquote the inner text as directly observed stderr.

## Classification and revised RED boundary

Native refuses to export these two SPECIFIC ordered inputs because their first
extrusion layer is empty. The same triangle multiset original order is admitted,
so this also demonstrates deterministic face-order sensitivity somewhere between
mesh connectivity and export. It does NOT prove std::sort alone is responsible;
triangle traversal/make_loops can also be order-sensitive. No Native instability,
new Ares rejection or portable replay of -100 is claimed.

Strict public RED contains ONLY Native-admitted
inputs: first tracer five cases; cube control; odd-original; six-use
original/reverse/rotate. Keep odd-reverse/odd-rotated in external diagnosis and
manifest admission classification, not in parity tests requiring success.
No committed failing parity test is removed: public RED has not been written.
Collapsed diagnostic remains outside the claimed feature matrix.

Preflight one admitted Arachne control and an unequal-orientation repeated-face
case for the same-direction fallback before including them. Keep rejected cases
as admission diagnostics instead of changing shape/order repeatedly to obtain
success. Public tests require result0, compare complete output generator-only,
and compare API/CLI raw bytes; current Ares mesh_topology guard defines RED for
accepted high-valence inputs. Closed/open-boundary controls stay unchanged.
Inputs/generator/provenance and RED are a separate commit before production.

## Ordering risk and implementation STOP boundary

Keep TriangleMesh.cpp:create_edge_map520-563 and its_face_edge_ids_impl565-613
as the owner; TriangleMeshSlicer.cpp:1865-1902/1982 and make_loops1383-1482 are
existing consumers. Rust target remains mesh_slicer/topology.rs, with existing
chaining transport unchanged unless separately specified.

C++ compares only endpoints and std::sort is not stable; Rust presently adds
face/local-edge order. First/even-valence accepted permutations show no public
output difference, but they do not prove all mappings equivalent. Do not
silently invent a stable-sort policy or port a generic library algorithm.
If an accepted-input GREEN comparison exposes a mapping/order discrepancy that
requires a sort or make_loops rewrite, STOP for source-cited proof/addendum
before that change. A blanket guard deletion alone is never GREEN.

Read-only audit: official ELF comments GCC14.2/clang18.1.3,
seeded GCC15.3/clang21.1.8, both libstdc++; these do not pin per-TU header versions.
Available GNU15.3 stl_algo.h:88/1806-1822/1851-1858/1876-1908 documents median,
threshold16, introsort/heap fallback/final insertion. No compiler replay,
Native patch/probe or new sort module follows from these observations.

Continue with fresh roots. Separate exact negative-test retirement after RED;
no assertion edits now. Strict/fmt/wasm and public gates remain mandatory.
