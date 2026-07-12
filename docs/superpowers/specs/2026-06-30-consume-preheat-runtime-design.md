# Consume Preheat Runtime Design

## Goal

Consume the already-registered `preheat_time` and `preheat_steps` options as typed Ares runtime state, matching Orca's defaults and option ranges and keeping the upstream backtrace-enable predicate source-cited for the later G-code post-processing slice. This slice adds no new options, crates, dependencies, UI, CLI, WASM bindings, public API, backtrace gate execution, or preheat G-code insertion.

## Context And Approach

Ares currently stores `preheat_time` and `preheat_steps` as option registry metadata only. The keys are present in `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs` and registry tests, but no runtime accessor reads them, so invalid values can remain unobserved during G-code formatting.

Considered approaches:

- Implement Orca's full preheat post-processor now. Rejected because Orca inserts M104 and M104.1 lines through `GCodeProcessor` backtrace state over already-exported G-code, and Ares does not yet have that post-processing boundary.
- Leave both keys as registry-only metadata. Rejected because the options have concrete runtime configuration state in Orca and should fail early when supplied with values outside the already-ported option definitions.
- Recommended: add a focused `PreheatOptions` parser, then consume the parsed options from the current G-code formatting entry point without executing Orca's backtrace gate or emitting M104 or M104.1 lines.

## Upstream Boundary

Upstream OrcaSlicer boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1566-1567` declares `preheat_time` and `preheat_steps` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5757-5765` defines `preheat_time` as `coFloat`, default `30.0`, minimum `0`, maximum `120`, advanced mode, seconds sidetext, and tooltip text describing early next-tool preheat.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5767-5774` defines `preheat_steps` as `coInt`, default `1`, minimum `1`, maximum `10`, develop mode, and tooltip text describing multiple M104.1 preheat commands.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.hpp:831-832` stores `m_preheat_time` and `m_preheat_steps`.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1969-1974` copies both options from config, clamps `m_preheat_steps` to at least `1`, and sets `m_result.backtrace_enabled` when `ooze_prevention` is true, `m_preheat_time > 0`, and the printer is XL or a multi-filament non-single-extruder-multimaterial setup.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1327-1328` creates `ExportLines::Backtrace { m_preheat_time, m_preheat_steps }` for toolchange processing.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1233-1325` inserts early M104 or M104.1 commands during toolchange post-processing. This insertion behavior is out of scope.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:2497-2498` resets the processor defaults to `0.0` seconds and `1` step after processing state is cleared.

## Ares Destination Boundary

- Create `crates/ares-core/src/options/preheat.rs` for crate-private `#[derive(Clone, Copy, Debug, PartialEq)] PreheatOptions { time_s: f64, steps: u32 }`, typed parsing, value accessors, and runtime consumption. Define `SliceOptions::preheat_options()` inside this file as a crate-private inherent method via `impl super::SliceOptions`, following the existing submodule-owned accessor pattern in `crates/ares-core/src/options/infill.rs`, `crates/ares-core/src/options/raft.rs`, and `crates/ares-core/src/options/gcode_flavor.rs`.
- Update `crates/ares-core/src/options.rs` only to register the new module on an existing `option_modules!(...)` line, keeping the file at or below the 400-line split threshold.
- Update `crates/ares-core/src/options/tests.rs` only to register the focused option tests on an existing `option_test_modules!(...)` line, keeping the file at or below the 400-line split threshold.
- Create `crates/ares-core/src/options/tests/preheat.rs` for default, numeric parsing, boundary, invalid-value, and runtime-consumption tests.
- Update `crates/ares-core/src/gcode.rs` so `format_gcode` reads and consumes `options.preheat_options()?.consume_runtime()` before G-code output, making invalid values fail through the current formatting path while preserving output for valid values.
- Update `docs/roadmap.md` after implementation to record the source-cited runtime-state consumption and the remaining M104/M104.1 post-processing deferral.

## Included Behavior

1. Parse omitted `preheat_time` as `30.0` seconds.
2. Parse supplied `preheat_time` from JSON numbers or numeric strings through `SliceOptions::range_f64`, which uses the existing finite `options::parsing::parse_range_f64` parser, accepting only `0.0..=120.0`.
3. Parse omitted `preheat_steps` as `1`.
4. Parse supplied `preheat_steps` from JSON integer numbers or integer strings with a new focused inline integer parser, accepting only `1..=10`.
5. Reject non-numeric, non-integer, null, array, object, boolean, negative, zero-step, and out-of-range values with `SliceError::InvalidInput` containing the relevant option key.
6. Expose crate-private by-value `PreheatOptions::time_s(self) -> f64` and `PreheatOptions::steps(self) -> u32` for typed option tests and runtime consumption; `PreheatOptions` derives `Copy` so both accessors can be called from `consume_runtime(self)`.
7. Expose crate-private `PreheatOptions::consume_runtime(self) -> ()` with body `let _ = (self.time_s(), self.steps());`. The method has no side effects; parsing and validation happen in `preheat_options()`, and the method exists only to make the typed runtime state explicitly consumed by `format_gcode` without `#[allow(dead_code)]`.
8. Consume the parsed options from `format_gcode` through `options.preheat_options()?.consume_runtime()` so invalid values are detected before any G-code bytes are returned and the crate-private accessors are exercised in non-test code under `-D warnings`.
9. Preserve current G-code output for valid preheat values because Ares has no Orca-equivalent post-processor in this slice.

## Deferred Behavior

- Early M104 command insertion for non-XL printers.
- M104.1 command insertion for XL printers.
- Backtrace buffering, line replacement, cooldown M104 removal, toolchange-line scanning, physical-extruder remapping, layer-aware preheat temperature selection, and elapsed-time estimates from Orca `GCodeProcessor`.
- Orca's `m_result.backtrace_enabled` predicate from `GCodeProcessor.cpp:1974`: `ooze_prevention && m_preheat_time > 0 && (m_is_XL_printer || (!m_single_extruder_multi_material && filament_count > 1))`. This slice does not add a crate-private predicate method because it would have no real non-test caller before the post-processor/backtrace boundary exists and would violate `-D warnings`.
- Automatic detection of XL printer context and single-extruder-multimaterial context in a preheat-specific pipeline path.
- Orca's downstream `max(1, m_preheat_steps)` sanity clamp in `GCodeProcessor.cpp:1969-1974`; this slice rejects invalid `preheat_steps` at parse time using the already-ported `1..=10` option range instead of accepting and clamping invalid values.
- UI metadata beyond already-registered option definitions, CLI and WASM binding changes, new public API, and Orca binary E2E preheat parity.

## Tests

- Add an options test proving defaults match Orca's `preheat_time = 30.0` and `preheat_steps = 1`.
- Add options tests proving accepted boundaries and numeric string forms for `preheat_time` and `preheat_steps`.
- Add options tests proving invalid values fail with the relevant option key.
- Add a G-code formatting regression proving invalid preheat values are rejected by the current formatting path.
- Add a G-code output stability regression proving valid non-default preheat values do not introduce additional preheat M104 or M104.1 command lines relative to baseline output in this slice.

## Docs Impact

- Update `docs/roadmap.md` after implementation with the source-cited runtime-state consumption and explicit post-processor deferrals.
- No user-facing CLI, WASM, or option documentation changes are required.

## Acceptance Criteria

1. `SliceOptions::default().preheat_options()` returns `time_s() == 30.0` and `steps() == 1`.
2. `preheat_time` accepts `0.0`, `120.0`, and numeric strings within range, and rejects values outside `0.0..=120.0`.
3. `preheat_steps` accepts `1`, `10`, and integer strings within range as `u32`, and rejects `0`, `11`, fractional values, and non-integer strings.
4. `format_gcode` returns `SliceError::InvalidInput` for invalid `preheat_time` or `preheat_steps` options before producing output.
5. Valid preheat values preserve current G-code command output and do not introduce additional preheat M104 or M104.1 lines.
6. `options.rs`, `options/tests.rs`, and `gcode.rs` remain at or below 400 LOC after the edit.
7. Focused tests, formatting, clippy, WASM check, and the workspace test suite pass before commit.

## Verification Plan

- `cargo nextest run -p ares-core preheat`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
