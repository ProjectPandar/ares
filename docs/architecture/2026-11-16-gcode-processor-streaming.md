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
