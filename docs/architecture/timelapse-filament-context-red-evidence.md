# Timelapse current-filament context: admitted RED and repair PLAN

SPEC `20444a910` approved; base `afca7400a`. This commit contains only E2E tests and documentation. Production `gcode_emit/timelapse.rs` is unchanged.

## Admission and reproduction

Final RED root: `/tmp/worker2-timelapse-red.1zBoNI`; each mode has its own fresh Native cache and artifact root. Reproduce using the unmodified `${TMP_ROOT}/ares-parity-control-set-v2.sh` with seeded/official, jobs2/threads2, and nextest filter `test(timelapse_templates_render_current_logical_filament_vectors)`. Allocate fresh cache/root for every invocation; do not reuse the recorded cache.

The aggregate E2E runs six complete projects per mode. Native slicing succeeds, core API succeeds and CLI exits0 for all six. API/CLI raw outputs are equal. Each active Native stream has50 physical layers and100 executable timelapse blocks:50 direct layer-start and50 traditional tracked interludes. Both logical tools consume positive-XYE outer walls; both current slots occur. Current/upcoming wall-tool differences occur50 times in ordinary controls and54 in the handoff control.

| Case | Complete Native/API/CLI result per mode | Independent witness |
| --- | --- | --- |
| vendor | PASS | Empty original timelapse, both actual wall tools |
| implicit-reverse | DIVERGENT | First current writer1: Native M104223, Ares211 |
| implicit-slot0 | DIVERGENT | Slot0 start preserved, later current1 still rendered as211 |
| handoff | DIVERGENT | Current/upcoming tool distinction, implicit slot1 mismatch |
| explicit | DIVERGENT | Executable explicit211/223 correct, additional Ares timelapse block |
| empty | PASS | Explicit empty string and unchanged vendor complete output |

Each mode exits100 at the aggregate's final failure assertion: two finite controls PASS and four admitted complete-output failures. This is one failed nextest test with four failing cases, not four failed nextest tests. The complete comparator remains `golden::compare_ordered_bytes_generator_only`; no assertions or comparator rules are removed or relaxed.

There are two independent symptoms. The implicit reverse/handoff first difference is line68, M104223 Native versus211 Ares. Every active case also has100 Native versus150 Ares blocks: Ares emits a traditional block for both cube/tool visits. The explicit control first differs at line212: Native resumes travel, whereas Ares inserts a third block. This is present before the proposed production repair, so render-context correction alone may not make the finite suite GREEN.

Artifacts include final identical-consumer3MF, original export/binding application proofs, loaded typed timelapse string, unchanged non-settings ZIP-member hashes, Native producer/request receipts, Native/API/CLI G-code, CLI stdout/stderr/exit, comparison manifest and independent hook observations. `red-evidence.json` joins each artifact input SHA to a real Native slice receipt and checks the receipt's output SHA against `orca.gcode`; disk and memory inputs are thereby verified. Earlier compile errors and the initial unsynchronized-disk admission attempt are retained in inbox and excluded from admitted RED.

Tests reuse the existing admitted layer fixture through two visibility-only edits (`layer_context.rs` fixture module, `layer_context_fixture.rs::prepare`), scoped to the `tool_templates` parent. Existing test bodies/assertions are untouched. The new wrapper owns only the timelapse setting, preserves other ZIP members, and writes the final bytes to the Native project path. Test-target strict Clippy and fmt pass; every changed Rust file is at most400 lines.

## Preapproved minimum repair PLAN

At the sole render call in `gcode_emit/timelapse.rs::append`, replace `template::render(source, &mut config)` with `template::render_for_filament(source, &mut config, state.extruders.active_id())`.

Source owner: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `GCode.cpp:4672-4674,5167` passes `m_writer.filament()->id()`. Keep writer refresh, render/error handling, validation/write-back, output and position/last-Z order unchanged.

Run the unchanged six-case E2E in both modes using fresh roots/caches. If it still fails, preserve the trial and report the remaining difference; do not change scheduling or expand the production scope. Only a successful finite repair proceeds to whole option_coverage, strict/fmt/wasm, code review and coordinator-assigned heavy gates. No heavy run or push is authorized by this RED report.
