# G-code processor streaming boundary

## Status

Proposed source-cited rewrite slice for review. This slice is limited to the G-code processor boundary; it does not change perimeter generation, Arachne ownership, output bytes, or viewer semantics.

## Upstream owner

The owning upstream implementation is OrcaSlicer 2.5.0 `src/libslic3r/GCode/GCodeProcessor.cpp`:

- `GCodeProcessor::process_file`, lines 3899–3918, parses the file and invokes `process_gcode_line` for each parsed line while retaining processor results and line-end metadata.
- `GCodeProcessor::process_buffer`, lines 3948–3954, parses an in-memory buffer and invokes `process_gcode_line` per line. It does not first materialize an owned vector containing every line.

The upstream boundary is processing a generated G-code buffer into processor results, not a new slicing pipeline.

## Ares destination and current issue

The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/processor.rs:185-315`, the second pass that turns emitted G-code into `ProcessedGCode` and optional move data.

Current behavior converts the complete output into `text`, eagerly clones every line into `Vec<String>`, and then retains `text`, `lines`, the output builder, line mapping, and (for processed calls) the move recorder during processing. The exact 200 mm cube measurement showed the dominant memory increase in this phase: approximately 200,500 kB at export start to 4,677,808 kB peak. Native Orca 2.5.0 peaked at 2,249,712 kB on the same centered geometry and preset.

## Included behavior

- Preserve the public `ProcessedGCode` API and all existing `GCodeProcessorResult` fields.
- Preserve per-line processing order, line endings, line numbers, line-end metadata, estimated statistics, progress comments, material statistics, printable-area validation, and optional move recording.
- Process the generated in-memory G-code according to the upstream `process_buffer` boundary, without retaining an all-lines `Vec<String>` solely to drive the second pass.
- Retain only the data needed by the processor result, line map/line-end metadata, and output construction at each step.
- Keep output bytes identical for the existing supported inputs.
- Verify the browser public API still slices a runtime-generated 200 mm cube and exposes layer data in the viewer.

## Incremental time and move ownership

The line-iterator change alone does not bound processor ownership. Extend this same rewrite slice to OrcaSlicer 2.5.0's incremental `TimeMachine` and move insertion:

- `GCodeProcessor.cpp:578-613`, `:613-759`, `:782-796`: calculate queued block times, attribute delays/cache/statistics, then erase processed blocks while preserving the planner boundary. Do not retain an entire file's `MotionBlock` values for a later schedule pass.
- `GCodeProcessor.hpp:948-955`: Marlin planner `queue_size = 64`, `refresh_threshold = 256`; `GCodeProcessor.cpp:5685-5692` enqueues a G1 block and triggers calculation. Preserve the separate Klipper junction-flush condition on those same lines and its lazy-flush behavior; do not replace it with a fixed 256-block rule.
- `GCodeProcessor.cpp:4000-4002`: final `calculate_time(..., is_final=true)` drains pending delays under the existing flavor-specific final-block rules.
- `GCodeProcessor.cpp:8103-8192`: collect actual-speed insertions only for the current flush, resize the existing result move vector by the actual insertion count, and move its affected tail backwards into final slots. Remap queued block move IDs and the pending OptionsZ marker after insertion. This is in-place growth, not a second all-moves vector with a speculative `2 * recorded.len()` reservation.

The Rust destination extends to `processor/{estimate,schedule,moves}.rs`, `processor/estimate/options_z.rs`, and `processor/moves/actual_speed.rs`. Reuse the existing flavor-specific rolling planners and their arithmetic; replace the full-file scheduling/replay compatibility shell with incremental block admission, flush attribution, and in-place move insertion. Retain exported timing/cache and line-end metadata, result moves, material/custom-stop statistics, and the existing approved distinction between official export-cache marker identity and stable public marker identity. That distinction must not require copying the complete MoveVertex buffer.

Verification retains the public browser cube test and existing processor/processed-output parity checks in both modes. Reuse the established failing WASM cube workload before this ownership change, then require slice completion and positive viewer layers matching the exported header. Compare same-input output SHA once before/after in uncommitted evidence only. Keep all timing arithmetic, G-code bytes, line mappings, marker positions, actual-speed geometry, delays, and finalization semantics unchanged. Do not add a standalone early-drop workaround or a legacy scheduling fallback.

## Explicitly deferred

- Arachne/perimeter aggregate lifetime changes (`materialize.rs` and `PreparedArachnePerimeters`).
- G-code format or ordering changes.
- Changes to `ProcessedGCode` or the web worker protocol.
- Any change to memory limits, allocator behavior, panic handling, or WASM feature selection.
- Changes to Native `-102` `ToolpathOutsidePrintableArea` behavior; that behavior is already consistent and is not part of this slice.
- General G-code file loading (`process_file`) beyond the shared source boundary; this slice processes the generated in-memory buffer.

## Verification boundary

The implementation will be verified by a public-API web E2E test that generates the cube fixture at runtime, imports it without a committed STL, and slices it successfully in WASM. The viewer layer count must be greater than zero and equal the exported G-code header's `total layer number`. RSS and WASM linear-memory curves are diagnostic artifacts only, not test assertions.

A one-time pre-fix/post-fix byte SHA comparison for the same input and preset belongs only in the uncommitted evidence directory. Existing processor/processed-output parity tests remain the byte-level regression gate in both relevant modes; any PASS-to-FAIL is a failure of this slice.
