# Task 22O Package A0 Amendment: Fixed Voronoi Sidecar Qualification

## Status and amendment boundary

This specification is a draft amendment to the approved Task 22O Classic
perimeter-generator specification and plan. It does not replace or edit either
approved document. It adds one detached prerequisite, Package A0, before the
tracked Package A geometry work may choose or introduce a Voronoi engine.

The immutable parent documents are:

- `docs/superpowers/specs/2026-07-21-ksr-fdmtest-v4-task22o-classic-perimeter-generator.md`,
  SHA-256 `78c44972e284eb615bf96228cbc5d0fe3a5c731a853c3b1cf518f92219b95674`;
- `docs/superpowers/plans/2026-07-21-ksr-fdmtest-v4-task22o-classic-perimeter-generator.md`,
  SHA-256 `94c361d0d4c89eb5019f07f3a3e4101b8d89857d02c06629e3c794920f645e80`.

No Package A0 execution may start until this amendment and its implementation
plan are frozen as one exact frame and receive two independent document
approvals. Package A0 is ignored evidence work only. It creates no tracked Rust,
Cargo, lockfile, notice, architecture, roadmap, workflow, or test edit.

## Frozen parent evidence

The fixed Ares baseline remains commit
`e27e8c7736a768d1b4d7d291c62a553c722e1f21`, tree
`6a69f55b22a2638c383cfb811dac94d5389d5056`. The fixed OrcaSlicer source remains
tag `v2.4.2`, commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
tree `b62d6017ba1ac7cb986f70fd6844353c7a776549`.

Package 0 is immutable and approved. Package A0 may reference but must never
edit, regenerate, or replace these artifacts:

- source manifest SHA-256
  `fe334df1d9f2d10706cbdc521024cee644d67db93883c113bdc6542f4dd8508d`;
- final manifest SHA-256
  `48305cb8ec6859622cdc34012e015586eb833b5de2bdc6bdc78b3991b14f54a2`;
- KSR aggregate SHA-256
  `ca6b3f925568e9aacbf198df54f2404b96ffccd89889481cb1ddcfbf34406e92`;
- verifier result SHA-256
  `36c236057c40245f3092a39d68663dfe66e8d581292341ca87dabe978566d52d`;
- fixed-source/runtime review SHA-256
  `6e1b57c38de5ca61db9f79b1ca14a3123bd74e774e061e16fe200f372b43d1c7`;
- qualification/provenance review SHA-256
  `acf0a1388afd54dd33211e8e0afa58dffc56c0a265a8a971670fbcd74d07a5cc`;
- detached approval envelope SHA-256
  `747fa8432c45a5bfee28fd1eecbac12b20604fa635780471930294cc287563dc`.

The supplied KSR 3MF and reference G-code identities remain respectively
`698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`
and `10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3`.

## Reason for the amendment

The approved `ORCA22O` version 1 payload freezes final Classic entities and gap
surfaces. It does not contain the raw MedialAxis segment input, native Voronoi
cells and directed half-edges, source categories, repair/closing result,
inside/outside annotations, finite-primary validation decisions, endpoint
widths, neighbor rotation, or ThickPolyline chaining decisions. Final entities
cannot uniquely reconstruct those states or prove that a candidate engine has
the same native order and branch behavior.

Package A therefore lacks enough independent evidence to accept
`boostvoronoi` or any other implementation. Package A0 adds a fixed-source,
observational sidecar and an exact candidate comparison. It does not change the
Task 22O production behavior or authorize a fallback algorithm.

## Fixed Voronoi and dependency identities

The sidecar and comparison are bound to these fixed Git blobs:

- `src/libslic3r/Geometry/Voronoi.cpp`,
  `4f7173e5c282b8bff362449303ea55003460a045`;
- `src/libslic3r/Geometry/Voronoi.hpp`,
  `23a50bb8f74812a63a2b029e01feb9b41234b469`;
- `src/libslic3r/Geometry/VoronoiUtils.cpp`,
  `f140782891ceddf537f29ed33a22288c1cce2ef3`;
- `src/libslic3r/Geometry/VoronoiUtils.hpp`,
  `6872e4ea7215be633962039493a82f1222388c3a`;
- `tests/libslic3r/test_voronoi.cpp`,
  `b86f448c02b6b97cf579f35361e929001774bc39`;
- `deps/Boost/Boost.cmake`,
  `896031680aff0f02939ae7ddbddf04337022041b`.

Fixed Orca uses Boost 1.84.0. Its archive SHA-256 is
`4d27e9efed0f6f152dc28db6430b9d3dfb40c0345da7342eaa5a987dde57bd95`.
The only candidate in this amendment is `boostvoronoi` 0.12.1, crate archive
SHA-256 `077839b66f949d7728595b25f9c24b6d6972e0fd3afbd35cf034de66dd1bdb34`,
upstream commit `4060736209b06919a08590f37480d88b429599ba`, Rust 1.87.0
minimum, BSL-1.0. Its Boost-derived lineage makes it eligible for comparison;
it is not accepted by lineage, package metadata, or a successful build alone.
The ignored probe is a standalone Cargo workspace with its own lockfile and
target directory. Its dependency requirement is exactly `=0.12.1`; all locked
build/test commands must leave the root workspace manifests and lockfile
byte-identical.

## Fixed derivative source boundary

Create a new detached fixed worktree and clean out-of-tree build. Reapply the
approved Package 0 seven-file patch without changing its bytes, then add the
sidecar. Relative to that Package 0 parent, the complete allowed delta is
exactly these nine paths:

1. `src/libslic3r/CMakeLists.txt`;
2. `src/libslic3r/PrintObject.cpp`;
3. `src/libslic3r/Ares22OOracleRuntime.hpp`;
4. `src/libslic3r/Ares22OOracleRuntime.cpp`;
5. `src/libslic3r/Geometry/MedialAxis.cpp`;
6. `src/libslic3r/Ares22OVoronoiOracle.hpp`;
7. `src/libslic3r/Ares22OVoronoiOracle.cpp`;
8. `src/libslic3r/Ares22OVoronoiWire.hpp`;
9. `src/libslic3r/Ares22OVoronoiWire.cpp`.

Relative to fixed commit `8500fcd...`, status must contain exactly the original
seven Package 0 paths, `Geometry/MedialAxis.cpp`, and the four new
`Ares22OVoronoi*` files: twelve paths total. `OrcaSlicer.cpp` and
`Ares22OOracle.hpp/.cpp` must be byte-identical to the approved Package 0
parent. `Voronoi.cpp/.hpp`, `VoronoiUtils.cpp/.hpp`, `ExPolygon.*`,
`PerimeterGenerator.*`, `Thread.*`, `Print.cpp`, and every unlisted source file
must be byte-identical to the fixed commit.

The sidecar must not instrument `Voronoi.cpp` or claim the wrapper's internal
per-angle repair-attempt trace. It records only the final state returned by each
unchanged `construct_voronoi()` wrapper call. Every added instrumentation source
file remains below 400 physical LOC.

## Activation and observational constraints

The first statement of the already approved runtime guard captures
`ORCA22O_VORONOI_PATH` together with `ORCA22O_PAYLOAD_PATH`. A nonempty sidecar
path is valid only when paired with a nonempty absolute payload path. Normalize
each absolute final and its exact `final + ".tmp"` path using the resolved parent
directory and platform path-comparison rules. O final, O temp, V final, and V
temp must be pairwise distinct and all four must be absent. Any empty, relative,
aliased, or existing path fails closed before argument conversion or slicing.
With both variables absent, the derivative behaves like fixed Orca and creates
no payload, sidecar, or temp. Payload-only activation preserves the approved
Package 0 path contract and must remain byte-identical to approved Package 0.

`PrintObject.cpp` resolves the stable object index in fixed `Print::objects()`
order before the perimeter `tbb::parallel_for`. Inside its worker lambda, each
call to `m_layers[layer_idx]->make_perimeters()` is wrapped by a narrow
thread-local RAII token carrying object and layer indices. `MedialAxis.cpp`
records only while that worker token is active, excluding earlier slicing,
later Fill processing, and unrelated Voronoi users. Each invocation appends in
call order to its preallocated object/layer slot. Serialization traverses object,
layer, then slot order; it never sorts records after capture. The exporter may
not alter an input, branch, return value, scheduler source, container,
allocator-visible lifetime, or result.

Immediately after the existing `export_ares22o_payload(*this)` call returns and
before `posPerimeters` completion, call the V finalizer for the same PrintObject.
It fills a preallocated PrintObject slot and returns until every object slot is
complete. On the last object, it first verifies that the exact O final exists,
then serializes and publishes only `ORCA22V` through V temp to V final. No
destructor performs finalization. Any capture or publish failure propagates
before the perimeter step is marked done. Ignored tooling later constructs
`ARES22V` from the exact complete O and V files; fixed code never writes the
composite.

## Sidecar capture contract

For every captured MedialAxis invocation, preserve native order and record:

- invocation ordinal; exact `double` min/max-width bits; input ExPolygon; and
  initial `m_lines`;
- after the first unchanged wrapper call: returned state, issue, `is_valid`,
  and complete final repaired diagram;
- when morphological closing occurs: exact replacement lines and the second
  returned state, issue, validity, and complete final diagram;
- after inside/outside annotation: ordered native cells, vertices, and directed
  half-edges, including source categories, twin/next/previous references,
  rotation neighbors, finite/primary/curved flags, and annotations;
- every finite-primary validation decision, endpoint predicate, exact endpoint
  width bits, accepted/active result, and branch identity; and
- native neighbor-rotation and chaining decisions plus the raw returned ordered
  ThickPolylines, widths, and endpoint flags.

Encode stable native indices, never pointer values. Do not sort, deduplicate,
normalize, sample, apply a tolerance, omit a field, or reconstruct a state from
the final `ORCA22O` entities.

The fixed direct probe also emits three non-MedialAxis record variants without
editing `Voronoi.cpp`: raw point sites passed to the unchanged Boost 1.84
constructor, raw ordered segments passed to the unchanged Boost constructor,
and ordered segments passed to the unchanged Orca `VoronoiDiagram` wrapper.
Raw records contain their exact ordered input and complete public final native
diagram. Wrapped records additionally contain `try_to_repair_if_needed`, final
state, issue, and validity. MedialAxis-only annotation, validation, width,
rotation, and chaining fields do not exist in those variants; they are not
encoded as misleading zero or null values.

## Wire contract

The inner sidecar is exactly `ORCA22V\0`, little-endian `u32 version=1`, a
little-endian `u64` record count and ordered tagged records,
`ORCVEOF\0`, then physical EOF. Counts and indices are `u64`, scaled
coordinates are `i64`, floating-point values are encoded by exact bits, enums
and booleans use canonical `u8`, and nullable indices use one documented
sentinel. The canonical record tags are MedialAxis, raw points, raw segments,
and wrapped segments; each tag has a closed, variant-specific field sequence.
Recursive lengths, discriminants, and references are checked before allocation
or dereference by the ignored parser.

The neutral comparison composite is exactly `ARES22V\0`, a little-endian `u64`
approved-parent length, the exact complete approved `ARES22O` payload, a
little-endian `u64` sidecar length, the exact complete `ORCA22V` payload, then
physical EOF. Publish the sidecar through a fresh temporary and atomic replace
only after complete serialization succeeds.

## Corpus and qualification controls

The corpus is exhaustive for the approved frame, with no sampling or
deduplication:

- all KSR plus A1-A7, B1, C1, and D1 captured perimeter invocations;
- fixed `test_voronoi.cpp` ordinary, hole, multiple-hole, edge-collapse,
  duplicate, intersecting, missing-vertex, and repair vectors; and
- direct MedialAxis rectangle, concave/T-junction, one-hole, two-hole, and
  near-degenerate closing cases with exact min/max widths.

Before executing the direct corpus, freeze a source-case inventory that binds
each selected fixed `test_voronoi.cpp` `TEST_CASE` and exact constructor call
line range to its fixed blob, ordered input variable/range, record tag, inclusion
reason, and any explicitly deferred point/segment call. The inventory must
resolve every ordinary, hole, multiple-hole, edge-collapse, duplicate,
intersecting, missing-vertex, and repair coverage claim; a category label alone
does not identify a case.

A4 and A6 may correctly contain zero MedialAxis calls but do not satisfy any
topology-coverage requirement. The manifest records actual invocation,
state/issue, closing, cell/edge category, validation, chaining, and
ThickPolyline coverage. Any missing required branch blocks engine selection.

The out-of-tree fixed probe links the derivative fixed libslic3r and creates an
explicit probe-only record session. Raw point/segment cases call the unchanged
Boost constructors; wrapped-segment cases call the unchanged Orca wrapper;
direct MedialAxis cases enter the same layer token used by slicer workers. All
variants use the same record serializer and atomic publisher. The probe is not
part of the fixed-source delta and may not activate capture by changing
environment state after process startup.

Every fixed run uses a fresh hashed datadir clone, fresh absolute output paths,
archived root logs, a bounded supervisor, no retry, and no overwrite. Execute:

1. both variables absent: successful G-code and no O/V/temp output;
2. payload only: two fresh processes for KSR and all ten positive synthetics,
   exact approved O bytes, no V output;
3. sidecar only: fail closed before slicing, no G-code/O/V/temp output;
4. both variables: two fresh processes for the same eleven inputs, exact
   approved O bytes, byte-identical V and composite pairs, strict recursive
   parser/EOF/index/parent-binding success; and
5. direct fixed corpus probe twice, with byte-identical complete V wires.

The first failed attempt is retained and blocks the gate. Do not run a third
process, choose a preferred run, or change the wire/comparator after observing
a mismatch.

## Immutable approval stages

Package A0 has two distinct immutable subjects.

First freeze `sidecar-manifest-v1.json` after the fixed derivative, corpus,
controls, and paired runs pass. Two fresh reviewers inspect that exact manifest:
one reviews fixed-source identity, seams, capture completeness, wire, and parent
O preservation; the other reviews corpus, isolation, repeatability, indices,
EOFs, and absence of normalization or run selection. Their detached envelope
binds both reports and the manifest without mutating any reviewed file.

Only after that envelope approves may the candidate adapter be written. Compare
every accessible native-order cell, directed edge, category, reference, flag,
vertex, repair/closing result, validation decision, endpoint width bit, rotation
decision, and ThickPolyline value against the approved sidecar corpus. Run the
candidate tests and both default and `wasm32-unknown-unknown` checks.

Then freeze a separate `engine-selection-manifest-v1.json` referencing the exact
approved sidecar manifest and envelope. Two new review turns inspect the exact
engine manifest: one for semantic field/bit parity, one for native/WASM,
dependency lineage, license, and no normalization. A second detached envelope
binds those reports. The sidecar and engine reviews may not be collapsed.
Native semantic comparisons, native tests, the default check, and the WASM
compile check use the Tier-1 Rust 1.91.0 toolchain, not the ambient default
toolchain. Record and verify the exact compiler/Cargo identities and the
1.91.0 `wasm32-unknown-unknown` target. A0 proves WASM compilation only;
browser runtime parity remains Package H.

If any field is inaccessible or differs, coverage is incomplete, or either
platform check fails, freeze `selected_engine: null`; leave workspace Cargo,
lockfile, notices, tracked REDs, and production untouched. A new document
amendment is required before a fixed Boost subset or another Rust path is
authorized. No approximate library, raster skeleton, hardcoded KSR result, or
legacy fallback is allowed.

If every comparison and review passes, Package A may freeze its own exact
tracked leaf manifest and introduce only the approved engine, adapter, and
required BSL notice during its RED/GREEN cycle. Package A0 itself remains
ignored-only.

## Amendment exit criteria

Package A0 is complete only when:

1. the immutable parent docs and Package 0 artifacts still match every stated
   hash;
2. this spec/plan frame has two independent detached approvals;
3. the derivative has exactly the approved nine-path parent delta and
   twelve-path fixed-commit status;
4. all parent-equality and fixed-source-equality assertions pass;
5. wire corruption REDs and comparator mutation REDs pass;
6. every control and two-run qualification completes without retry or residue;
7. the sidecar manifest has two independent approvals and a detached envelope;
8. the candidate is compared field-for-field and bit-for-bit on the complete
   corpus and passes default plus WASM checks, or is explicitly rejected;
9. the separate engine manifest has two new approvals and a detached envelope;
10. no tracked workspace file other than these amendment documents changed.
