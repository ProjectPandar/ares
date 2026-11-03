# Slice 2: separated per-object skirts under ByObject — execution SPEC

Status: discovery complete on approved Slice 1 GREEN
`c1780265dd00dfd9322ffc6bba734d335c5311fe`; public RED authorized by owner.
Parent [two-slice SPEC](per-object-skirt-by-object-spec.md) remains authoritative.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Fresh discovery after Slice 1

`${TMP_ROOT}/ares-byobject-skirt-slice2-discovery`: official and seeded, CPU18, independent
Native caches/artifacts, canonical env archived from current coordinator path,
Native first then same-input validated API/CLI, production unchanged.
Existing disjoint fixture: two distinct objects at x80/x140,y110, height4,
one filament, no_brim, combine_brims0 (verified current source/runtime key),
skirt2/height1, support0/raft0/tower0. Geometry/non-config members unchanged.

Both modes: ByLayer/perobject and ByObject/combined now strictly match complete
Native/API/CLI and raw API==CLI. Target ByObject/perobject: Native exit0, two
skirt blocks and sequential object stacks; Ares UnsupportedProjectFeature(
"skirt_type per-object"), CLI1, no G-code. Slice 1 removed the previously
reported comment blocker without hiding or excluding comments.

An additional same-geometry target with skirt_distance35 was probed solely to
investigate the shared-group rejection boundary. BOTH Native and Ares reject
EARLIER at sequential clearance (Native -63/POSIX193; Ares
SequentialPrintCollision). It is NOT a witness reaching `_make_skirt`'s shared
SlicingError and NOT the admission RED. Do not claim shared-group source-branch
reachability from this input. Retain its evidence; no clearance bypass or new
geometry is authorized. Public RED uses the three admitted inputs above.

## Exact source / Rust boundary and constrained GREEN proposal

- `OrcaSlicer/src/libslic3r/Print.cpp:2646::_make_skirt`, :2780-2977 constructs
  instance-shifted occupied hulls, nominal-reach groups and per-group loops.
  `Print.cpp:2548-2557` throws SlicingError for shared perobject groups under
  ByObject: this case must remain guarded, not accidentally admitted.
- `Print.cpp:1286-1303::validate` and :613::sequential_print_clearance_valid
  remain unchanged. The distance35 probe shows their earlier rejection only.
- `GCode.cpp:2730-2762`, :3280-3374, :5127-5136 owns sequential instance ordering,
  complete object layer stacks and per-filament order. Existing Rust ByObject
  lifecycle and Slice 1 transition comments are reused unchanged.
- `GCode.cpp:2036` initializes group history once; :5240 resizes it;
  :4465-4483 and :5334 use the object's own group's history. There is no
  clear-all-groups transition reset. :4286-4328 first/later-layer history gates
  and :4395-4414 flow-height calculation remain authoritative.

Rust destination:
`crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs::SkirtPlan::generate`,
with existing `skirt/member_groups.rs` and `layers/object_skirt.rs` consumers.
The minimal repair cannot blindly remove the ByObject disjunct before groups
are known. Proposed bounded admission: remove ONLY that early sequence
rejection, then retain rejection for ByObject/perobject if an existing computed
`generation_groups` group has multiple object members. Reuse the current
UnsupportedProjectFeature rejection for unsupported shared groups; do not claim
that this compatibility guard fully ports Native's shared SlicingError type/map.
This is a source-owned scope guard, not a new geometry grouping algorithm or
fallback. Full shared-group diagnostic parity needs its own authentic reachable
input and separate approval. If this placement exceeds owner scope, stop before
GREEN rather than silently admitting shared groups.

W5's tower/multi-filament and W2's support/raft disjuncts MUST remain unchanged.
No consumer reset, new ordering policy, geometry, brim or API changes.
The emitter remains a temporary compatibility shell for Print/GCode concepts.

## Public RED and acceptance

Freeze the three admitted inputs only, exact SHA/member manifest, no Native
output in Git. Runtime `OrcaRunner::from_env` Native FIRST, assert Native exit0.
Target assert two skirt blocks; second occurs AFTER `move to origin position for
next object` and BEFORE the second stack's local id0 extrusion, first before
first-stack id0 (Native reuses local labels per process_layers); positive
XY/E, 40 layers and complete sequential stacks. Both controls retain strict whole
ordered Native/API/CLI equality and raw API==CLI. Target asserts that SAME strict
equality and real API/CLI success, so pre-GREEN fails on actual unsupported API
rejection; never inverted acceptance or divergence-is-success.

Independent RED approval before ANY production patch. GREEN must retain inputs,
assertions, controls' raw bytes and comparator, preserve shared-group rejection,
and rerun both modes fresh CPU18, existing ByObject/perobject controls and
allocated fmt/strict/WASM. No full/workspace/push/merge; coordinator owns broad
verification. Changed Rust <=400 LOC.

Failure inventory: Native invalid/clearance/shared input is not positive RED;
missing product, wrong effective settings, env/compile/timeout/affinity faults
are NON-GATE. Wrong group timing or second-group history is a real consumer
failure requiring amendment, not global reset. Comment differences cannot be
stripped. If all match, evidence-only NO-RED STOP. No broader unsupported input
widening. Deferred copies-of-one-object, shared-group support/diagnostic parity,
draft/minimum/multilayer skirt, brim, support/raft, tower and multi-filament.
