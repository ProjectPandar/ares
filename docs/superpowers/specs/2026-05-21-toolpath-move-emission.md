# M13 Toolpath Move Emission Spec

## Goal
Convert ordered print path artifacts into deterministic travel/print move artifacts and emit the first path-following `G0`/`G1` XY moves from the core `slice` API and CLI.

## Context
M12 introduced ordered print path artifacts but still only exposed metadata comments. OrcaSlicer travels to the first path point before emitting extrusion moves through `GCode::extrude_path`, `GCode::extrude_loop`, `GCode::travel_to`, and `GCodeWriter::travel_to_xy` / `extrude_to_xy`. Ares does not yet compute extrusion E values, speeds, acceleration, retraction, seam placement, or travel optimization, so M13 emits geometry-following moves as artifacts plus non-extruding XY `G0`/`G1` commands. `G0` travel is a transitional Ares artifact, not a claim of Orca G-code parity.

## Requirements
- `ares-core` exposes `generate_toolpath_moves`, `LayerToolpathMoves`, `ToolpathMove`, and `ToolpathMoveKind`.
- Toolpath move generation consumes `LayerPrintPaths` only and preserves represented layers.
- Each print path emits one travel move to the first point.
- Open sparse infill paths emit one print move for each remaining point.
- Closed external perimeter paths emit print moves through each remaining point and one closing print move back to the first point.
- Move roles preserve the source print path role metadata.
- Empty print paths are rejected through existing `PrintPath` validation; move generation does not alter geometry.
- `SlicingPipeline` includes a `Moves` stage after `PrintPaths`, stores layer toolpath moves, and reports total toolpath move count.
- `slice` and `ares slice` output include total/per-layer toolpath move metadata plus deterministic `;MOVE:<kind>:<role>:` artifact lines.
- `slice` emits XY movement commands for toolpath moves: `G0` for travel and `G1` for print moves, with no extrusion E values or feedrate/speed fields.
- Existing segment, contour, perimeter, infill, and print path metadata remains unchanged except for appending the new stage and move metadata.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC; split files when necessary.

## Non-goals
- No extrusion E values, volumetric flow, feedrates, acceleration, jerk, retraction, Z-hop, seam placement, travel optimization, support/bridge/skirt/brim, arc fitting, wipe tower, or Orca G-code parity.
- No new geometry generation beyond translating current print paths into moves.
- No new workspace crates.

## Acceptance evidence
- Unit tests cover travel/print move generation for open sparse infill and closed external perimeter paths, represented empty layer preservation, layer preservation, and move role/kind metadata.
- Pipeline tests assert `Moves` stage and total move diagnostics.
- Core `slice` and CLI tests assert appended move metadata, exact sample move lines, exact move-command counts derived from `;MOVE:` adjacency, and adjacent matching `G0`/`G1` XY commands immediately after their `;MOVE:` markers.
- Documentation adds M13 milestone and ARD entries and updates `docs/roadmap.md`.
- Full verification passes: `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and LOC checks.
