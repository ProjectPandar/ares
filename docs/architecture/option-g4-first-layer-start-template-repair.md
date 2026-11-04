# G4 first-layer startup template context repair PLAN

RED and the single GREEN land together; RED never lands separately.

## Accepted Native-backed RED

Seeded and official both exit100. All five cases Native-admitted, both actual wall tools present; empty/short/forward whole PASS, reverse/duplicate DIVERGENT. Both Native and Ares actually emit first-layer tool runs[1,0] for reverse/duplicate; sorting is not the remaining mismatch. Strict firstdiff byte1089/line38: Native `M104 S223 ; set final nozzle temp`, Ares `M104 S211 ; set final nozzle temp`. Authored machine start contains unindexed `[nozzle_temperature_initial_layer]`; admitted values211/223. The earlier zero-based fixture failure remains admission-only, not this production RED.

## Owning upstream boundary

`Print.cpp:2429` constructs ToolOrdering(*this,-1,false) for by-layer. `ToolOrdering.cpp:483-514,543-579` chooses/configures initial one-based region order; `handle_dontcare_extruder:188-249` reorders and converts to zero-based. `ToolOrdering::first_extruder` supplies the scheduled initial tool to `GCode::_do_export`; `GCode.cpp:2764-2780` reuses Print ordering and chooses first_extruder except the existing Type2/priming exception. `GCode.cpp:2845-2856` sets initial/current tool placeholders, and3042-3044 installs the entire initial-temperature vector (not a scalar selected from slot0).

Crucially `GCode.cpp:3118` calls `placeholder_parser_process("machine_start_gcode", ..., initial_extruder_id)`. `PlaceholderParser.cpp:2522-2529` installs this supplied current_extruder_id in parser context; `legacy_variable_expansion:908-915` uses that context for unindexed legacy vector references. Thus the first tool determines the implicit slot for `[nozzle_temperature_initial_layer]`, without mutating the underlying vector or explicit indexes.

## Rust diagnosis and minimum proposed repair

`ares-core/src/project_slice/gcode_emit/tool_ordering.rs::ToolOrdering::initial_extruder` already selects scheduled first_extruder with the existing Type2 exception. `machine/first_filaments.rs::resolve` derives it and `InitialFilaments::insert_machine` sets scalar initial/current placeholders. `machine/start_config.rs::self_start_config` already calls insert_machine, so `{initial_extruder}` works in the observer.
But `machine.rs::append_start` at the machine-start render uses `template::render`, whose `template.rs:27-28` wrapper calls `render_for_filament(...,0)`. That independent parser argument forces implicit legacy vector access to slot0 despite correct scalar placeholders. Existing `template::render_for_filament` is the direct Rust destination for upstream PlaceholderParser::process's current_extruder_id; no new parser state abstraction is required.

Proposed minimum production change: in machine-start rendering only, use `template::render_for_filament(template, &mut config, state.initial_filaments.initial())` instead of the slot0 wrapper. Preserve template name/error capture, config vectors and explicit indexing, random context, temperature auto-insertion policy and ordering. Do not derive selection again from region0, first object, lowest tool or master_extruder_id. No temperature-specific scalar alias or fallback.

## Included / deferred and verification

Included: machine-start implicit legacy vector slot follows actual resolved initial tool; existing explicit/scalar template semantics unchanged. Selected-tool1 reverse/duplicate whole parity, emitted schedule and positive outer-wall proof remain exactly as RED. This is a rewrite of the cited parser-call context, not an Ares startup pipeline feature.
Deferred: file-start/layer/end/role/toolchange context changes, new Type2 priming behavior, physical-nozzle mapping, by-object transitions, support/non-support preferences, parsing grammar, cooling, ND/tie-break policy and other G4 domain promotion. Existing Type2 selection is reused, not redesigned or newly claimed covered. If further whole mismatch remains, retain it for a separate source-owned extension rather than expand this fix silently.

Squash docs revisions into one SPEC/PLAN record and tracer revisions into one RED, with one GREEN after RED. After the repair run fresh five-case directed seeded/official; unchanged whole comparator, assert both wall tools and actual reversal. Run entire option_coverage group both modes including tests::* because registry changed. Workspace nextest, all-target/all-feature workspace Clippy-Dwarnings, fmt, WASM; production repair then standard four-leg full with fresh caches/resources, literal candidate FAIL subset and raw input/channel audit. No ignore, assertion weakening, output normalization, Native modification or automatic retries.
