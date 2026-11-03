# SPEC: default Voronoi detection and rotation-repair prerequisite

Status: SPECac538 independently approved by COORD; RED authorized on9bbc.
Additional conditions: reuse the merged `tests/parity/voronoi-incident` environment/
observer/driver in place; inherit `ORCA_REPLAY_CPU`, `ARES_ORCA_DEBUG_DIR` and
`ARES_GDB_BIN`; runtime-only Native records, no duplicate harness/host defaults.
A bounded read-only search of merged inputs must report actual MedialAxis
repair-active reachability honestly. GREEN requires targeted consumer regressions
AND the final coordinator-scheduled FULL before integration; current work is RED
only. New main notifications do not relabel this branch's evidence.
Planning base: `9bbc63c937a7383a639317e40a95e15ad2bdefd7`.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
All source paths below are relative to `src/libslic3r/`.
[Non-negotiable decisions](voronoi-default-repair-prerequisite-ard.md).

## 1. Why this source slice blocks painted MMU

The independently approved painted SPECf894 and public RED0ce6 remain on their
5317 branch. Coordinator paused that program after its read-only graph evidence
found a REAL repair-active case. Incident fix5317 changes cell anchors, not
`Geometry::VoronoiDiagram`'s default detection/rotation repair. Raw Boost builder
success is not equivalent to the effective diagram returned to the caller.

Actual frozen `painted-split.3mf` (archive SHA256
`a18492a3cb74b68012e2d8c8975dafaa81267e77731cdab28411badeb4dc31cc`), ordinary
Native CLI slicing, reached this stack without branch forcing:
`PrintObject::slice_volumes -> apply_mm_segmentation ->
multi_material_segmentation_by_painting -> segmentation_by_painting ->
build_graph -> Geometry::VoronoiDiagram::construct_voronoi<ColoredLines>`.
`MultiMaterialSegmentation.cpp:1555-1556` uses the default-true repair argument.

The first observed repair record, call0002 of split Native1, has
`repair_enabled=true`, `State::REPAIR_SUCCESSFUL=1`, final
`IssueType::NO_ISSUE_DETECTED=0`, `m_is_modified=true`; effective10 cells,
11 vertices,40 directed edges. Its ordered original input is:

```text
(10000000,10000000) -> (-10000000,10000000)
(-10000000,10000000) -> (-10000000,-10000000)
(-10000000,-10000000) -> (10000000,-10000000)
(10000000,-10000000) -> (10000000,-9400000)
(10000000,-9400000) -> (10000000,10000000)
```

This is an actual projected collinear color split, not a synthetic failure,
invalid-input workaround or Rust-derived expected geometry. Endpoint/color/
polygon/local-line identity and full caller stack are retained. In that ONE
split process,49 of50 effective diagrams were repair-successful; one was not
repaired. Two leaves processes captured100 no-repair diagrams; two unpainted
processes correctly made zero MMU graph calls. All5 Native processes exited0.
The driver STOPped10 immediately on examining the first repair record, before
Rust backend replay, split Native2 or producer implementation. This is NOT a
complete phase qualification or a measured Rust numeric mismatch. The initial
issue and selected repair angle were NOT captured and must not be inferred
from the final issue0/state1. No official effective-VD capture is claimed.

## 2. Exact upstream owner and Rust destination

| Upstream functions/data | Included Rust boundary, OUTSIDE vendor |
|---|---|
| `Geometry/Voronoi.hpp:49-75,89-112`: `IssueType`, `State`, effective cells/edges/vertices and default-true segment constructor; `Voronoi.cpp:28-73` | New private `crates/ares-core/src/geometry/voronoi/{mod.rs,diagram.rs}`: source-owned effective VD facade and default constructor. Existing patched `boostvoronoi::Builder` remains only the raw construction backend. |
| `Voronoi.cpp:90-158` `copy_to_local` | Same `diagram.rs`: preserve complete vector/index/pointer topology, source category, anchors, flags/colors, effective storage and clear/reset semantics; allocate local copied storage only when repair requires it. |
| `Voronoi.cpp:166-178` `detect_known_issues`, `185-225` `detect_known_voronoi_cell_issues`, `227-240` `detect_known_voronoi_edge_issues` | Private `geometry/voronoi/{detect.rs,cell_range.rs}`: same issue precedence, finite/focus predicates, segment-cell range traversal and intersection/missing-vertex decisions. |
| `Voronoi.cpp:248-261` `try_to_repair_degenerated_voronoi_diagram` | Private `geometry/voronoi/rotation.rs`: try `PI/6, PI/5, PI/7, PI/11` IN ORDER, stop at first success, retain exact unsuccessful state/last issue. |
| `Voronoi.cpp:263-283` endpoint encode/decode; `290-362` `try_to_repair_degenerated_voronoi_diagram_by_rotation` | Same `rotation.rs`: rotate integer sites, rebuild using the same backend, copy local, detect in rotated coordinates, preserve endpoint identity, rotate back non-endpoints, clear temporary vertex marks. |
| `Point.hpp:236-247` `Point::rotate/rotated`; `Geometry/VoronoiUtils.cpp:212-254,307-345` segment-cell range, `to_point/is_finite/make_rotated_vertex`; source-point helpers `VoronoiUtils.cpp:39-91` and `VoronoiUtils.hpp` | Private `geometry/voronoi/{cell_range.rs,source.rs,rotation.rs}`: source rounding, arithmetic order, source-segment/point identity and incident/color preservation. No anchor normalization. |
| `Geometry/VoronoiUtilsCgal.cpp:40-138,188-332`: filtered parabolic tangent predicates, `orientation_of_two_edges:227`, `check_if_three_edges_are_ccw:270`, `is_voronoi_diagram_planar_angle:303-332` | Private `geometry/voronoi/{planarity.rs,predicates.rs}`: Rust rewrite of these exact filtered/exact predicate semantics. No CGAL/native FFI, replacement geometry engine or f64-only approximation. |
| `Geometry/MedialAxis.cpp:458-460` actual default constructor consumer | Existing `geometry/medial_axis/diagram.rs::build` and its existing annotation/validation/chaining consumers use this effective VD, replacing their raw-Builder-only handoff. Adapt those owners only as needed; do not add a dormant wrapper or public phase API. |

All new Rust files <400LOC; split along these named functions. No vendor change
belongs to this slice. A backend defect/interface barrier discovered during the
port causes another bounded STOP, not a hidden vendor patch. Preserve initial/
reverse segment source categories; public accessors that collapse them are not
proof of preservation. Production must not parse debug strings or raw Rust
object memory to recover metadata. Any required source-index/orientation mapping
must be source-cited and verified against full Native categories before use.

Existing `medial_axis` is a temporary compatibility shell around the cited
upstream MedialAxis consumer, not an Ares pipeline. It is the real existing
consumer for this prerequisite to land BEFORE painted code, avoiding an unused
private helper or admission bypass. No caller may label unrepaired raw output
as repaired or return it as a fallback after repair failure.

## 3. Included / deferred

Included: default segment-constructor dispatch, all invoked edge/cell/angle
issue detection, source precedence and all4 repair attempts, copied effective
storage, endpoint remapping/vertex mark cleanup, issue/state/is_valid semantics,
clear/reset and actual existing MedialAxis default-constructor consumption.
Source-enabled predicates are not replaced by witness-specific zero-only stubs.
Expose no IO, terminal, GDB or native runtime dependency in portable core.

Deferred: point-only/mixed-point constructor expansion; unrelated VoronoiUtils
operations and unused `is_voronoi_diagram_planar_intersection:157-186` alternative;
`Arachne/SkeletalTrapezoidation.cpp:378` also invokes the default-true segment
constructor. That real Arachne consumer and Arachne-wide migration are deferred
in this slice, not assumed raw-builder-equivalent. New medial-axis annotation/
chaining algorithms defer, as does
`MedialAxis.cpp:462-472` downstream morphological-closing retry. Keep the existing
caller error contract on unsuccessful construction; do not claim that deferred
closing behavior or all MedialAxis parity. If active qualification needs it,
STOP for an independently reviewed extension rather than silently choosing raw
geometry. Deferred likewise: painted carrier/registry/projection/graph/slabs/
merge/apply/admission, cutting, new UI or G-code behavior. Painted stays paused
until this independent prerequisite is approved, qualified and integrated.

## 4. Durable measured evidence and RED candidate

Canonical root (verified complete mirrors, NOT only temporary storage):
`${EVIDENCE_ROOT}/worker1/voronoi-default-repair-spec-0ce6/`.
`PATH-MAPPING.json` maps historical absolute `/tmp` prefixes to durable paths;
`DURABLE-INVENTORY.json` verifies all1750 copied files /47,855,913 bytes with
matching SHA256. No previous results are relabeled as this SPEC/base evidence.

Inside `ares-w1-painted-projection-phase-0ce6/`: `BEFORE-CODE.md`,
`provenance.json` (exact ELF/debug/source13 hashes), read-only observer/source,
`STOP-WRAPPER-REPAIR-GAP.json`, split Native1 `capture/call-0002/record.json`,
raw arrays/layout, entry/finish thread masks and caller stack, Native G-code/
launch/exit logs, `STOP-audit-result.json` and artifact inventories. All150 actual
effective diagrams were independently decoded from saved bytes. Split's one
observed full G-code matches ordinary same-input Native using the unchanged
existing comparator; this is NOT an independent split repeat. Infrastructure
preparation failures are retained NON-GATE. No backend replay or Rust geometric
RED was measured in this stopped attempt; approved RED0ce6 is public rejection,
not this prerequisite's constructor RED.

After SPEC approval, separately commit a test-first, runtime-generated RED on
its exact hash. Prefer this frozen `painted-split` reachable Native witness;
keep the ordinary unpainted/leaves controls. New actual Native repeats must
show `REPAIR_SUCCESSFUL` on the same whole ordered input and identical complete
effective arrays. Use the actual default constructor in the real MMU caller;
no raw-Boost replacement, disabled detector, invented diagram or branch forcing.
Capture initial issue, attempt order/angles and copied/remapped results via
read-only GDB; missing capture/debug is infrastructure, not semantic RED.
Seeded debug evidence does not qualify official DWARF; ordinary complete public
Native outputs in both modes remain distinct controls.

Agreed prerequisite seam: existing `geometry::medial_axis::diagram::build` and
its source-owned default VD handoff. Tests may consume runtime Native records
through a test adapter, but must invoke this REAL current Rust constructor,
compile and run, and compare all effective cell/category/anchor, vertex f64
bits/color and ordered edge/topology/flags against Native. Never use compile
failure, malformed/painted XML rejection, synthetic expected-error success,
debug-field ignores, tolerance or normalization as the prerequisite RED. If
current Rust unexpectedly matches, report it; do not manufacture a failure or
expand fixture search without owner approval. Future state assertions must
exercise the source facade, not assertions against copied Native metadata.

Before tests/implementation enumerate failure modes: missed/default-disabled
repair; wrong issue precedence; miscopied vectors/twin/cycle/anchor/category;
rotation/rounding/angle order drift; incorrect endpoint mapping; residual colors;
failed repair hidden by raw fallback; local/raw ownership confusion; incorrect
filtered/exact predicate decisions; changed consumer output; stale/omitted Native.

## 5. Review and exit gates

Docs-only SPEC/ARD -> independently reviewed fresh reachable constructor RED ->
minimal source rewrite plus existing consumer -> independent exact-hash GREEN.
The present Native witness authorizes no implementation. Retain exact frozen
archives/member/settings hashes, explicit Python/environment configuration,
actual singleton parent CPU propagation and raw output. Require fresh independent
Native repaired and no-repair repeats before Rust results; no cached credit.

Exit: complete ordered effective-diagram equality, wrapper state/dispatch and
endpoint-remap parity on the approved Native witness set; real MedialAxis
consumer regression evidence plus approved targeted VD/annotation/Arachne and
support consumers, fmt, strict Clippy and WASM. On a broader prerequisite or
backend gap STOP; no alternative engine, field weakening or cfg/allow parking.
Any FULL/workspace or integration-window validation requires coordinator
scheduling; no FULL/push/merge now. Final integrated exact-hash evidence is not
inherited from old hashes. CPU18/jobs2/threads2 is the current bounded allocation.
