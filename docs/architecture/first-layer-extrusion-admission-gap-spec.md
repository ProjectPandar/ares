# SPEC: per-object first-layer extrusion admission for the odd-rotated gap

Status: docs-only SPEC; no RED or production change here.
Builds on the mesh edge ordering candidate.

## Owning upstream boundary

- GCode.cpp::collect_layers_to_print(const PrintObject&):1718-1778:
  pair object/support layers by print_z and EPSILON (:1744-1766), inspect
  the first pair for THIS object, and throw SlicingError if both members
  lack extrusion entities (:1771-1778). Later populated layers cannot rescue
  the first pair. Skirt, brim, custom start G-code and another object's
  extrusions do not satisfy this predicate.
- Layer.hpp::LayerRegion::has_extrusions:101: nonempty perimeters.entities
  OR fills.entities. Layer::has_extrusions:207 ORs across all regions;
  SupportLayer::has_extrusions:295 checks nonempty support_fills.
  This is collection presence, not a generic positive-E, polygon area,
  bounding-box, recursive path-length or triangle-count test.
- GCode.cpp::collect_layers_to_print(const Print&):1848-1866 preserves
  per-object failures and throws their aggregate. Exception.hpp:40-46
  names the aggregate Errors; OrcaSlicer.cpp:6286-6291 returns
  CLI_SLICING_ERROR (-100, Utils.hpp:72).
- GCode.cpp:1781-1827 later empty-layer warning behavior is adjacent and
  deferred. It must not be turned into another blanket fatal condition.

## Ares destination and scaffold audit

Rust destination, if a predicate defect is proven:
crates/ares-core/src/project_slice/gcode_emit/layers/schedule.rs.
Existing pair_layers:330-369 is the temporary representation adapter for
GCode object/support LayerToPrint pairing. build:219-246 ALREADY checks the
first merged entry per object and returns InvalidInput with the empty-first-
layer message. It currently derives object presence from nonempty ordered
island.entities (:225-230), support from support_fills (:231-233), and
handles the existing temporary raft adapter (:234-241).

OrderedExtrusionLayer/Island (island_print_order.rs:13-39) flatten the source
region collections. order_island:154-179 wraps perimeter/fill collections.
A presence audit must map these wrappers back to the cited Layer predicates;
no new Ares pipeline, public load/selection API or CLI error-code redesign
is included. Existing scaffold remains a compatibility shell around those
upstream collections, not a new independent admission policy.

## Durable evidence summary (retain even if /tmp roots expire)

Tests/parity/mesh-edge-pairing/generate.py and the committed input fixtures
reconstruct all geometry. Odd variants have identical vertices and oriented
triangle multiset: cube plus one nonzero-area face, three uses on a vertical
indexed edge; odd-rotated moves the first seven triangles to the end.

| Input | SHA256 | Native official/seeded | Ares after pairing | future_gap |
|---|---|---|---|---|
| odd-reverse | 05e50f1e7587f0d40821e39f4c84007ec01a4977fe862ccd69631976ee99babe | result-100 / Unix156, stderr Errors, no G-code | InvalidInput(empty first layer), CLI1, no G-code | false |
| odd-rotated | 93a493cec3f419398d2cfc04401e0c5f53c373ffe38d2e526c885e876f6b5bc3 | result-100 / Unix156, stderr Errors, no G-code | API Ok / CLI0, identical G-code | true |

Ares odd-rotated raw G-code SHA256 in BOTH modes:
99521ddc83fa146da43b50d88271afb586afc95f2c046f875089130fb6afc6b4.
The admitted matrix passes13/13 each mode with full
ordered generator-header-only comparison and API/CLI raw equality; three
manifold controls are byte-identical to RED d5d5bdcc1. Diagnostics are not
strict parity PASS. No retry-to-PASS or extra output normalization was used.

Read-only inspection of this exact Ares G-code finds M83 (relative E), first
;LAYER_CHANGE at line51, second at line160; first print_z0.2. Within that
first-layer block there are65 XY moves with positive relative E:4 Inner wall,
4 Outer wall,57 Bottom surface. Priming/custom moves before line51 and pure-E
unretractions are excluded. Thus this is actual first-layer object extrusion,
not evidence of an absent Ares first-layer check. The positive-E count is
an artifact observation ONLY; it is not the proposed admission predicate.

Native logging, with unchanged input hashes, established one printable
instance,13 triangles, perimeter generation including layer0, completion
of infill and then an export exception. The longer no-first-layer-extrusions
text is attributed through the unique GCode.cpp throw/aggregate chain, not
verbatim stderr. This proves differing layer/extrusion state at export;
it does not prove std::sort alone is the cause.

Canonical seeded replay uses seed0.

## Preflight gate and implementation STOP

Before any production, audit the paired first-layer Z, object identity and
region perimeter/fill collection presence in both implementations using
existing artifacts and permitted read-only inspection, without code probes.
The odd-reverse negative control demonstrates that Ares can already reject
an empty first layer. Odd-rotated demonstrates real first-layer Ares paths.

If the Ares predicate is falsely true only because the ordered adapter
misrepresents empty source collections, identify that exact transformation
and prove a public counterexample before a minimal schedule adapter patch.
Do not assume that defect exists from the current G-code evidence.

If correct object/support collections really are nonempty in Ares but empty
in Native, GCode.cpp:1774-1778 cannot fix this gap by itself. STOP and submit
a separate geometry/state SPEC citing the proven owning boundary, such as
TriangleMesh.cpp:520-613 or TriangleMeshSlicer.cpp:1383-1482/1865-1889.
No sorting replay, make_loops rewrite, empty-layer retagging, dropping emitted
paths, fixture-specific branch or mesh_topology guard is included here.

## RED and exit plan

1. Commit a separate public Native rejection RED for exact odd-rotated:
   both modes must return Native-100/Errors/no G-code while current Ares
   demonstrably succeeds. Do not reinterpret it as success parity. Preserve
   the existing diagnostic collector; introduce an independently named
   admission gate, without changing its assertions or the13 admitted cases.
2. Keep odd-reverse as typed-error/no-output control and odd-original as
   strict Native0 control. Add public object-only empty-first-layer,
   support-present first-layer, later-layer-only, and multi-object isolation
   fixtures ONLY after Native proves their intended admission. Retain all
   rejected/admitted evidence, including inconclusive cases; do not retry
   an input to obtain a desired admission result.
3. Report the predicate/state preflight with RED. If the STOP condition
   holds, propose the separately sourced next boundary and write no GREEN.
   Otherwise submit a small schedule-only rewrite of the proven mismatch,
   preserving all existing assertions and the raft compatibility mapping.
4. Successful closure requires odd-rotated future_gap=false from the
   correct upstream state/predicate, both-mode admitted13/13 byte parity,
   negative-control typed errors/no-output and strict/fmt/wasm32.
   An independent geometry SPEC, if needed, owns that closure instead.

All runs require fresh Native cache/artifact roots; no test retirement is part
of this slice.
