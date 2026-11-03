# Fresh mesh centering: widen bbox endpoints before averaging

Status: docs-only SPEC REVIEW-REQUEST. No production, test or fixture change.
Stack parent: reviewed safe-zone `0699507534806d594099a772600e58a8d087ed6e`.
Branch: `worker6/mesh-center-f64-spec`. No FULL window requested.

## Owning source and exact Rust destination

The producer is fresh ModelVolume mesh preparation, not wipe formatting:

| Upstream source | Required behavior |
| --- | --- |
| `TriangleMesh.cpp:431-437`, `TriangleMesh::bounding_box` | Widen `m_stats.min/max` float endpoints to double. |
| `BoundingBox.hpp:261`, `BoundingBoxf3` | Inherits `BoundingBox3Base<Vec3d>`, not Vec3f. |
| `BoundingBox.cpp:231-237`, `BoundingBox3Base::center` | `(min + max) / 2` in the bounding box's point type: here double. |
| `Model.cpp:2600-2617`, `ModelVolume::center_geometry_after_creation` | Retain the full Vec3d shift; cast its components to float only for mesh/convex-hull translation, while volume offset and init shift retain double. |
| `TriangleMesh.cpp:271-284`, `TriangleMesh::translate` | Existing vertex subtraction remains float. |
| `TriangleMeshSlicer.cpp:1830-1861`, slicing transformation | Later convert the composed slicing transform to float; do not prematurely round its compensation shift. |

Exact Rust production destination is
`crates/ares-core/src/project/load/mesh_prepare.rs::center_shift`.
Preserve the existing f32 source-vertex min/max reduction; widen each endpoint
before addition and division, for all three axes. Return the resulting f64
`Point3d` shift without narrowing it back to f32. Keep `prepare`'s existing
f32 subtraction and full-precision `component_transform.translated(shift)` /
`init_shift` handoff unchanged. Correct the touched helper's inaccurate comment
that currently describes BoundingBoxf3 as Vec3f; that is the mistaken source
interpretation being replaced, not an additional rounding-policy redesign.

Existing `PreparedMesh` and component-transform carriers remain compatibility
adapters around ModelVolume's centered mesh/init shift. No new Ares pipeline,
loader mode, public API, file I/O, UI or platform-specific core behavior.

## Read-only Native-first evidence and first differing producer

Fixed external witness: two authored normal-part volumes, second volume's Z
extent shortened to 5.4 with its bottom retained; beam enabled, orientation 0,
beam width 0.5, top shell layers 1. Both official and seeded Native double
replay are stable. Complete Native/CLI streams have identical 165432-byte
lengths and differ only at line 571 (apart from the existing generator line):
Native `G1 X125.811 Y114.025 E-.08885`, Rust `... E-.08886`.
Beam-disabled and height-9.4 variants are complete-stream PASS controls.

GDB located the earliest observed difference in the centering helper itself:

| Value | Native | Rust before repair |
| --- | --- | --- |
| retained Z shift | -4.799999952316284 | -4.800000190734863 |
| shift cast to f32 for mesh subtraction | -4.800000190734863 | -4.800000190734863 |
| later float volume translation | 2.700000047683716 | 2.6999998092651367 |
| facet bottom Z at slice Z=0.5 | 2.384185791015625e-7 | 0 |
| facet sloped-edge upper Z | 3.6000001430511475 | 3.5999999046325684 |
| raw slice corner integer | 9611112 | 9611111 |
| overhang-clipped wipe interior X | 9811122 | 9811121 |

Trace checkpoints: facet input → raw regions **before interlocking** → before
microstructure → lower-slice offset input → overhang clipping input → before
seam-tail `clip_end` → stored wipe path. The one-grid-unit discrepancy exists
before all the downstream operations. In particular, overhang clipping gets
the same subject polygon but already-different lower polygons. The wipe starts
at the same `[10025002,3965002]`; its next vertex differs by one X unit. The
unchanged upstream `Wipe::wipe` dE formula independently reproduces
`-0.08885462680130957` versus `-0.08885501193382397`, explaining the printed E
without any formatter defect or compensation.

External producer report and raw debugger receipts:
`${EVIDENCE_ROOT}/w6-stepped-wipe-centering-producer.json`.
Fixed Native-first replays:
`${EVIDENCE_ROOT}/w6-stepped-wipe-fixed-{official,seeded}/comparison.json`.
No Native output or debug probe is committed. No GREEN has been attempted.

## Included and deferred behavior

Included: only the source-owned precision of the three-axis fresh bbox-center
average, preserving f32 endpoint values and downstream subtraction semantics.
The stored init shift and volume compensation consume that double result
through their existing paths, including already-supported mesh sharing.

Deferred: fresh/shared ownership selection, component order, signed-volume
winding correction, unit conversion, bounding-box point collection, Native's
near-zero shift guard, transform multiplication/FMA association, instance
centering, clipping/intersection arithmetic, seam splitting, wipe budgets,
E formatting, and all miter/join parameters. This is not a boundary-miter
witness. Do not modify `TriangleMeshSlicer` consumers or insert an epsilon to
cancel a legitimate residual Z. No legacy fallback or Unsupported guard.

If the bounded center-average repair leaves any whole-output residual or
regresses a control, stop and report it; do not silently broaden the slice.

## Public RED matrix and shared runner

After separate test authorization, freeze only the three missing input-only
external variants below. Preserve their exact archived bytes; their generator
script and original Native receipts remain outside Git. Reuse existing base
controls instead of copying them, and use `runner/native_stream_case.rs`
without duplicating or weakening its assertions.

| Case | Expected initial verdict | Input SHA256 |
| --- | --- | --- |
| step5.4-beam1 | genuine target RED | `114204a6de7dcaf101c5ea2ce525746f0ece22440ba3d680475b48b9c1f388af` |
| step5.4-beam0 | PASS control | `6ec7579e5b2e5887085505791d264bccec68885c5c16da70a4aee91bee0fdca0` |
| step9.4-beam1 | PASS control | `e4903ade421dab2970cd8110238859190cc332dfe46c4843040bfb790c11162a` |
| default two-extruder beam1 | existing PASS control | `04c7d2a58b7b21909b867542faad149a6b9d0b1bb976eab3fec57b7b60ff082c` |
| original single volume | existing PASS control | `1ac6d319469fbee245bdc9b44b1600980b5777e62a0e84f34fac0f798dc85651` |

Run every case in both official and seeded modes. Confirm Native twice before
validated core byte-in/options API and real CLI, with the existing complete
ordered generator-only Native/API and Native/CLI comparisons, input SHA
assertions and raw API/CLI equality. Existing mesh-sharing/centering regressions
must also remain PASS. Failure inventory: retaining an early f32 average;
narrowing the resulting compensation shift; accidentally changing f32 mesh
subtraction; breaking existing fresh/shared owners; hiding E differences by
normalization or rounding. Assert complete outputs, not one expected E word.
No ignores, skips, probes, Native snapshots, allow/expect or comparator changes.

## Sequence and gates

1. Review this docs-only source boundary and producer evidence first.
2. With separate authorization, commit fresh Native-first public RED/controls
   in both modes; discovery's CLI-only receipts are not a substitute for API RED.
3. Production-only GREEN is confined to `mesh_prepare.rs::center_shift` and
   its directly corrected comment. All complete target/control streams must
   pass before claiming completion. No unit tests written after production.
4. Changed files remain below 400 lines; fmt and workspace/all-target clippy
   with `-D warnings` clean; preserve all four Tier-1 platform boundaries.
5. Reviewed endpoint → inward → safe-zone stack remains queued and unchanged.
   This spec does not start or request FULL. Current queue is W4 → W2 → W3
   full stack → W6 → W5. Integration rebases onto then-current main and requires
   exact-final-hash dual FULL 0 FAIL/0 skipped under coordinator authorization.
6. No push, stash, self-merge, stale-base exemption or extra CPU authorization.
   All commits retain the required coauthor.
