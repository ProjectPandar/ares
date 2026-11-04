# GCodeProcessor time-estimation parity

## Upstream ownership and Rust boundary

This is a source-cited rewrite correction of OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`, not a new Ares timing model:

- `src/libslic3r/GCode/GCodeProcessor.hpp::TimeBlock`, `TimeMachine`, `TimeProcessor` (TimeBlock447, TimeMachine482, calculate_time580).
- `GCodeProcessor.cpp::TimeBlock::calculate_trapezoid` (255), `TimeMachine::calculate_time` (419), motion/limit command handling including `process_M204` (5207), `process_filament_change` (5512), and `run_post_process` (981).
- Rust destination: `ares-core/src/project_slice/gcode_emit/processor/{motion, schedule, estimate, filament_change, export_lines, time}.rs` and their existing planner/limits/export submodules. Existing types are compatibility shells for these upstream concepts; correct only the first demonstrated mismatching upstream-owned behavior.

The final implementation boundary remains conditional on observable first-divergence evidence. A total-duration mismatch alone does not authorize changing the planner, limits, delays or BBL exporter indiscriminately.

## First bounded slice: SYNC additional time

`GCodeProcessor.cpp:5430–5436::process_SYNC` reads the command's float T value and calls `simulate_st_synchronize(time)` without a BBL-only gate. The existing Rust `estimate.rs::synchronizes_planner` recognizes SYNC T, but `delays.rs::command_delay` supplies no additional time. This is an observed source mismatch, not yet a claim that it explains the entire complex X2D gap or precedes every other possible divergence.

Destination of this first correction: existing `processor/delays.rs` command-time selection, carried through the existing `FlushEvent`/`scheduled_times` additional-time contract. Preserve upstream float parsing and the existing planner flush. Do not change limits, trapezoids, filaments or M73 insertion while correcting this branch.

Minimal prewritten public E2E matrix: use repository cube10 and a fixed custom-code bracket with ordinary motion before/after SYNC T5, on Klipper K1, Marlin Ender-3 and BBL X2D. Validate the applied custom code, require Native success and complete output, and compare ordered bytes including timing metadata/M73. Prewrite matching no-SYNC and SYNC T0 controls before production edits. Keep M73 enabled in admitted progress cases. The fixture must not depend on bridge or another branch.

The remaining complex-X2D residual is deferred to a separately evidenced next slice; do not bundle a speculative second correction. Original external files are forensic corroboration, not test fixtures.

## Contract

Given identical effective printer configuration and complete G-code motion/command input, reproduce upstream normal-mode block admission, machine-limit updates, speed/junction selection, reverse/forward planning, trapezoid times, flush/extra-time attribution and accumulated snapshots. Preserve upstream numeric precision and accumulation order. Derive model/preparation/total time and BBL M73 P/R from those same upstream-compatible results.

- Included: the proven mismatching source slice, its required state/configuration dependencies, and BBL progress export if it is independently shown to diverge after time-accounting parity.
- Deferred: bridge geometry/filament ordering, placeholder expansion, lift/object/timelapse emission ownership, silent-mode work not required by the reproducer, independent timing heuristics and unrelated processor cleanup.
- No calibration multiplier, approximate duration tolerance to hide13%, metadata rewriting, M73 suppression, golden normalization changes or legacy fallback.
- Do not infer a universal BBL defect from one complex X2D model. Establish non-BBL Klipper and Marlin controls first, then BBL/X2D and material-transition scope.

## Admission and failure matrix before implementation

Use repository-owned minimal geometry (initially cube10.stl), current main, declared actual Native image/resources, seeded byte authority, exact effective settings verification and complete artifacts. External X2D narrow models and their outputs are forensic evidence only, never committed fixtures or test expectations. Do not require the separate bridge port or execute another unmerged production branch as current main.

Potential failures to discriminate:

1. G-code input sequences/configurations differ: generated-stream comparison alone cannot establish an estimator bug. Separate emit differences from same-input processor timing.
2. Incorrect model/preparation partition versus actual total-time discrepancy.
3. Omitted/duplicated delays, tool changes or planner flushes; physical/logical filament mapping and BBL command gating.
4. Different initial/evolving axis feedrate, acceleration, jerk/junction deviation or firmware-specific units.
5. Block-count/arc-length or entry/cruise/exit-speed/trapezoid differences, including retained planner windows.
6. Same accumulated time but different snapshot/G1-ID mapping or BBL M73 P/R export.
7. Invalid fixture admission, unsupported bridge/placeholder behavior, failed Native production or an incomplete oracle dump: not a valid timing RED.

## Evidence sequence and TDD gate

1. Reproduce minimum single-material Klipper/Marlin and BBL controls; verify actual case artifacts/status and firmware flavor, not just a permissive harness process exit.
2. Reduce a valid non-bridge timing failure using the same geometry/configuration across appropriate cases. Candidate dimensions are material transitions, motion duration and machine-limit/BBL command policy; change one dimension at a time. Keep M73 enabled for the admitted progress-output RED. A disabled-M73 control separates export from estimation without replacing the RED.
3. Align identical command inputs/configuration at block admission; locate the earliest difference in state/limits/block data, then planned speeds/trapezoids, per-block time, cumulative total and export snapshots. Use existing test/oracle interfaces or independent Native test tooling; no production probes. Oracle success and complete input/configuration must be established before trusting a dump.
4. Prewrite an independently observable RED and passing neighboring controls before production edits. Preserve full ordered-byte assertions and make block/time evidence repeatable. A failure caused by unrelated production ownership is not relabeled as this RED.
5. Implement only the source-bounded correction and remove only orphans it creates. Modules must remain≤400LOC. Run workspace nextest, strict all-target/all-feature Clippy, fmt and WASM, current-base full seeded/official controls and unchanged complete common streams. Official ND may be classified only under the documented two-condition harness rule; no retries to erase genuine failures.

Exit: the admitted minimal timing/progress RED and its neighboring controls pass, the first upstream divergence is eliminated, and all applicable gates are verified.
