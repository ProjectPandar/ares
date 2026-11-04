# SPEC: Repetier GCodeProcessor cached feedrate

Status: Proposed.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Selection and scope

Source inspection predicts an independent processor difference: Native dispatches
M401/M402 and restores its cached feedrate for Repetier; Ares MotionState accepts
neither command and retains the latest G1 F speed. This is not an observed RED.
The source-predicted candidate is independent of Triangles mode1, which remains
blocked on the polylines extraction.

Source search finds no existing M401/M402 or CachedPosition port. Exclude
fill_entities/support ironing, anchor_projection, skirt, modifiers, counterbore
and the processor OPTIONS/Z move index repair, which owns estimate/move-index
bookkeeping; this proposal changes only non-motion command state in motion and
a private child module. Shared test registration is limited to one dedicated
module registration.

## Upstream boundary

| Source | Behavior |
| --- | --- |
| GCode/GCodeProcessor.cpp:1825-1826 | M401/M402 command-table dispatch. |
| GCode/GCodeProcessor.hpp:408-414,812 | CachedPosition stores float XYZE/feedrate; m_feedrate is float mm/s. |
| GCode/GCodeProcessor.cpp:233-237,2503-2506 | CachedPosition reset uses FLT_MAX; each processor starts with no cached speed. |
| GCode/GCodeProcessor.cpp:5357-5366 | Repetier-only M401 caches m_start_position and current m_feedrate. It creates no motion/time block. |
| GCode/GCodeProcessor.cpp:5368-5398 | Repetier-only M402 selects cached XYZE and restores the saved feedrate when present; no G1 emulation or new vertex. |
| GCode/GCodeProcessor.cpp:2881-2888 | Before every command m_start_position is overwritten from m_end_position. M402 changes only start position, not end position; its coordinate assignment does not persist into the next G1. |
| GCode/GCodeReader.hpp:35; GCodeReader.cpp:292-315 | has_value takes a character and compares the literal character in the raw word. M402's has_value(4,p) is character4, not 'F'. Ordinary printable F words do not override the cache through this call. |
| GCode/GCodeProcessor.cpp:3879-3880 | Ordinary G1 F updates current mm/s feedrate; later motion without F consumes it. |
| GCode/GCodeProcessor.hpp:887,905,914-915 | Existing apply_config/process_file/process_buffer/finalize public processor boundary. |

Do not implement firmware GoToMemoryPosition semantics instead of this processor
source. Do not persistently restore XYZE, invent motion at M402, or interpret its
ordinary F argument as an override. Printable G-code is the bounded input domain;
control-character words and firmware behavior are deferred without new guards.

## Rust destination

Current `project_slice/gcode_emit/processor/motion.rs:231-234` delegates config
commands then discards commands outside its supported motion set. No M401/M402
handling or cached feedrate exists in motion/config_commands.rs.

After Native-backed RED, port the persistent cached-speed part into a
new private `processor/motion/cached_feedrate.rs`, with minimum MotionState
field/default/dispatch wiring. Preserve source f32 mm/s cache precision and
FLT_MAX semantics; do not add a public API, alternate estimate loop, runtime
hook, fallback, geometry change or planner synchronization. The existing state
is a temporary compatibility shell around GCodeProcessor command state.

An optional cached f32 may represent the reset sentinel if observable behavior
matches Native. Explicitly account for the sentinel rather than adding internal
validation. No change to G1 numeric arithmetic, flavor routing or existing
config commands. Keep motion.rs and every new file at most400 LOC; any necessary
mechanical extraction needs a separate exact-text proof.

Public E2E destination: dedicated ares-cli orca_parity/runner module plus its
private input helper. Use existing OrcaRunner export/slice_case caches and public
validated API/CLI full-byte comparison. No new cache recipe, native build,
producer configuration or core Native test is required.

## Included / deferred

Include single-extruder Repetier, printable M401/M402 commands in a constant
hand-authored machine-start template, cache overwrite, repeated restore,
no-prior-cache behavior and non-Repetier no-op. Scope is processor speed/time
and resulting full G-code footer/progress, not writer firmware execution.

Defer multi-tool cache lifetime, arbitrary control-character words, inches,
relative extrusion and arc combinations, parser redesign, firmware position
restore and unrelated timing/OPTIONS/Z behavior. Do not add a rejection or skip
for a deferred input. Report unrelated failures before changing the controls.

## Native-first preflight and separate RED

Fresh Native cache/artifact root for each official/seeded leg.

Reuse a tracked authored prism STL and existing Ender-3 project/preset adapter.
Override only gcode_flavor to repetier and a constant machine-start template;
one centered object, constant layers, count1 rectilinear, no tower/support/brim/
modifier/ironing/arc/scarf/SAFC. Verify exported effective settings and exact
retained template. Admission alone is insufficient proof of consumption.

Use a legal in-bed XY travel sequence: G90/M83; G1 establishing X70/Y70/F60;
M401; G1 X170/Y70/F6000; M402; G1 X170/Y170 without F. This creates a large
predicted speed/time distinction. Final preset homing/priming and ordinary
slicing remain the same in each paired input. Final concrete template/order
must be proved Native-admitted before relying on this prediction.

Eight proposed cases per mode:

| Case | Required witness |
| --- | --- |
| R0 | Repetier template without cache commands: whole-output baseline control. |
| R1 | Save low speed, change high speed, restore, next travel without F: active cache consumption versus R0. |
| R2 | Two saves separated by distinct G1 speeds: latest cache wins. |
| R3 | Two restores with intervening speed changes: cached value survives restore. |
| R4 | M402 F120 following save: printable F does not supersede the saved speed. |
| R5 | M402 before any M401, no F: reset sentinel does not change speed. |
| R6 | Marlin flavor with same M401/M402 sequence: source no-op control. |
| R7 | Save+restore without intervening speed change: inert cache control. |

Pair active inputs with matching no-command controls when necessary, preserving
all remaining template bytes. Capture Native whole streams and prove actual
nonempty motion, effective flavor and changed time/footer or progress consumption
for active cases. R4 must establish the source's unusual F behavior rather than
assume normal F units. R5/R6/R7 must retain ordinary trajectory/speed behavior.
Native rejection, substitution or a non-cache baseline mismatch is reported
before replacing any case or extending production scope.

Commit tests/inputs separately as RED: compare full ordered Native output with
public validated API and CLI output, generator-line-only normalization, exact
line counts and layers. No selected-line replacement of the full comparison.
Record whole first difference and the cache-bearing template interval; attribute
any progress/footer first difference using paired Native consumption evidence.
If all active cases match main, report NO RED FOUND and stop without production.
No Native stream/receipt/gdb product may enter git.

## Acceptance

Frozen RED comparisons and assertions remain unchanged. Both modes must pass
all eight cases, with non-cache/inert raw Ares controls unchanged against RED.
Report every unrelated changed raw stream. Gates: option_coverage whole group
both modes, strict workspace/all-target/all-feature clippy(-D warnings), fmt,
wasm32 and the self-hosted workspace. No ignore/allow/probe/guard or
obsolete-test retirement.
