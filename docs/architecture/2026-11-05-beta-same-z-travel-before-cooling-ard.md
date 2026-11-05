# Same-Z travel before layer cooling

## Source boundary

Rewrite OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`, `src/libslic3r/GCodeWriter.cpp::travel_to_xyz`:

- Lines 1001–1095 handle a deferred lift.
- Lines 1096–1105 handle `!force_z && !will_move_z(point.z())`, update lift bookkeeping, establish clear position and return XY-only travel.
- Lines 1123–1138 split an unclear-position travel into XY and Z only after the earlier branches did not return.

`GCode.cpp:10500–10504,10528–10532` supplies the travel destination and layer-change force-Z argument. `GCode.cpp:6479–6488` legitimately invalidates the clear-position flag after a nonempty timelapse template.

## Rust destination

`crates/ares-core/src/project_slice/gcode_emit/motion/path/start_travel/approach/move_emit.rs::emit_approach_move` owns the corresponding travel-shape branches. Its ordinary `first_position` branch must honor the upstream same-Z return before emitting a separate Z leg. Reuse `approach.rs::travel_will_move_z` and the existing writer/lift state.

The existing approach-shape chain is a temporary compatibility shell around upstream writer concepts, not an independently designed travel pipeline.

## Decisions

- An unclear position alone does not require a Z restatement. Preserve the source ordering of deferred-hop, same-Z and actual-Z-change branches.
- Preserve force-Z/layer-change requests, genuine initial positioning, deferred hops, sloped/contoured travel and actual Z changes. An unknown writer Z must not become known through a nominal-Z fallback.
- Keep the source lift bookkeeping and existing tolerance; do not introduce numeric adjustments or infer equality from formatted commands.
- Keep timelapse position invalidation, template bytes, object labels, layer-start exclusion and legitimate acceleration unchanged.
- Correct the producer before cooling consumes the G-code. Do not filter commands in a comparator or after cooling.

`CoolingBuffer.cpp:460–500` resets relative E and uses the E delta when XYZ length is zero. An additional zero-XYZ command can therefore contribute time. Preserve this parser behavior, slowdown sorting/arithmetic and the integer rewrite at `CoolingBuffer.cpp:995`; compensating there would diverge from upstream.

## Included and deferred behavior

Included is the same-Z branch ordering for the ordinary post-inline-photo approach, retaining existing force-Z and lift guards. The core remains platform-neutral.

Deferred are custom-G-code parser/cache invalidation, additional template interpretation, new timing architecture, remap features, scarf/segmented/arc changes and unrelated feedrate discrepancies.

## Verification boundary

Compare the original OrcaSlicer and public byte-input API on identical project bytes. Verify the literal final-layer feedrate prefix and the inline-photo-to-top-surface travel without an additional Z line. Keep complete-output equality and existing first-travel/photo assertions. Section equality does not imply whole-output parity.
