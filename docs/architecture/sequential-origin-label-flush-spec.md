# Sequential origin-travel object-change labels


## Owning upstream slice

OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1 `GCode.cpp::GCode::_do_export` sequential transition calls `travel_to(Point(0,0), erNone, ...)` (lines 3321–3322; local excerpt3327). `GCode::travel_to`7466–7467 unconditionally consumes writer object-change labels after travel acceleration/retract and before travel moves. `GCodeWriter.cpp1181–1200::add_object_end_labels/add_object_change_labels` consumes end before start and owns absolute-E reset semantics. Queue ownership is `GCode.cpp5478–5494`.

Rust destination: `ares-core/src/project_slice/gcode_emit/layers/transition.rs::emit`, after existing `motion::begin_path_travel`, before direct origin-hop writer moves. Reuse existing `motion::append_object_start` (end-before-start consumption from motion/state/object_labels.rs), no new helper, state or formatting. Existing transition is a temporary compatibility shell for the upstream export/travel boundary; its missing consumption is not an independent Ares pipeline design opportunity.

Included only: literal pending-label consumption at this upstream-owned boundary and its necessary source comment. Deferred: generic travel, queue/name/ID/label generation, formatting, motion/lift/retract, geometry, cooling, M73, priming multi-tool boundaries, SEMM/tower, collision/arrangement, other flavors and unrelated orchestration.

## Evidence and public test seam

Original K1 overhang+cube-hole sequential Native-arranged input SHA256baa6c8d352d5e62c400cc450d50de643ce868fcc619a71352748f89aabe7aab1. Four fresh Native slices replay stable; twoAPI/twoCLI successful/allrawidentical. Real strict first mismatch byte170162/line6491: prior EXCLUDE_OBJECT_END delayed across incoming-origin travel to next layer boundary. Complete ordered diff moves only one marker occurrence, not any motion; ordinary per-layer end markers remain. Same by-layer two-object input passes discovery.

Confirmed seam is unchanged `runner::native_stream_case`, complete original Native→public API/CLI comparison with only existing generator-line normalization; raw API==CLI. Two frozen original projects: sequential target and same stock by-layer2object control. Test assertions require actual two object names/markers and their original Native transitions, never recompute geometry or implement a comparator.

## Sequence and exit criteria

1. SPEC commit, then tests-only RED commit; production byte-identical to base. Each phase runs all two-input Native double-replay preflights in official and seeded modes before any public slice (8Native slices), then2public tests/mode, themselves Native-first. Require target success/rawAPI==CLI plus true complete-output mismatch in both modes; control complete-outputPASS. Admission/Native instability/different defect→STOP, not RED.
2. Only after verified RED, consume existing pending object-change labels at the exact Rust travel boundary. No M73/formatting workaround, new helper, comparator changes or generic refactor.
3. Same two fixtures/modes GREEN:4complete-output passing outcomes and4rawAPI==CLI; both control streams unchanged RED→GREEN.
4. cargo fmt/check/workspace-all-target strict clippy and wasm32 core/vgcode/adapter checks. LinuxE2E/WASMcompile only; no Windows/macOS/browser-runtime or full-suite claim.
