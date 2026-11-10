# Unconditional machine filament-load time in the G-code processor

## Decision

The G-code processor must pass the configured `machine_load_filament_time` to its filament-change accounting for every processing pass. The value must not be replaced with `0.0` based on `post_process`.

This follows the pinned OrcaSlicer source. Native initializes `filament_load_times` from `machine_load_filament_time` at `src/libslic3r/GCode/GCodeProcessor.cpp:3217` and `:3558-3560`. `get_filament_load_time` at `:7991-7995` consults only `extruder_unloaded`; it has no `post_process` condition. Native export invokes `finalize(true)` from `src/libslic3r/GCode.cpp:3011`.

## Current Ares boundary

`crates/ares-core/src/project_slice/gcode_emit/processor.rs:118-151` exposes `process_gcode` and currently chooses the estimate argument as follows:

- `post_process == true`: configured `machine_load_filament_time`;
- `post_process == false`: `0.0`.

That conditional is Ares-specific and does not correspond to the Native processor state model. The implementation slice shall replace it with the configured value in both branches, without changing the function signature or callers.

The internal `processor::process` function already receives the configured value from `crates/ares-core/src/project_slice/gcode_emit/orchestration.rs:374-385`. The normal slice/export path therefore already follows the upstream rule. The defect is limited to the public reprocessing entry point's argument selection.

## Call-site inventory and impact

### Public `process_gcode` callers

The repository callers and their current purpose are:

- `crates/ares-core/tests/gcode_processor_api.rs`: public API behavior checks, all with `post_process=false`.
- `crates/ares-core/tests/gcode_processor_end_gcode_delays.rs`: processor delay behavior, `post_process=false`.
- `crates/ares-cli/tests/orca_parity/runner/preheat_normal_move_durations.rs`: Native-backed Normal move-duration oracle, intentionally `post_process=false`.
- `crates/ares-cli/tests/orca_parity/runner/wipe_tags.rs`: processor move-tag inspection, `post_process=false`.
- `crates/ares-cli/tests/orca_parity/runner/klipper_time_estimator.rs`: time-estimator comparison, `post_process=false`.
- `crates/ares-cli/tests/orca_parity/runner/processor_migration/roles.rs`: role and processor migration comparisons, `post_process=false`.
- `crates/ares-cli/tests/orca_parity/runner/processor_migration/stops.rs`: stop/custom-time comparison, `post_process=true`.
- `crates/ares-cli/tests/orca_parity/runner/processor_migration.rs`: processor migration comparison, `post_process=true`.

No production caller invokes the public `process_gcode` wrapper. The production slice path calls the internal `processor::process` directly and already supplies the configured load time.

### WASM and vgcode

`crates/ares-wasm/src/profiles.rs:143-165` reaches `ares_core::slice_with_profiles_processed_validated`, whose slice orchestration reaches the internal `processor::process` call described above. It does not call the public `process_gcode` wrapper. `crates/ares-wasm/src/moves.rs:28-44` converts the resulting `ProcessedGCode` through `ares-vgcode`.

`crates/ares-vgcode/src/convert.rs:15-88` consumes `GCodeProcessorResult`; it does not select or gate machine-load time. Once the processor publishes the corrected move duration, the existing converter and WASM viewer automatically expose it in Normal time fields. No vgcode or WASM API/schema change is required.

## Runtime effect

For the retained X2D two-prism fixture, Native row 7 (`T1 H-1`, G-code id 924) publishes:

- load: `29.0 s`;
- unload: `0.0 s`;
- tool change: `0.0 s`;
- Normal move duration: `29.0 s`.

With the existing Ares `post_process=false` call, the first observed Ares value was `10.5 s`. A probe-only run using `post_process=true` matched all 4020 Normal duration bit values, demonstrating that the remaining discrepancy is the wrapper's zero-load argument, not the duration comparator or vgcode conversion. The production fix must keep the preheat test call at `post_process=false`; only the internal load-time argument changes.

## Verification and compatibility requirements

1. Apply only the one-line argument-selection change in `processor.rs`.
2. Keep `preheat_normal_move_durations_from_x2d_geometry` on `post_process=false`.
3. Run official and seeded preheat modes and require all ordered move identities and Normal duration IEEE-754 bits to match, including zero and internal-only vertices.
4. Run the full parity run using the existing official and seeded inventories. Compare the exact baseline failure inventory and require P→F = 0; any changed unrelated failure must be classified rather than hidden.
5. Run `cargo fmt --all --check`, `git diff --check`, and strict workspace Clippy after implementation.
6. Confirm G-code bytes remain unchanged for the API/CLI cases whose only observed difference should be processor result timing. The normal slice/export path already passed the configured load time before this change, so its G-code output is expected to remain byte-identical.

Deferred behavior remains unchanged: Stealth duration publication is not expanded, preheat injection is not redesigned, and the public Native-backed test remains the verification boundary.
