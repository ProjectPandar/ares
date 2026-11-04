# Ironing fan requests: activated full-output rewrite slice

## Source ownership and destination

OrcaSlicer 2.4.2 `src/libslic3r/PrintConfig.cpp:3481–3490` owns the
filament `ironing_fan_speed` integer array, default/minimum -1, maximum100.
`PrintConfig.cpp:4281–4295` owns `ironing_type`, including `top` and `solid`.
`GCode.cpp::_extrude` (`append_role_based_fan_marker` and
`apply_role_based_fan_speed`, around6890–6922) emits ironing START/END.
`GCode/CoolingBuffer.cpp::parse_layer_gcode` recognizes both markers;
`apply_layer_cooldown` around880–890 consumes them and around983–1006
selects overhang, internal bridge, support interface, ironing, then baseline.
The local source package contains reference replay modifications; its
continuing-role layer marker logic is not claimed to be pristine upstream.

Rust destinations are the existing `project_slice/gcode_emit/motion/fan.rs`
producer, its options/state, and `cooling.rs`/`cooling/role_fans.rs` consumer.
Existing `pipeline` fan tests do not prove the public saved-project emitter
implements this boundary; do not route through that temporary legacy shell.

## Non-negotiable behavior

- -1 disables requests;0 is an explicit off request. The fixed ironing speed
  may be below baseline and does not depend on overhang fan enablement.
- Closed initial layers suppress START requests; an enabled request keeps its
  configured speed during the ordinary baseline ramp.
- START is re-established for a continuing role in each new layer chunk.
- Ironing END is NOT support-interface END: upstream always consumes it and
  requests fan emission, even without an active ironing request. Never retain
  inactive ironing END as a literal comment just because support END does so.
- Preserve source priority and restores. Do not change geometry, slowdown,
  minimum-PWM policy, support fan semantics, or the complete-output comparator.
- Keep the byte-oriented core free of I/O and support browser WASM as well as
  Windows, macOS and Linux. WASM compilation is not browser-runtime evidence.

## Evidence and acceptance

The historical9a7b46fc matrix reports `ironing_fan_speed` PASS, but its
activation table enables ironing only for flow/spacing/speed/pattern.
Therefore this PASS alone is not evidence of an exercised ironing fan.
Add `ironing_type=top` to that domain's fixture activation, without changing
its min/max/seeded cases or accepting fewer comparisons.

A public saved3MF test additionally uses `ironing_type=solid`: native
positive-E ironing paths must exist, native positive extrusion must resume
in another role afterwards, and modal ironing PWM must match disabled,
off, below-baseline, closed-layer, overhang-enabled and ramp controls.
Every control still compares the complete output, not only its witness.
Both official and seeded references must pass, followed by full
workspace Nextest, Clippy, rustfmt and core/adapter WASM compilation.

## Deferred behavior

No new ironing geometry, filament ironing overrides, overhang/bridge policy,
AdaptivePA, timelapse, tower or SLA implementation is included.
