# M15 Speed / Feedrate Emission Spec

## Goal
Attach deterministic movement speeds to extrusion/travel artifacts and emit first `F` feedrate values on path-following XY commands from the core `slice` API and CLI.

## Context
M14 emits absolute extrusion `E` values but no feedrate values, so generated path-following moves are still missing a basic printer-relevant field. OrcaSlicer stores speeds in mm/s and emits G-code feedrates as mm/min. `GCodeWriter::travel_to_xy` emits `F` on travel moves, while extrusion code sets speed before extrusion paths. Ares will add a structured speed artifact stage after extrusion artifacts and then format those speeds directly on current movement commands.

Relevant OrcaSlicer references:
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:587-595` emits `F` from a configured feedrate.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:601-617` emits travel XY with `travel_speed * 60`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2039-2050` defines `outer_wall_speed` default `60` mm/s.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4054-4062` defines `sparse_infill_speed` default `100` mm/s.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6610-6617` defines `travel_speed` default `120` mm/s.

## Requirements
- `ares-core` exposes `generate_speed_moves`, `LayerSpeedMoves`, `SpeedMove`, and `SpeedOptions`.
- `SliceOptions` exposes typed speed options for `travel_speed`, `outer_wall_speed`, and `sparse_infill_speed`.
- Speeds are parsed as positive finite millimeters per second from JSON numbers or numeric strings.
- Defaults match the Orca references above: travel `120`, external perimeter `60`, sparse infill `100` mm/s.
- Speed artifacts consume `LayerExtrusionMoves` and preserve represented empty layers, layer IDs, print Z, move kind, role, point, and `E` positions.
- Travel moves use `travel_speed`; external perimeter print moves use `outer_wall_speed`; sparse infill print moves use `sparse_infill_speed`.
- Feedrate values are emitted as `speed_mm_s * 60` mm/min.
- `SlicingPipeline` includes a `Speeds` stage after `Extrusions`, stores layer speed artifacts, and reports total speed move count.
- `slice` and `ares slice` output include total/per-layer speed metadata plus deterministic `;SPEED:<kind>:<role>:x,y:<feedrate>` artifact lines.
- Current `;EXTRUSION -> ;MOVE -> command` layout is extended to `;SPEED -> ;EXTRUSION -> ;MOVE -> command` without duplicating path-following commands.
- `G0` travel and `G1` print commands include `F` values; print commands keep absolute `E` values.
- Existing segment, contour, perimeter, infill, print path, move, and extrusion metadata remains unchanged except for appending speed stage/metadata and adding `F` fields to path-following commands.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC; split files when necessary.

## Non-goals
- No acceleration, jerk, volumetric speed limiting, cooling slowdowns, first-layer speed overrides, small-perimeter speed, bridge speed, support speed, retraction/wipe speeds, Z travel speed, pressure advance, seam placement, travel optimization, support/bridge/skirt/brim generation, multi-extruder/toolchange behavior, or Orca G-code parity.
- No speed commands emitted as separate standalone `G1 F...`; speeds are attached to current path-following XY commands for this milestone.
- No new workspace crates.

## Acceptance evidence
- Unit tests cover speed option defaults, numeric/string parsing, and invalid speed rejection.
- Unit tests cover speed artifact generation for travel, external perimeter print, sparse infill print, empty represented layers, and feedrate conversion.
- Pipeline tests assert `Speeds` stage, layer speed artifacts, and total speed move count.
- Core `slice` and CLI tests assert appended speed metadata, exact sample `;SPEED:` lines, no duplicate path-following commands, travel commands with `F7200`, external perimeter print commands with `F3600`, and sparse infill print commands with `F6000` using defaults.
- Documentation adds M15 milestone and ARD entries and updates `docs/roadmap.md`.
- Full verification passes: `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and LOC checks.
