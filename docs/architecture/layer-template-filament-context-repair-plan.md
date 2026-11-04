# Layer-template current-filament context: repair PLAN

## Source owner and admitted RED

This is the bounded `libslic3r/GCode::process_layer` invocation-context rewrite specified in [the SPEC](layer-template-filament-context-spec.md) and [its ARD](ard-layer-template-filament-context.md).
Upstream identity remains OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

`GCode.cpp:4633-4641` renders `before_layer_change_gcode` with `m_writer.filament()->id()` before `change_layer`. `GCode.cpp:4676-4684` renders `layer_change_gcode` with that same current logical writer filament after `change_layer`. `GCode::placeholder_parser_process` at `3856,3895-3920` forwards the supplied ID to the parser; `PlaceholderParser.cpp:827,908-915` uses it for unindexed legacy vector references.

The six finite Ender-3/PLA211/223/by-layer/SEMM/Manual[1,1] Native/API/CLI witnesses preserve the exported relative-E reset, original vendor bodies, effective filament vectors, object wall ownership and configured plate sequences. Vendor control and explicit-index control pass strict whole-output comparison. Before-only, after-only, paired slot0-start and ordered-handoff cases fail the existing strict comparator through both public API and real CLI. Native emits `M104 S223` for current logical filament1; Ares emits `M104 S211`, even though the independently observed start/change sequence and `{current_extruder}` scalar both identify filament1. Explicit `[nozzle_temperature_0]`/`_1` stay211/223.

The handoff additionally admits physical layer2 with writer-current0 and upcoming outer-wall tool1. Both logical tools have actual positive-XYE outer-wall consumers. This distinguishes current writer context from a prospective first-tool selection; the correct repair must preserve this ordering.

API and CLI raw streams are identical for all six cases in both modes.

| Mode at RED | Native admission | Strict vendor/explicit controls | Strict implicit cases, API and CLI | Nextest result |
| --- | --- | --- | --- | --- |
| Seeded, fresh cache/root | 6/6; all Native behavior assertions pass | 2/2 PASS | 4/4 DIVERGENT | 1 aggregate test FAIL, exit100, 12.401s |
| Official, fresh cache/root | 6/6; all Native behavior assertions pass | 2/2 PASS | 4/4 DIVERGENT | 1 aggregate test FAIL, exit100, 9.580s |

Both commands used `--no-fail-fast`. All twelve CLI invocations exited0 and retained complete streams. The remaining974 tests were filtered out, not run. Strict test-target Clippy `-D warnings` and fmt/diff checks pass; all four changed Rust files are at most400 lines. No assertion or comparator has been removed or relaxed.

## Minimum repair

Modify only `ares-core/src/project_slice/gcode_emit/layer_gcode.rs`:

1. In `append_before_layer_change_gcode`, replace `template::render(template, &mut config)` with the existing `template::render_for_filament(template, &mut config, state.extruders.active_id())`; cite `GCode.cpp:4639-4641` at the call.
2. In `append_layer_change`, make the same argument substitution; cite `GCode.cpp:4682-4684` at the call.

`motion/extruder_registry.rs::active_id` is the existing current logical writer registry, including SEMM's distinction from physical nozzle0. `motion/writer_state.rs::refresh_writer_state` already refreshes the scalar and filament aliases from this registry. The missing behavior is the parser invocation ID: `template.rs:27-28` currently delegates its default wrapper to slot0. Reuse the existing filament-aware renderer directly; do not change that shared wrapper or create new context plumbing.

Keep both call positions, configuration construction and overrides, writer refresh, vector contents, explicit indexes, RNG, error recording, processed-text emission and assignment write-back intact. Do not select a tool, use the initial/most-used/upcoming filament, parse the output for context, or add a guard/fallback. The existing layer module remains the temporary compatibility shell around this cited upstream slice.

Production file remains below400 lines. Test files stay as committed, with registration in `runner/compatible_regions/tool_templates.rs`; no shared top-level registration edit. No changes to `machine.rs`, shared template/parser/writer/boundary modules, other template owners, option inventory/sweep, profiles, brim, infill/geometry, CI/oracle or viewer files.

## Verification gates

- Make the minimum production change and run fresh finite seeded and official E2E. All six cases must pass whole-output comparison through API and CLI, with Native admission, dual actual walls, both current slots and the admitted current/upcoming handoff retained. Keep every assertion.
- Run the full option_coverage group in both modes with distinct fresh Native caches and artifact roots. No root/cache reuse across runs.
- Run strict workspace/all-targets/all-features Clippy `-D warnings`, fmt, and wasm32 checks for core/vgcode/wasm. Run `nextest --workspace --no-fail-fast` in the standard environment.
- Run candidate-only seeded/official full with unchanged control-set-v2, each leg with a fresh cache/root/target. Preserve all failures; no automatic full retry or exception.
- This PLAN is not a claim of global G4/template parity or a new clean full baseline.

The repaired core remains a platform-neutral byte/API implementation for WASM, Windows, macOS and Linux. Adjacent timelapse/by-object/role/filament/machine-end template invocation contexts and parser grammar/errors remain deferred as in the SPEC.
