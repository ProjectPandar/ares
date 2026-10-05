# M853: GCodeProcessor move vertices → libvgcode input data

## Goal
Port `GCodeProcessorResult::moves` recording (`OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.{hpp,cpp}`: `MoveVertex` `:184-218`, `store_move_vertex` `:5630-5718`, `process_G1` move typing `:3834-3925`, `process_tags` `:3113-3330`) into the existing `project_slice/gcode_emit/processor` pass as a public result, and port `src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:191-268 convert()` into `ares-vgcode`. Spec/plan: `docs/superpowers/specs/2026-10-04-gcode-processor-move-vertices.md`.

## Rewrite gate
Governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. No Ares preview pipeline: ares-core records upstream `MoveVertex`, ares-vgcode holds the upstream `convert()`, ares-wasm only exposes typed buffers, and the web app only renders them.

## Exit checklist
- Public `GCodeProcessorResult { moves, spiral_vase_mode }`, `MoveVertex`, `EMoveType`, and `slice_project_plate_processed`; `slice_project_plate` G-code bytes are unchanged.
- Move vertex count and fields match the patched `--process-gcode` oracle dump on `orca_cli_ender3.3mf` and `ksr_fdmtest_v4` (f32 tolerance), with a CSV diff artifact.
- `ares-vgcode` converts moves to `GCodeInputData` following `LibVGCodeWrapper::convert`.
- Web Preview renders toolpaths by role with a layer slider and legend; Playwright screenshot artifact.
- Included: actual-speed split vertices and Seam vertices. Deferred: actual_feedrate values, stealth times, layer_duration, PA/accel/jerk fields, object_label_id/print_z, foreign G-code loading.
- fmt, workspace clippy (no new warnings), workspace nextest, wasm32 build, official + seeded control set with no new FAIL.
