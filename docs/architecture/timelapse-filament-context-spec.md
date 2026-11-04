# Timelapse template current-filament context: SPEC

Status: Accepted. The PLAN extension `1f79cf081` adds the source-owned per-layer traditional insertion flag check/set, as recorded in [the repair PLAN](timelapse-once-per-layer-repair-plan.md).

## Upstream ownership and Rust boundary

Continue the roadmap's G4 `GCode::process_layer` rewrite after the merged machine-start and layer-template invocation contexts. This ports the remaining timelapse invocation argument, using existing parser/writer ownership.

Source: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `libslic3r/GCode.cpp:4667-4675`: non-BBL layer-start timelapse passes `m_writer.filament()->id()` to `placeholder_parser_process` at `4672-4674`.
- `GCode.cpp:5145-5178`: `insert_timelapse_gcode` passes the same logical filament ID at `5167`; position clearing and last-Z adoption follow rendering at `5170-5178`.
- `GCode.cpp:3856,3895-3920`: parser invocation refreshes writer placeholders, forwards the supplied filament ID and validates/writes back outputs.
- `libslic3r/PlaceholderParser.cpp:827,908-915`: implicit legacy vector indexing follows that invocation ID; explicit indexes do not.

Rust destination: only `crates/ares-core/src/project_slice/gcode_emit/timelapse.rs::append`, currently using `template::render` at line140. That wrapper supplies slot0 (`template.rs:27-28`). The proposed repair uses existing `render_for_filament` with `state.extruders.active_id()` at this call. The shared append function serves direct and tracked insertion. Existing scheduling/position machinery remains a compatibility shell around the cited upstream owners; no new Ares pipeline, context type, public API or parser feature is proposed.

## Included behavior and deferred neighbors

Include current logical writer filament selection for implicit timelapse vectors. Preserve scalar writer placeholders, explicit indexes, invocation timing, error reporting, output validation/write-back, position clearing, last-Z adoption and physical-ID placeholders.

Defer BBL-specific fixture admission, wipe-tower timelapse, safe-position selection, physical-ID mapping corrections, by-object scheduling, new template grammar and other template owners. The shared call's source rule also applies to tracked consumers, but this slice claims runtime coverage only for admitted non-BBL direct/traditional paths. Do not expand production scope if this argument-only change cannot repair admitted cases.

## Finite Native/API/CLI admission and RED

Reuse the admitted Ender3 .4 two-PLA211/223, relative-E, by-layer SEMM/Manual map[1,1] project recipe. Two separated cubes must consume distinct actual wall filaments. Preserve vendor layer/start/change bodies and relative-E reset dependencies. Disable support, raft, brim, skirt, wipe tower, arcs and spiral. Persist exported/loaded `time_lapse_gcode` and sequence settings rather than assuming the exporter retained them.

| Case | Witness |
| --- | --- |
| Vendor control | Original recipe, no new timelapse body; unchanged complete output |
| Implicit, reverse start | Sequence[2,1], lawful `M104 S[nozzle_temperature]`; current slot1 must select223 |
| Implicit, slot0 start | Sequence[1,2]; retain slot0 behavior and observe later slot1 |
| Ordered handoff | Sequence[2,1], admitted other-layer count1 row[2,5,2,1]; distinguish writer-current from upcoming-first tool |
| Explicit indexes | Fixed `[nozzle_temperature_0]` and `[nozzle_temperature_1]` produce211/223 independently of current writer |
| Empty control | Explicit empty timelapse body retains the original recipe's output and insertion behavior |

For active cases record each emitted M104, current logical writer, physical layer, phase relative to wall extrusion, writer Z and next wall-tool visit. Require both logical tools to have positive XY/E outer-wall consumers, both current slots to be exercised, and a current/upcoming mismatch. Distinguish the direct layer-start and traditional tracked invocations from the actual Native stream; require both paths to be observed before claiming their coverage. Markers can identify blocks but cannot substitute for executable M104 consumption.

Failure inventory: exporter loss/rejection; collapsed vector or physical/logical confusion; no actual second-tool extrusion; scalar current_extruder correct while implicit vector remains slot0; upcoming-tool mistaken for invocation writer; duplicate timelapse phases misclassified; vendor reset removed; command-only witness hiding later travel changes; Native nondeterminism mistaken for Ares regression.

Native admission is mandatory before calling a mismatch RED. Save input ZIP-member hashes, loaded/exported options, Native/API/CLI raw output, independent observations and original complete comparison results in repeatable artifacts. Use the existing generator-only strict comparator and raw API/CLI equality. No assertion deletion or comparator relaxation. If phase admission or finite project parity requires unrelated repair, stop and specify it separately.

## Exit gates

1. Finite E2E tests and admitted RED in a separate commit before production edits.
2. Repair PLAN: one render invocation argument, preserving all surrounding order and behavior.
3. GREEN finite six cases and whole option_coverage in seeded/official modes; strict workspace/all-targets/all-features Clippy, fmt and wasm32 core/vgcode/wasm.

Every directed/full invocation uses a fresh Native cache/artifact root; full legs use fresh targets and unmodified control-set-v2. Native variants retain raw failures and require attribution.

Touched files: `gcode_emit/timelapse.rs`, existing CLI `tests/orca_parity/runner/compatible_regions/tool_templates.rs` registration and new `tool_templates/timelapse_context*.rs` leaves, plus these docs. Reuse admitted fixture code where visibility permits; any necessary test-only visibility change must be stated in the RED diff. No existing assertions change; each Rust file stays at most400 lines. No production overlap with current anchor, brim, profiles, multiline geometry, CI or viewer slices.
