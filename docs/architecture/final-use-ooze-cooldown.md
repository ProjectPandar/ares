# Final-use ooze-prevention cooldown

## Upstream boundary

This slice rewrites OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c` behavior from:

- `src/libslic3r/GCode.cpp:481–514`, `OozePrevention::pre_toolchange`: determine final tool use at `485–496` and emit zero-temperature cooldown rather than a later-use idle or standby temperature.
- `src/libslic3r/GCode/ToolOrdering.cpp:1245`, `collect_extruder_statistics`, including the last-layer-per-extruder producer at `1267–1272` and `is_last_extrusion_layer` at `3662–3667`.
- `src/libslic3r/IMEXHelpers.cpp:88–97`: heater identity mapping, whose IMEX behavior is deferred.

## Decisions and Rust destinations

The owning state is `ToolOrdering`, not the temperature emitter. `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/statistics.rs` collects the last extrusion `print_z` for each extruder while traversing ordered layers. `tool_ordering.rs` retains that state and exposes the upstream final-extrusion predicate: an absent tool has no later extrusion, and a present tool is on its final extrusion layer when `print_z >= last_print_z - 1e-4`.

The existing layer, boundary, drive, timelapse and tool-transition contexts under `project_slice/gcode_emit/` transport the ordering reference to `tool_transition/temperature.rs`. That consumer uses the producer-owned predicate; it must not infer tool lifetime by rescanning layers at emission time.

For a final-use non-IMEX tool, the pre-toolchange branch emits zero temperature with the existing temperature command formatting and ` ;cooldown` suffix. Later-use tools retain the existing idle-temperature or standby-delta branch. The single-extruder-multi-material guard remains unchanged. Existing Ares context and temperature scaffolding is a temporary compatibility shell around the cited upstream `ToolOrdering` and `OozePrevention` concepts, not an independent slicing pipeline.

## Included and deferred behavior

Included: ordered-layer last-use collection, the upstream Z comparison, context transport, and non-IMEX final-use cooldown selection. No changes to extrusion coordinates, E arithmetic, feed rates, M73 accounting, fan commands, tool templates, footer bytes or random-number generation are part of this slice.

Deferred: IMEX shared-heater qualification and heater mapping beyond logical non-IMEX tool identity; adjacent wipe-tower planning, tool ordering, thermal scheduling and post-toolchange temperature behavior outside the cited final-use branch. No alternative heater mapping or legacy fallback is introduced.

## Failure inventory and evidence boundary

- Using idle temperature for a final-use tool leaves a nonzero cooldown command where upstream emits zero.
- Losing the ordering reference across a layer or tool-transition context prevents the final-use predicate from reaching the consumer.
- Recording the wrong producer Z or changing the upstream comparison can classify an earlier extrusion as final or miss the actual final extrusion.
- Applying the zero-temperature branch to a tool used later changes standby behavior.
- Treating logical IMEX tools as independent heaters can shut down a shared heater still needed by another tool; that behavior requires a separate witness.
- A matching labelled cooldown section does not establish equality of the complete Native/API/CLI output. Existing strict comparisons must retain differences outside that section.

The end-to-end witness in `crates/ares-cli/tests/orca_parity/runner/compatible_regions/modifier_travel.rs` compares the final-use cooldown section byte-for-byte against Native. It supplements the existing complete-output and motion assertions without relaxing their comparisons.
