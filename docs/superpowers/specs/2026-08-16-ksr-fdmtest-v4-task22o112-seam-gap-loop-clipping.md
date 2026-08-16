# Spec: Task 22o.112 seam-gap loop clipping

## Observable contract

The project G-code emitter resolves `seam_gap` from the effective region options in `ksr_fdmtest_v4.project.3mf`. A percent value is relative to the selected nozzle diameter. Every closed perimeter loop is shortened from its terminal end by that distance, so extrusion does not return to the loop start. Open fill and gap paths remain unchanged.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode.cpp:5792-5805`, where `GCode::extrude_loop` resolves `PrintRegionConfig::seam_gap`, converts percent against the active nozzle diameter, and calls `ExtrusionLoop::clip_end` before emission. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/motion/options.rs` plus the loop-only emission modules under `gcode_emit/motion/`.

## Included behavior

- Resolve absolute and percent `seam_gap` values from typed effective project settings.
- Clip across multiple terminal extrusion paths when the requested gap is longer than the last path.
- Preserve the path start and all non-loop extrusion entities.

## Deferred behavior

Seam candidate selection, aligned seam interpolation, scarf seams, and per-tool nozzle switching remain separate source-cited slices. The current emitter supports one active tool, so nozzle selection uses the active project nozzle already used by its motion boundary.
