# Adaptive pressure advance port

See [ARD-0104](../../architecture/ard-0104-adaptive-pressure-advance-port.md).
Port `AdaptivePAProcessor::process_layer/resetPreviousPA`,
`AdaptivePAInterpolator::parseAndSetData/operator()` and
`PchipInterpolatorHelper::setData/computePCHIP/interpolate`, plus
`GCode.cpp` PA_CHANGE emission and post-cooling/post-FanMover invocation.

Included: model parsing, flow/acceleration interpolation, clamping, prediction
rounding, standalone/loop/intra-feature markers, speed lookahead, bridge override,
activation, per-filament settings and source-owned PA lifecycle state. Destination:
private `gcode_emit/adaptive_pa`, existing motion options/path/loop modules and
filter orchestration. Preserve current multi-region routing rather than replacing
it with the old WIP emitter. The shell is temporary around the named upstream API.

Deferred to dependent work: smooth Type2 tower generation and its PA reset;
calibration-mode generation, exhaustive printer/option coverage and actual target
hardware execution. No guard or skipped active fixture counts as parity.

Acceptance: one tracer at a time, failing before implementation and passing
complete ordered G-code afterward. Expand from constant PA to nonconstant
multi-acceleration, numeric prefixes, invalid models, cooling/overhang and
activation controls. Run the official 2.4.2 single-CPU binary and separate seeded
reference; run workspace Nextest, Clippy, rustfmt and WASM compilation.
