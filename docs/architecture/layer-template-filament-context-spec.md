# Layer-change template current-filament context: SPEC

Status: SPEC REVIEW-REQUEST, worker2; based on merged `36ed482eb`.
No Native execution, test implementation or production change before SPEC approval.

## Roadmap and upstream boundary

This advances the roadmap's bounded G4 ToolOrdering/GCode rewrite beyond the merged machine-start context. The adjacent layer-template owner was explicitly deferred in [the startup repair plan](option-g4-first-layer-start-template-repair.md). It does not add an Ares-owned sequencing or template pipeline.

Source identity: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`, inspected in `${TMP_ROOT}/ares-orca-procgcode-source`.

- `libslic3r/PrintConfig.cpp:1147-1154,4415-4422` declares `before_layer_change_gcode` and `layer_change_gcode`, both strings with empty defaults.
- `libslic3r/GCode.cpp:4631-4641`, `GCode::process_layer`, renders the before-layer template before `change_layer`, passing **`m_writer.filament()->id()`** plus layer overrides.
- `GCode.cpp:4663-4685` changes layer/writer Z, then renders the after-layer template at `4682-4684`, again passing **`m_writer.filament()->id()`**. This is the writer's current logical filament, not a physical hotend, initial filament, most-used filament or upcoming scheduled tool.
- `GCode.cpp:3856,3895-3920`, `GCode::placeholder_parser_process`, refreshes writer state, calls `PlaceholderParser::process` with the supplied filament ID, validates outputs and performs writer write-back.
- `libslic3r/PlaceholderParser.cpp:827,908-915` stores that invocation ID and selects it for an unindexed legacy vector reference. Explicit legacy indexes remain independent of the invocation ID.

Rust destination: existing `ares-core/src/project_slice/gcode_emit/layer_gcode.rs::{append_before_layer_change_gcode,append_layer_change}`. Both currently refresh writer placeholders and then call `template::render`, whose wrapper in `template.rs:27-28` supplies slot0. The existing `render_for_filament` is the destination for the cited parser-call argument; `EmitState.extruders.active_id()` already owns the current writer filament. Reuse these boundaries; no new parser state, ordering algorithm, context object or public API. Existing layer emission is a compatibility shell around this upstream `process_layer` slice until its invocation context is verified.

## Included finite witnesses

Use the admitted Ender3 .4, two PLA slots211/223, relative-E, by-layer SEMM/Manual map[1,1] recipe from the merged G4 witnesses. Two separated cubes have distinct actual wall filaments. No support, raft, brim, skirt, tower, timelapse, arcs or spiral. Preserve vendor template bodies and admission dependencies; do not force an empty layer template through relative-E validation.

Append lawful layer-template commands containing `M104 S[nozzle_temperature]`, with a layer/tool marker for observation. Keep per-slot initial and ordinary nozzle temperatures equal. The command must match the current filament's admitted value211 or223; a comment/footer alone is not consumer proof. Explicit-index controls use `[nozzle_temperature_0]` and `[nozzle_temperature_1]` without changing the parser grammar or vector values.

| Case | Scope | Required distinction |
| --- | --- | --- |
| Vendor control | Original templates, first sequence[2,1] | Original full output remains equal; no added layer commands |
| Before only | Before hook, first sequence[2,1] | Implicit vector slot follows current writer, including nonzero initial slot |
| After only | After hook, first sequence[2,1] | Same invocation rule after the existing Z transition |
| Both, slot0 start | Both hooks, first sequence[1,2] | Slot0 control and later invocations across actual tool changes |
| Both, ordered handoff | Both hooks, first sequence[2,1]; admitted count1 row[2,5,2,1] | At layers2-5 distinguish previous layer's last tool from upcoming first tool; no early selection |
| Explicit indexes | Both hooks, reverse first sequence | Both indexed values remain fixed regardless of writer slot |

Require Native admission in both seeded and official modes, actual exported/loaded template/vector/plate values, and actual positive-XYE outer-wall extrusion for both logical filaments. Establish persistent tool state from the admitted start/change sequence and corroborating wall-role visits, not only the layer marker. For every hook record its physical layer, current logical filament, emitted M104, explicit values, writer Z and next wall/tool visit. Require both current slots to occur in the paired-hook witness and at least one admitted boundary with writer-current different from upcoming-first. If that boundary is absent, report admission failure and revise the fixture; do not assume the schedule or invent expected output.

## Failure inventory before tests/code

- Export may discard template edits, collapse a filament vector, or change plate sequence/map ownership. Persist loaded typed values, ZIP-member hashes and effective exported options; rejected admission is not production RED.
- SEMM physical hotend0 may be mistaken for logical filament1. Verify two actual wall-tool consumers and the logical writer context independently.
- The scalar `{current_extruder}` can be correct while the parser's implicit slot remains0. Compare the real M104 value with the independently observed writer state.
- A layer's next tool may differ from its current writer. Observe invocation before subsequent selection, rather than using the first upcoming role or cached initial tool.
- Explicit indexes, vector contents, layer numbers, Z movement, template order or assignment write-back may change accidentally. Preserve the current renderer/error/write-back order; new tests do not replace or weaken existing template tests.
- Native path-order instability may cause a whole-file mismatch. Retain raw inputs/streams, Ares/Orca SHA and actual timing; use fresh diagnostic runs only under coordinator direction. No changed comparator, motion stripping, golden output or automatic exception.

## Ownership, exclusions and architecture decision

See [ARD: invocation context](ard-layer-template-filament-context.md). Production claim is **only `layer_gcode.rs`**. Read `template.rs`, `motion/state.rs`, `motion/writer_state.rs` and `layers/boundary.rs`; do not modify them. No `machine.rs` change.

New tests live under `ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/layer_context{,_fixture,_observation}.rs`, registered only in the existing253-line `tool_templates.rs`. This avoids worker1's top-level `orca_parity.rs` registration and the shared400-line `runner.rs`. No option inventory/plan/sweep promotion or existing assertion edit. Every changed Rust module must stay at most400 lines.

Read-only claims audit found no planned source/test overlap with worker1 APA/anchor (`02661ff52`), worker3 inner brim (`ec7be9f54`), worker6 multiline geometry (`29619b591`), worker5 profiles catalog split, worker4 CI/oracle work, or the viewer/web slice. Recheck live claims before implementation; the shared roadmap entry is documentation coordination, not a claim on another runtime owner.

Defer timelapse, wrapping, file-start, by-object templates, role hooks, filament start/end/toolchange, parser grammar, malformed/empty-script errors, new write-back behavior, first/most-used preference, Auto map grouping, physical-nozzle policy, support/raft/tower and arbitrary option-domain completeness. No fallback, new guard, suppression, debug instrumentation or Native modification.

## Test-first plan and exits

1. After SPEC approval commit finite E2E tests before any production edit. Use live original Orca, public Ares API and real Ares CLI on identical saved3MF inputs; compare whole outputs using the existing generator-only strict comparator. Retain input ZIPs, Native command receipts, stdout/stderr/exit channels, raw G-code, manifests and hook/tool/M104 observations in repeatable artifacts. A missing Native effect, CLI rejection or harness failure is an admission failure, not RED/GREEN.
2. Submit actual admitted RED and a source-cited repair PLAN for separate approval. The suspected minimum repair is passing the existing active logical filament to the two layer render calls; this SPEC does not authorize production changes in advance of that RED/approval.
3. After repair approval preserve every assertion, vector and writer/error transition. Run fresh finite directed controls plus the entire option_coverage group in both modes on the reported HEAD, then strict workspace/all-target/all-feature Clippy `-D warnings`, fmt and core/vgcode/wasm wasm32 checks. Request the allocated jobs2/threads2 standard-env workspace slot; do not start from an inferred vacancy.
4. Every nextest command explicitly uses threads2/jobs2. Every directed/full run has a new Native cache and artifact root; candidate full legs additionally have separate fresh targets. After exact code APPROVED and explicit full allocation, run candidate-only seeded/official concurrently with control-set-v2 unchanged, start load<30. Preserve any failure and seek exact coordinator adjudication; no automatic full rerun. SPEC approval alone does not authorize full, push or merge.

Exit is this bounded `GCode::process_layer` invocation-context port with admitted whole-file E2E and actual writer-slot consumers, not global G4/template/printer parity. Core remains byte/API oriented and platform-neutral for WASM, Windows, macOS and Linux.
