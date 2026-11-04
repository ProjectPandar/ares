# ARD: Layer templates use current logical writer filament

Status: proposed; base `36ed482eb`.

## Upstream decision boundary

OrcaSlicer `8500fcd` `libslic3r/GCode.cpp:4639-4641,4682-4684`, within `GCode::process_layer`, passes `m_writer.filament()->id()` to both layer-template invocations. `GCode::placeholder_parser_process:3856,3895-3920` supplies it to `PlaceholderParser::process` after writer refresh; `PlaceholderParser.cpp:827,908-915` uses it for unindexed legacy vector expansion.

## Non-negotiable decisions for this slice

- Invocation context is the current logical writer filament at each existing layer hook. It is not the physical nozzle, initial tool, most-used tool or next scheduled tool.
- Keep the before hook before `change_layer` and the after hook after it. Preserve scalar placeholders, vectors, explicit indexes, RNG, error capture and assignment write-back.
- Reuse `EmitState.extruders.active_id()` and `template::render_for_filament`. No new context plumbing, tool selection, parser algorithm or public API.
- Runtime edits stay in `ares-core/src/project_slice/gcode_emit/layer_gcode.rs`. The existing module is the compatibility shell for the cited upstream slice, not an Ares-owned pipeline extension.
- Establish Native-admitted public API/CLI E2E RED before a production repair. Actual M104 consumption and wall/tool execution are required; comments or settings footer alone are insufficient.
- Keep full-output comparisons and existing assertions. Native instability remains visible and needs a specific decision; it does not justify a new normalization or fallback.
- Do not extend this port into APA/anchor, brim, profiles, geometry, CI/oracle or viewer files. Adjacent template call sites remain deferred.

The complete included/deferred behavior, failure inventory, finite cases and gates are in [the SPEC](layer-template-filament-context-spec.md). This is a source-cited `libslic3r` rewrite under [ARD-0020](ard-0020-future-milestone-rewrite-gate.md).
