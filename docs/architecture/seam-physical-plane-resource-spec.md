# SPEC: physical Layer::slice_z handoff for the resource-control seam witness

Status: **SPEC-REVIEW-REQUEST, docs only**. Base main
`41f2c90f765d42c6620583121296bb61a2cb839c`; rebase only after stack-24
MAIN-ADVANCE. No permanent RED or production correction is authorized yet.
[ARD](ard-seam-physical-plane-resource.md).

## Upstream boundary and Rust destination

Orca source pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Paths in this table are relative to `OrcaSlicer/src/libslic3r/`.

| Owner | Source | Contract |
| --- | --- | --- |
| Physical plane producer | `PrintObjectSlice.cpp::compute_slice_z:24-47`, `new_layers:49-75`; `Layer.hpp::Layer:256-260` | Store one canonical slice_z per physical Layer, separately from height, print_z and raft-shifted id. |
| Candidate handoff | `GCode/SeamPlacer.cpp::gather_seam_candidates:1014-1040`, especially1024-1032 | Fetch the physical Layer and pass its slice_z to process_perimeter_polygon; region occupancy is not a plane source. |
| Scoring/selection consumers | `SeamPlacer.cpp::calculate_candidates_visibility:1042-1057`, `pick_seam_point:917-928`, `place_seam:1500-1650` | Score candidates at that plane, associate/select/project the seam, split the existing loop. |
| G-code consumer | `GCode.cpp::extrude_loop:5744-5775` | Invoke place_seam before clipping/emission; different seam starts change ensuing travel/extrusion. |

Rust already owns the producer in
`crates/ares-core/src/project_slice/layers.rs::planned_layers_with_zaa:243-283`.
`perimeters/classic/traversal.rs::PostClassicTraversalPrintObject::physical_layers:134-140`
already borrows that object's canonical planned layers. Reuse this view.

**Only proposed correction:** replace
`seam_placement/runtime.rs::layer_slice_zs:51-66` and its
`seam_placement.rs::apply_objects:136` call. Remove the region-record height
reconstruction helper and its now-unused ClassicTraversalRecord import; map
`traversal.objects[object_index].physical_layers()` to existing f32 slice_z
inputs at the caller. `alignment/preparation.rs::prepare:38-46` retains its
physical-layer zip. No new plane producer, resolver, pipeline or fallback.
The height scan is temporary Ares scaffolding around the upstream Layer concept,
not behavior to preserve. Remove its conflicting layer-bottom comment.

## Qualified same-request witness and first differing block

Positive is the **unchanged original resource-build control**:
`d951edca050a77d6bf7a0c90a14de35ceac8804104af7bb5d3cc2e40ad67157c`.
Fresh official Native bare `--slice 0 --outputdir`, public
`slice_project_plate_validated` with default overrides, and real bare CLI all
admit at main41f2. API==CLI raw SHA:
`14261facd18982eb5cf5212b642cbb36c08596d722bfceba3e5be0ee8462c79f`.
Both whole-file generator-only strict comparisons FAIL byte5217/line224 at
M73 P67 R2. No streams are missing and M73 remains in the comparator.

Motion995 (G0/G1/G2/G3 diagnostic projection only), raw line1198, is the first
different movement. It follows the identical inner-wall wipe ending
`X161.373 Y162.335 E-.36`:

| Block | Native | API/CLI |
| --- | --- | --- |
| First travel | `G1 X158.477 Y155.273 Z1.12 F15000` | `G1 X158.645 Y155.206 Z1.12 F15000` |
| Travel destination | `G1 X154.246 Y144.953 Z1.12` | `G1 X155.061 Y145.839 Z1.12` |
| First XY extrusion, line1203 | `G1 X154.021 Y144.916 E.00847` | `G1 X155.027 Y145.636 E.00766` |

This is the third physical layer (zero-based2, print_z0.72, slice_z0.62),
entering the first inner hole-wall loop, not a skirt/support/outer-contour loop.
Rust resolved region1 is modifier part3, sparse density50%, over the normal
part's20%. Read-only layer-boundary snapshots map all six loops at this layer
to region1. Native and Rust pre-seam loop15 (zero-based) have the same25-point
oriented closed cycle. All24 pre-seam loop cycles agree exactly in integer XYZ
up to cyclic start; reversal is NOT allowed in this diagnostic comparison.

Read-only Native seam plans versus unchanged Rust plans show:

| Physical layer | Native candidate z | Rust candidate z | Native/Rust selected indices |
| --- | --- | --- | --- |
| 0 | 0.16 | 0.16 | 2,9 / 2,9 |
| 1 | 0.42 | 0.32 | 5 / 5 |
| 2 | 0.62 | 0.32 | 0,8 / 0,10 |
| 3 | 0.82 | 0.32 | 0,10 / 0,10 |

All67 candidate XY coordinates and local angles agree bit-for-bit. None of the
seven perimeters is finalized into a seam string. At layer2, Native chooses
candidate8 `(4.054037,-4.207164,0.62)`; Rust chooses candidate10
`(4.209999,-4.021460,0.32)` in object-local coordinates (G-code origin150,150).
These precede loop splitting and travel routing.

Offline replay of captured Rust visibility samples/tree reproduces all67
current scores exactly. Changing **only query z** to the captured Native planes
recovers every Native perimeter choice. Candidate8 score changes
1.42107248→1.38636470; candidate10 changes1.42035854→1.39681113.
Corrected visibility is bit-exact for32/67 points, maximum absolute residual
4.76837158203125e-7; no visibility rewrite/tolerance is proposed. This is
counterfactual attribution, NOT an executed production GREEN or whole-output PASS.

The prior M73 premise is closed by owner: Native total457.6746216522006 equals
Ares replay of the complete Native stream exactly; Ares own stream totals
457.70780961899436. The earlier M73 boundary is a total-denominator symptom.
Do not port/change GCodeProcessor, move IDs, buffering, M73 formatting or clocks.
W3's different input/opposite symptom has not been diagnosed here.

## Minimal-delta passing control

Control SHA:
`6b26059cdff3ffd567fcc7d279154da27feb86d165d1deb6bfaeb00b21bd5c2c`.
Only `Metadata/model_settings.config` differs: modifier part3
`sparse_infill_density="50%"` becomes `"20%"`, equal to its parent normal part.
All mesh/model/relationship/global-setting entries are byte-identical.
This removes the effective modifier-region distinction, not the modifier mesh.

One fresh official bare Native invocation at main41f2 completed. The unchanged
whole-file generator-only comparator reports Native/API PASS and Native/CLI
PASS; API==CLI raw PASS. All three complete streams/receipts are retained.
No alternate control search or retry-to-success occurred. Seeded control/RED
execution is proposed for the separately approved test phase, not claimed here.

## Included, deferred and replacement boundaries

Included: handoff of the already planned physical slice_z, in physical order,
independent of region-major optional slots. Keep the existing f64→f32 conversion
and existing byte-oriented, platform-neutral API on browser WASM and desktops.
Canonical first-layer/ZAA/raft-relative planes are borrowed, not recomputed.

Deferred/unchanged: plane planning, ZAA activation, raft ids/print_z, region
resolution/occupancy/layout, Classic lower slices, perimeter generation/order,
candidate topology, mesh visibility/sampling, scoring/ties, seam strings,
projection/clipping, travel routing, arcs, M73/statistics/footer and libvgcode.
No regions[0], first-occupied/modulo/dedup/sort, height accumulation, new guard,
fallback, public diagnostic API or core I/O. If complete parity needs another
owner, STOP and report it separately; do not expand this slice.

The older [v2 seam closure](seam-candidate-slice-z-no-red.md) remains NO-RED;
its rejected fixtures/tests/entries are not reopened or merged. This is a new
source-owner request with a same-request public witness and qualified control.

## Approval gates and predicted RED

1. Exact-hash SPEC approval first. All probes stay outside git; no permanent
   fixtures/tests or production edits before approval. MAIN-ADVANCE then fetch
   and rebase docs; report changed premises rather than silently updating them.
2. Proposed bounded public RED: one paired control/positive test per mode,
   fresh official then seeded. Per mode: Native control→public API+real CLI
   complete whole PASS/raw equality, then fresh Native positive→both public
   calls/receipts before assertions. Positive must assert success/raw equality
   and both complete strict comparisons. Predict whole strict FAIL at the
   existing M73 boundary on41f2, with the line1198 motion witness retained.
   Missing streams mean NOT EVALUATED, not equality or a valid RED.
3. Proposed budget: four Native commands total (control+positive per mode),
   CPU15/jobs1/nextest threads1, fresh per-mode caches/artifacts/TMPDIR,
   load<60 before builds/every Native, START/END receipts and complete effective
   snapshots. Freeze exact two inputs/SHA-entry manifest and test-only commit;
   no Native output, debugger/probe or modified comparator in git.
4. Separate RED approval then GREEN authorization for the handoff only. Rerun
   frozen whole RED/control in both modes. Prediction: restored candidate8
   restores the three affected hole-wall starts, removes the motion/time delta
   and therefore the M73 symptom without touching M73. This prediction still
   requires complete whole-file verification; residual differences mean STOP.
5. Targeted seam/compatible-region regressions, fmt, strict scoped clippy,
   core+WASM adapter checks and changed Rust files≤400LOC. Coordinator alone
   schedules full/workspace and integration; FULL24 FREEZE remains active.

## Durable evidence

Report: `${LOCAL_EVIDENCE_ROOT}/worker4/motion-owner-main41f2-review.md`.
Root: `${LOCAL_EVIDENCE_ROOT}/worker4/motion-owner-main41f2-a62u`.
It retains first-motion localization, full positive/control Native/API/CLI
streams, Native24-loop/split and four-layer-plan snapshots, Rust plan/loop/
visibility snapshots, source hashes, exact replay and observer-neutrality checks.
Two additional read-only Native runs are complete and equal to the accepted
Native premise under the existing Orca generator normalizer. Successful Rust
observers preserve raw API output. Partial Rust observers (anonymous enum field
and optimized-out loop/record arguments) and compile-only disassembly mistakes
are retained/excluded; none is counted as a public PASS/RED. Archive SHA is
published with the external report and exact-hash review request.
