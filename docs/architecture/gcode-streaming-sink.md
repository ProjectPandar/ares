# Streaming Chunked G-code Sink (`GCodeSink`)

## Upstream Boundary

Pinned OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `OrcaSlicer/src/libslic3r/GCode.hpp:411-437`: `GCode::GCodeOutputStream` — streaming destination passed into `GCode::_do_export`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2850-3060, 3407-4961`: `GCode::do_export` and `GCode::_do_export` — emits G-code sections (file start, header, thumbnails or BBL config block, widths, executable block, end G-code, non-BBL config block, and filament/time statistics) into an output stream using a `.tmp` staging file that is cleaned up on error, then runs `GCodeProcessor::finalize(true)`.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.hpp:603`: `GCodeProcessor::Output_Block_Size = 65536`.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:859-898, 902-1168, 3987-4030`: `GCodeFileWriter` (64 KiB block buffering and flushing to the destination stream), `ExportLines` (tool backtrace and final line export), `GCodeProcessor::finalize`, `run_post_process`, and `run_second_pass_injection`.

## Problem

Previously, Ares materialized the complete post-processed G-code twice in memory before writing to disk:

1. `gcode_emit::orchestration::emit` built the raw G-code `Vec<u8>` while holding the post-island geometry buffers (`PreparedPostIslandPrintOrder`) alive through post-processing.
2. `gcode_emit::processor::process` ran `ExportLines`, which allocated a second monolithic `Vec<u8>` (`Vec::with_capacity(text.len())`) to hold the entire post-processed file (~6.2 MB for `ksr`, ~7.0 MB for `arachne`), and `pre_heating::inject` allocated a third full buffer when injecting multi-nozzle pre-heat lines.
3. `ares-cli` received `ProcessedGCode.gcode: Vec<u8>` and wrote it out via `fs::write`.

## Rust Destination Boundary

### Core (`crates/ares-core`)

- `crates/ares-core/src/gcode_sink.rs`:
  - `GCODE_OUTPUT_BLOCK_SIZE: usize = 65_536` (`GCodeProcessor::Output_Block_Size`).
  - `pub trait GCodeSink`: dyn-compatible platform-neutral sink with required `fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError>`, plus defaulted `flush(&mut self)` and `reserve_hint(&mut self, additional: usize)`.
  - Blanket impl for `&mut S` (`S: GCodeSink + ?Sized`) and built-in impl for `Vec<u8>`.
  - Adapters: `IoWriteSink<W: std::io::Write>` (preserving the underlying `std::io::Error` via `take_error()`) and `FnSink<F>` / `fn_sink(...)`.
  - `pub struct StreamedGCode`: carries `bytes_written`, `warnings`, `result` (`GCodeProcessorResult`), `print_statistics`, and `archive` without holding a duplicate `gcode: Vec<u8>`.
- `crates/ares-core/src/project_slice/gcode_emit/`:
  - `orchestration.rs`: exposes `emit_to_sink(..., &mut dyn GCodeSink) -> Result<StreamedGCode, SliceError>`, releases consumed `PreparedPostIslandPrintOrder` geometry buffers (`objects`, `support_layers`, `top_surfaces`, `curled_lines`, `nearest_seam_plans`) as soon as layer emission finishes, and routes existing `emit` through `emit_to_sink` with `&mut Vec<u8>`.
  - `orchestration_finish.rs`: routes `finish_to_sink` to `processor::process_to_sink`.
  - `processor.rs`, `processor/export_stream.rs`, `processor/export_lines.rs`, `processor/pre_heating.rs`:
    - Pass 1 runs `Estimate::record` and validates `PrintableAreaTracker::check` before any byte is written to the caller's sink. When `error_code != 0`, Pass 2 writes into a local `Vec<u8>` so `SliceError::ToolpathOutsidePrintableArea` retains its exact `gcode` payload while the caller's sink receives zero bytes.
    - Pass 2 streams finalized output through `ExportLines` into a fixed 64 KiB block buffer (`GCODE_OUTPUT_BLOCK_SIZE`), flushing non-empty chunks to `GCodeSink::write_chunk` at section boundaries (`; EXECUTABLE_BLOCK_START`, `; EXECUTABLE_BLOCK_END`) and whenever a block fills.
    - When `pre_heating` second-pass injection is active, `pre_heating::inject_to_sink` splices injected lines directly into `&mut dyn GCodeSink` in 64 KiB chunks without allocating a third monolithic buffer.
- `crates/ares-core/src/project_slice/{streaming.rs, validated.rs, cli_plates.rs, archive_plate.rs}` and `crates/ares-core/src/project_slice.rs`, `crates/ares-core/src/lib.rs`:
  - Preserves all existing `Vec<u8>` and `ProcessedGCode` public APIs identically.
  - Adds streaming entry points: `slice_project_plate_to_sink`, `slice_project_plate_processed_to_sink`, `slice_project_plate_validated_to_sink`, `slice_cli_project_plate_validated_to_sink`, `CliPresetProject::slice_project_to_sink`, `slice_cli_project_plates_to_sink`, and `CliPresetProject::slice_plates_to_sink`.

### CLI (`crates/ares-cli`)

- `crates/ares-cli/src/gcode_file_sink.rs`: lazy `AtomicFileSink` implementing `GCodeSink` over `std::io::BufWriter<File>` (`64 KiB` capacity).
  - Opens no file until the first `write_chunk`, so validation, slicing, conflict, custom-template, and printable-area errors never create or modify the destination path.
  - Stages regular-file writes in a sibling `.tmp` file (matching `GCode.cpp:2911-2965`) and atomically renames on `commit()`, removing the `.tmp` file on `Drop` if post-slicing checks (`--strict`, critical slicing warnings, downward checks, `--dump-moves`) or I/O fail. Non-regular targets (such as `/dev/null`) write directly without renaming.
- `crates/ares-cli/src/{project_presets.rs, direct_project.rs, slice_plates.rs, statistics.rs, main.rs}`: wires single-plate and multi-plate 3MF CLI paths to stream directly into `AtomicFileSink` while preserving all existing CLI flags, diagnostics, `--export-3mf`, `--export-statistics`, `--dump-moves`, and exit codes.

## Included and Deferred Behavior

### Included

- Platform-neutral, WASM-compatible `GCodeSink` trait and adapters (`Vec<u8>`, `&mut S`, `IoWriteSink`, `FnSink`).
- Chunked post-processor output streaming in `ares-core` with 100% byte-identical G-code and `GCodeProcessorResult` (including `record_moves` and `pre_heating`).
- Zero-byte-on-error guarantee before emission starts, plus atomic staging in `ares-cli`.
- Direct `BufWriter<File>` streaming across CLI single-plate (`ares slice <3mf> -o`, `ares --slice <n> --outputdir`) and multi-plate (`--slice 0` / `slice_plates`) paths.

### Deferred

- Streaming raw pre-processor layer output to a temporary file on native targets (deferred to keep `ares-core` free of direct filesystem I/O for Tier-1 WASM compatibility).
- Exposing `GCodeSink` as a browser `ReadableStream` in `crates/ares-wasm` (handled in a separate WASM slice).
- Streaming ZIP archive assembly in `export_sliced_3mf`, which continues to use `ProcessedGCode`.
