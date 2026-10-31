# G4 sequence-count review receipt

Status: bounded GREEN under the owner's explicit one-time Native nondeterminism adjudication; integration approval still required. No push performed.

## Source boundary and retained scope

The admitted slice is `PrintConfig.cpp:1115-1123`, `ParameterUtils.cpp:6-29`, `GCode/ToolOrdering.cpp:1320-1362` and first-layer `498-513`, with plate admission from `Format/bbs_3mf.cpp:4428-4435`. The Rust consumers already existed; the key-local parity harness now proves count0/1/2, empty-vector and reverse-priority controls with actual extrusion roles/order and unchanged whole-output comparison.

Production repair is type-owned footer nil semantics from `Config.hpp:837,1015,1344,1878` and `GCode.cpp:5632-5633`, in `ares-core/src/options/config_types/{semantic,vector,opaque}.rs` and `config_export/{value,writer}.rs` / `value/sequence.rs`. Empty/all-nil numeric/bool vectors are nil, zero/false remain nonnil, canonical nil cannot be resurrected by raw fallback, and empty String/Point/Group footer tokens remain visible. No key exception, ToolOrdering, support/infill/travel, shared settings-stack, master preference or Manual-map production edits. Deferred G4 keys, malformed/unbounded/by-object sequences and load-settings/PCP stay outside.

Fixture admission failures and the rejected 211/233 PLA temperature receipt remain historical, not production RED. The owner-approved fixture uses 211/223 and the observer preserves inherited TYPE roles across layers. Source-backed zero JSON expectation correction and the sole old empty-Floats nil expectation correction are explicit; the latter is folded into production GREEN, not a surviving follow-up assertion weakening.

## Fresh gates and exact tested tree

Full BASE `eade4762ab168297620395b6afbdc852779735e0`; tested CAND `855bada9c5c92f8eb23a97c96f7f44e2b661c99d`. Engineering ran at `6635985d48388afa37fb2dfb7725b4cbec3de477` on production main76, with owner-authorized docs-only carry across eade. All older production-base gates are VOID for integration credit and retained historically.

- `/tmp/worker2-g4-engineering-76-final.0Vs4kE/workspace.log`: 8295/8295 PASS, 5 skipped, 3009.648s, exit0.
- Same directory `clippy.log`, `fmt.log`, `wasm.log` and corresponding `.status`: all exit0; Clippy workspace/all-targets/all-features `-D warnings`; fmt check; wasm32 checks core/vgcode/wasm. Jobs2/threads2, fresh workspace with ARES_/ORCA_ unset.
- `/tmp/worker2-g4-76-directed-build.3Vy01i/{clippy,compile}.log`: CLI parity strict/build exit0.
- `/tmp/worker2-g4-76-directed-seeded.mUKSnx/nextest.log`: 2/2 PASS, 922 skipped, 9.868s; official `/tmp/worker2-g4-76-directed-official.O6CQJd/nextest.log`: 2/2 PASS, 922 skipped, 19.410s. Each covers five standalone consumers plus three sweep cases; `artifacts/option-coverage-summary.md` reports generated3/compared3/rejected0 for the selected key. No whole-output or consumer assertion relaxed.

Full root `/tmp/worker2-g4-full-eade-fresh.L1rhpk` retains `trees.json`, each `<leg>.commit`, `<leg>-process.json`, `<leg>-start-load.txt`, `<leg>.exit`, `<leg>.log`, fresh caches and complete artifacts. Standard configuration has no `ARES_PARITY_OPTIONS` or key filters; independent base/candidate targets, resource symlink layout, jobs2/threads2. No automatic full retry.

| Leg | Actual nextest summary | Exit |
| --- | --- | --- |
| seeded-base | 923 PASS / 0 FAIL / 0 skipped, 1052.117s | 0 |
| seeded-cand | 924 PASS / 0 FAIL / 0 skipped, 1061.753s | 0 |
| official-base | 923 PASS / 0 FAIL / 0 skipped, 807.441s | 0 |
| official-cand | 923 PASS / 1 FAIL / 0 skipped, 810.288s | 100 |

The driver exit0 records four-leg completion, **not** an unqualified literal all-PASS result. The owner's adjudication below makes the bounded G4 full gate accepted. Cross-tree all-case raw-input inventory / 650-plan identity audit is not claimed completed by this receipt; per-case raw artifacts are retained for review.

## Sole Native nondeterminism adjudication

The owner explicitly ruled `option_coverage::normal_support_output::interface_fan::support_interface_fan_preserves_complete_output_and_role_speed`, official-cand `closed-layers`, as Native nondeterminism, not G4 regression, and accepted G4 full with this **single observed failure exempted once**. This is not a blanket flaky allowance. The owner separately opened the Native nondeterminism issue.

Evidence: `/tmp/worker2-g4-full-eade-fresh.L1rhpk/interface-fan-adjudication-audit.json`, produced by `/tmp/worker2-interface-fan-offline-audit.py`. Four full receipts plus official directed BASE2/CAND2 under `/tmp/worker2-interface-fan-directed.iMDWTn` retain every manifest/input/Native/Ares stream. Directed runs all passed (8.045s, 8.147s, 8.120s, 8.209s); original failure remains untouched. All nine serialized ZIP members match across all eight observations. Ares raw SHA is identical 8/8: `96913956f36359125c7a264c4ced339e3295388728229ead6d7ed8dd0642f590`. Native raw hashes are all retained; diagnostic generated-banner-only grouping is seven identical vs one outlier. This offline grouping changes neither saved bytes nor the comparator.

Outlier first difference line850 is `G1 X113.388 Y113.135 F9000` vs the seven passing Native streams' `G1 X113.244 Y113.297 F9000`. Nineteen differing lines exchange two Internal solid infill path blocks, not support interface extrusion. Configuration is one logical filament, Auto For Flush, sequence count0/vector[0]; no Manual map or new sequence harness is selected. Shared vector/footer serialization is traversed, but all eight Ares outputs including footer are identical and no G4 diff modifies infill/travel.

Candidate upstream investigation boundary: `GCode.cpp:6150-6172` (`extrude_infill`, collection chaining) and `ShortestPath.cpp:1061-1069` (entity reordering). Exact nondeterministic origin/tie-break is **not proved**; merely finding support `parallel_for` is not evidence that it caused this internal-solid reorder. No production change, comparator tolerance, output stripping, new golden or further Native run was used to resolve this gate.

## Review handoff

Review the complete commit chain, source-cited SPEC/RED/approved nil repair, metadata promotion and key-only sweep wiring. The final docs commit changes no tested code. Request exact-hash approval before any non-force FF integration. Original failures, interrupted/VOID runs and admission errors remain available, rather than being relabeled GREEN.
