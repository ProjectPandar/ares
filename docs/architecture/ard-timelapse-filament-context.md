# ARD: timelapse invocation filament ownership

Status: accepted. SPEC `20444a910` and PLAN extension `1f79cf081`; base `afca7400a`.

Owner: upstream `libslic3r/GCode.cpp::GCode::process_layer`, direct timelapse rendering at `4672-4674` and `insert_timelapse_gcode` at `5167`, source SHA `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

The parser invocation owns implicit vector indexing. Its filament ID is the writer's current logical filament at invocation, independently of upcoming scheduled tool, most-used tool or mapped physical nozzle. `PlaceholderParser.cpp:827,908-915` supplies the corresponding indexing semantics.

The Rust boundary is the existing shared `gcode_emit/timelapse.rs::append` render call. Reuse `EmitState.extruders.active_id()` and `template::render_for_filament`; do not introduce duplicate state, a new abstraction, fallback or public API.

Preserve writer refresh, render/error handling, output validation/write-back, physical placeholders and tracked position/last-Z bookkeeping in their current order. The extension ports `GCode.cpp:5455-5460` once-only traditional insertion using the existing per-physical-layer flag, reset at the existing boundary equivalent to `GCode.cpp:4653`; no other scheduling change is included. The existing emission machinery is a temporary compatibility shell around upstream GCode behavior, not an independently designed Ares pipeline.

Runtime evidence is finite Native/API/CLI non-BBL direct/traditional insertion with actual executable temperature commands and two wall-tool consumers. Preserve full comparison, raw artifacts and test-first RED. BBL/tower/safe-position completeness is deferred. Stop if the admitted failures require more than the specified render argument.

See [SPEC, failure inventory and gates](timelapse-filament-context-spec.md).
