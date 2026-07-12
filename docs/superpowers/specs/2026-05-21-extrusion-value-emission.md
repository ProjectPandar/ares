# M14 Extrusion Value Emission Spec

## Goal
Attach deterministic filament extrusion values to existing print moves and emit first absolute `E` values on path-following `G1` XY commands from the core `slice` API and CLI.

## Context
M13 created stable travel/print move artifacts and emitted `G0`/`G1` XY commands without extrusion. OrcaSlicer emits extrusion through `GCodeWriter::extrude_to_xy`, which updates the active filament position and writes an absolute `E` value on extrusion moves. OrcaSlicer computes role flow with `Flow::extrusion_width`, `Flow::auto_extrusion_width`, and `Flow::mm3_per_mm`; Ares needs a small deterministic subset before speed, retraction, support, and parity milestones.

Relevant OrcaSlicer references:
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:933-952` updates XY and emits `E` for extrusion moves.
- `OrcaSlicer/src/libslic3r/Flow.cpp:18-35` defines automatic role widths, using `1.125 * nozzle_diameter` for external perimeter and sparse infill roles.
- `OrcaSlicer/src/libslic3r/Flow.cpp:80-105` resolves role line width through role-specific value, `line_width`, percent-over-nozzle, then automatic width.
- `OrcaSlicer/src/libslic3r/Flow.cpp:201-211` computes non-bridge extrusion volume per millimeter with the rounded-rectangle formula.

## Requirements
- `ares-core` exposes extrusion artifact generation through `generate_extrusion_moves`, `LayerExtrusionMoves`, `ExtrusionMove`, and `ExtrusionOptions`.
- Extrusion generation consumes planned `Layer` heights plus `LayerToolpathMoves`; it preserves represented empty layers and move ordering.
- Travel moves remain travel moves with no emitted `E` value.
- Print moves emit absolute cumulative `E` positions based on their segment length from the previous move point in the same path-following sequence.
- The first print after a travel uses the distance from that travel point to the print point. External perimeter closing moves use the closing segment length back to the first point.
- Extrusion values use `dE = segment_length * mm3_per_mm / filament_cross_section_area`.
- Non-bridge `mm3_per_mm` uses Orca's rounded-rectangle area formula: `layer_height * (line_width - layer_height * (1 - PI / 4))`.
- Filament cross-section area uses `PI * (filament_diameter / 2)^2` with the first filament diameter, matching the current single-extruder scope.
- Width resolution for extrusion values supports `line_width`, `outer_wall_line_width`, and `sparse_infill_line_width` as millimeter numbers/strings or percent strings over the first nozzle diameter.
- Role-specific width `0` falls back to `line_width`; `line_width = 0` falls back to Orca's automatic `1.125 * nozzle_diameter` for current external perimeter and sparse infill roles.
- Invalid extrusion inputs at the public options boundary return `SliceError::InvalidInput` instead of panicking.
- `SlicingPipeline` includes an `Extrusions` stage after `Moves`, stores layer extrusion artifacts, and reports total extrusion move count and total extrusion millimeters.
- `slice` and `ares slice` output include total/per-layer extrusion metadata plus deterministic `;EXTRUSION:<kind>:<role>:x,y:<e-or-empty>` artifact lines.
- `G1` print move commands include an absolute `E` field; `G0` travel commands still omit `E`.
- Existing segment, contour, perimeter, infill, print path, and move metadata remains unchanged except for appending the new stage and extrusion metadata.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC; split files when necessary.

## Non-goals
- No feedrates/speeds, acceleration, jerk, volumetric speed limiting, pressure advance, retraction, wipe, Z-hop, seam placement, travel optimization, support/bridge/skirt/brim, multi-extruder/toolchange behavior, relative extrusion mode, arc fitting, wipe tower, or Orca G-code parity.
- No change to current infill geometry spacing; extrusion width resolution is only for E-value calculation in this milestone.
- No new workspace crates.

## Acceptance evidence
- Unit tests cover extrusion width resolution for millimeter values, percent strings, role fallback to `line_width`, and automatic width fallback.
- Unit tests cover extrusion generation for simple print moves with known `E` values, travel moves without `E`, cumulative absolute E positions, represented empty layer preservation, and mismatched layer/move inputs returning `SliceError::InvalidInput`.
- Pipeline tests assert `Extrusions` stage, layer extrusion artifacts, total extrusion move count, and positive total extrusion millimeters.
- Core `slice` and CLI tests assert appended extrusion metadata, exact sample `;EXTRUSION:` lines, `G0` travel commands without `E`, `G1` print commands with `E`, and unchanged move-command adjacency.
- Documentation adds M14 milestone and ARD entries and updates `docs/roadmap.md`.
- Full verification passes: `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and LOC checks.
