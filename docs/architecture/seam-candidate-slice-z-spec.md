# SPEC: Preserve physical Layer::slice_z in seam candidates

Status: **REVIEW-REQUEST, docs only**. Base `c251aa6166bcc0270063fc6d2e41b84ab01a637d`.
No production correction or permanent RED test is authorized by this document.
See [ARD](ard-seam-candidate-slice-z.md). This is a source-cited rewrite handoff,
not a new Ares seam algorithm or slicing pipeline.

## 1. Upstream owner and Rust destination

Orca source pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Paths below are relative to `OrcaSlicer/src/libslic3r`.

| Boundary | Source file:function:lines | Contract |
| --- | --- | --- |
| Plane producer | `PrintObjectSlice.cpp::compute_slice_z:24-47` | One plane per physical layer: midpoint without ZAA or on first layer; otherwise lo+region-derived offset. |
| Layer construction | `PrintObjectSlice.cpp::new_layers:49-75`, especially61-64; `Layer.hpp::Layer:256-260` | Store computed slice_z separately from height, raft-shifted id and print_z. |
| Candidate consumer | `GCode/SeamPlacer.cpp::SeamPlacer::gather_seam_candidates:1014-1040`, especially1024-1032 | Fetch physical Layer, pass its slice_z to process_perimeter_polygon. Region occupancy does not select candidate plane. |
| Subsequent consumers | `GCode/SeamPlacer.cpp::calculate_candidates_visibility:1042-1057`; `place_seam:1500-1650`, especially1508-1509/1536-1560 | Score physical-plane candidates and associate/select/project the seam. `GCode.cpp::extrude_loop:5769-5775` invokes placement. |

Rust producer already exists: `project_slice/layers.rs::planned_layers_with_zaa:243-283`
retains canonical `PlannedLayer.slice_z`. It remains the compatibility shell for
compute_slice_z/new_layers; do not create another plane producer.

**Correction destination:**
`crates/ares-core/src/project_slice/seam_placement/runtime.rs::layer_slice_zs:51-66`
and its caller `project_slice/seam_placement.rs::apply_objects:136-138`.
Borrow the object's existing physical planned layers through the retained
perimeter-input predecessor and map their slice_z values to the existing f32
candidate input. Replace the region-record height scan, not the planner.
`alignment/preparation.rs::prepare:38-46` continues consuming one value per
physical ordered layer; do not change association, scoring or selection.

Source hashes of the inspected copies (SHA256):
- PrintObjectSlice.cpp: `d1aece12e7189353aa9a5d8ae6b1f0f320087c0d2574d7da64fe372eca076021`
- Layer.hpp: `3ca7e46d481f5bd4b627a08d5ff332ea5a8c86b11ee2259b21080ce18810651e`
- GCode/SeamPlacer.cpp: `1eb7af658e1c8530b96197ea4bc8c0e3700eabbfa43134123f5cf336c99a8bdd`

## 2. Witness and limits of attribution

Approved read-only localization used only immutable v2 `populated-control`:
input SHA `b2f1e2676b6e2ab5ede4a0a9f731115ce19ed5861dcbe337b5608d5dd988811c`.
Both implementations have four physical layers, empty R0, occupied R1, owner1.
Layer0 generator contours/holes match ordered integer coordinates exactly.
Native read-only observation records slice_z [0.16,0.42,0.62,0.82].
Ares flattened records are [None,None,None,None,R1L0,R1L1,R1L2,R1L3].
The runtime scan yields [0,0,0,0,0.16,0.42,0.62,0.82]; preparation zips four
physical layers against the zero prefix. Output-neutral Ares frames confirm
candidate z=0 on all four layers.

The first differing movement is travel59 at layer0, labelled Outer wall until
the next TYPE marker: after the outer-hole wipe, toward the **inner contour**.
Native targets NE (157.819,157.819), Ares SE (157.819,142.181), both CCW.
Ares seam placement receives an NE-starting inner loop and rotates it to SE
using external candidate (8.75,-8.75,0). This occurs before travel routing.
The full-stream first difference remains M73 at byte1644/line77; it is retained.

Ares context/append and seam probes each preserved **raw complete output**:
SHA `56fef7da418cf1af67ab93480642b6f4fed43cec51afeab72f7da055d18d00b6`.
Native occupancy observation exited normally, recorded12 frames, and preserved
complete Native output under established generator-only normalization;
binary SHA `64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`
was unchanged. Extra Native seam observation failed an observer ABI guard:
no complete G-code/barrier, partial seam frames excluded, no retry-to-success.
A preceding Ares compile-only diagnostic failure is retained separately.

Thus the plane handoff discrepancy is witnessed, but correcting it has NOT
been executed. Do not claim complete causal GREEN, Native seam internals,
seeded confirmation, or that all earlier contour/hole divergences share it.
The approved Classic append lower-slice discovery remains a separate boundary.

Durable report: `/home/indexyz/.ares-evidence/worker4/layer0-populated-control-localization.md`.
Archive: `/home/indexyz/.ares-evidence/worker4/layer0-populated-localization-evidence.tar.gz`,
SHA `6e9f2245a5e4430969fd55ea2e3b24aabb7cf5bddd87b2157a64fb07b376e09d`.
It retains successful neutrality receipts and unsuccessful observer histories.
ALL temporary hooks/registration/module were removed and clean status verified
before rebasing/creating this docs-only branch directly from current main.

## 3. Included and deferred behavior

Included: physical-layer plane handoff to the shared seam candidate consumer,
independent of empty region slots, grouping owner or region count. Reuse the
existing canonical plane, including whatever valid first-layer/ZAA/raft-relative
values its producer already retains. Keep the existing candidate f32 conversion.
Remove the conflicting runtime comments describing slice_z as layer bottom.
This replaces temporary height-reconstruction scaffolding around Layer::slice_z.

Deferred/unchanged:
- Layer plane planning, ZAA activation/min-Z policy, raft identity/print-Z policy.
- Classic append lower_slices, steep reversal flags/math and wall/brim order.
- Region grouping, redistribution, occupancy and traversal record layout.
- Visibility mesh/sampling, candidate topology, tie-breaking, seam-string search,
  nearest/random/aligned selection, loop splitting and motion/travel routing.
- Missing-build 3MF loader finding, N-split and excluded PASS-only accounting work.

Do not use regions[0], first occupied record, modulo/dedup/sort, cumulative
per-region heights, guessed raft offsets, fallback or a new guard. No new
public API, retained diagnostic I/O, fixture substitutions or comparator edits.
Core remains byte/options to byte output on WASM/Windows/macOS/Linux.

## 4. Test-first approval and acceptance plan

1. Obtain exact-hash SPEC approval before tests/production. Proposed bounded
   public RED set uses only frozen v2 populated-control and single-control;
   no new geometry/options. Fresh official then seeded roots, CPU15/jobs1/
   nextest threads1, load<60 before builds/Native, inbox START/END. Per mode:
   positive, fresh positive repeat, single-region control (3 comparisons).
   This proposed new-slice budget requires approval; old discovery budget is
   not silently reset or extended. Honor exclusive freezes; never delete Nix stores.
2. Freeze a **separate test-only commit** using `ares_core::slice_with_profiles`
   and the CLI adapter, complete ordered G-code and established generator-only
   normalization. Native commands must exit0/complete snapshots with effective
   settings verified. API/CLI outputs must agree. Positive must reproduce a
   deterministic genuine full-output RED; single-region control must PASS.
   Input bytes/mesh/options/transforms stay frozen; no Native products committed.
3. Request independent RED approval before the two-file handoff correction.
   No accounting-only merge or reuse of PASS N-one cases as RED. Stop/report
   admission/control failures, NO RED FOUND, or a changed first boundary.
4. After RED approval, change only the cited runtime input and caller. First
   rerun exact RED/control and repeat in both modes, then existing seam-mode
   and compatible-region controls. Retain movement/arcs/M73/statistics/footer.
   If whole-output RED remains, stop and request an amendment; do not repair
   adjacent seam scoring/motion to force GREEN or reinterpret stage parity as PASS.
5. Verify scoped core nextest, cargo fmt, strict clippy, wasm32 clippy and LOC<400.
   Use cargo nextest, not cargo test, as the Rust test entry. Archive repeatable
   complete artifacts outside /tmp; submit exact-hash REVIEW-REQUEST.
   Coordinator owns full scheduling, push and merge.
