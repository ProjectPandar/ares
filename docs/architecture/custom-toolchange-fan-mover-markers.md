# Custom toolchange FanMover marker lifecycle

## Upstream ownership

This slice rewrites OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c` non-tower custom change-filament emission and its FanMover marker protocol.

- `src/libslic3r/GCode.cpp:1093–1100`, `wrap_custom_gcode_for_fan_mover`: wrap only when `fan_speedup_time != 0 || fan_kickstart > 0`; otherwise return the custom text unchanged.
- `GCode.cpp:11197–11223`, `GCode::set_extruder`: render the custom change-filament text, ensure its final newline, emit its wrapped form, then emit `;_FORCE_RESUME_FAN_SPEED`. Subsequent tool/Z bookkeeping reads the unwrapped text.
- `GCode/FanMover.cpp:453–457`: recognize `;_FAN_MOVER_SKIP_`, set `m_is_custom_gcode` according to the START prefix, and set `time = -1`.

## Marker retention and consumption

The comment at `FanMover.cpp:454` describes markers being dropped, but it does not describe every producer output path. The executable consumer branch at `:469–500` inserts a record into the buffer only when `time >= 0`. A skip marker has negative time, so it is not inserted. The ordinary buffer-output loop at `:526–544` and the final `process_gcode(flush)` loop at `:54–62` write buffered raw records with newlines; neither can restore a marker that was never inserted. Preserve the existing state update and flush ordering, not an unconditional output-comment filter.

Initial tool selection uses a different path: `GCode.cpp:4490–4495` calls `file.write(this->set_extruder(initial_extruder_id, 0.))` directly. The resulting wrapped custom text does not traverse the layer FanMover and its marker records remain in the output. Layer text traverses cooling, FanMover and output filters at `GCode.cpp:5176–5218`; the FanMover activation predicate at `:5197–5209` matches the producer gate. Thus initial marker retention and in-consumer marker omission are both upstream behavior.

## Rust boundary and decisions

- `crates/ares-core/src/project_slice/gcode_emit/tool_transition/change.rs::append` emits the conditional START/END wrapper around the rendered custom text only, before the existing FORCE_RESUME record. Return the unwrapped text and keep template writer-state application and last-motion-Z inspection unchanged.
- `crates/ares-core/src/project_slice/gcode_emit/fan_mover.rs::FanMover::process_line` recognizes the same prefix and updates custom-region state. Returning for that marker implements the upstream negative-time exclusion from both buffer insertion and ordinary flushing, without changing scheduling for other record kinds. Existing flush functions remain responsible for buffered-record output.
- `gcode_emit/layers/preamble.rs::emit` returns the byte endpoint after initial preamble emission. `gcode_emit/layers.rs` and its included `layers/drive.rs` record that endpoint as a plain byte offset. `gcode_emit/orchestration.rs` starts only the FanMover consumer span there, preserving the earlier prefix verbatim. This ports the direct initial write at `GCode.cpp:4495` before `process_layers` at `:5192–5220`; it does not reorder initial emission or change tower initialization.
- `gcode_emit/layers/fan_setup.rs::gate` already provides the matching resolved configuration predicate. No new activation policy is introduced.

The existing Ares emitter and FanMover are temporary compatibility shells around the cited upstream concepts; this slice replaces the missing marker/state transport rather than adding an independent G-code pipeline.

## Included and deferred behavior

Included: non-tower change-filament wrapper production, the exact gate and marker bytes, consumer custom-state transition, negative-time omission, raw-rendered-text bookkeeping, and direct initial-output retention.

Upstream also writes this initial preamble outside the cooling and pressure-advance stages (`GCode.cpp:4495` versus the `process_layers` pipeline at `:5176–5220`). Matching those stage boundaries is deferred: retain the current Ares cooling and adaptive-PA spans exactly; change only which bytes enter FanMover.

Deferred: `WipeTowerIntegration::append_tcr` at `GCode.cpp:1464`, tower output/layout, other custom script categories, motion routing, first-travel and timelapse policy, footer/statistics/config export, fan interpolation or kickstart scheduling, E/feed/M73 arithmetic, and IMEX/shared-heater behavior. Do not alter FORCE_RESUME handling or existing manual-change suppression.

## Failure inventory

- Omitting the wrapper causes the initial custom-text prefix to differ from Native even when tools and extrusion commands agree.
- Wrapping with both gate inputs disabled adds marker records absent upstream.
- Failing to recognize a marker inside active FanMover leaves custom-region state unset and can allow fan-command movement inside custom text.
- Buffering negative-time markers leaks records upstream omits; deleting every marker globally removes records the direct initial-write path retains.
- Inspecting wrapped rather than raw text changes the upstream writer/tool/Z boundary.
- Moving FORCE_RESUME inside the wrapper changes event ordering.
- A matching labelled prefix does not establish complete-output parity; strict residual comparisons remain independent.

The public end-to-end boundary compares Native/API/CLI output from the existing J1 role2 fixture, preserving its complete-output and tool-usage assertions. A labelled initial-prefix comparison and a disabled-gate control isolate the marker protocol without replacing strict whole-output checks.
