# Consume Z Offset Design

## Goal

Port the OrcaSlicer `z_offset` runtime behavior into Ares G-code layer Z emission so the existing registered option changes concrete output coordinates instead of remaining metadata-only.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1609` declares `ConfigOptionFloat z_offset`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5893-5901` registers `z_offset` as a millimeter float, default `0`, described as a value added to or subtracted from all Z coordinates in output G-code.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3010` exposes `z_offset` to placeholder expansion, which confirms the value is part of the G-code export configuration.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3335` moves to `initial_layer_print_height + m_config.z_offset.value` before priming/wipe-tower startup.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3431`, `7659`, `7702`, and `7950` subtract `m_config.z_offset.value` back out when reporting logical `layer_z` placeholders from the writer position.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5607-5611` silently initializes the writer Z position to `m_config.z_offset.value`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5617-5637` implements `GCode::change_layer(print_z)` by emitting travel to `print_z + m_config.z_offset.value`.

## Ares Boundary

Implement the runtime slice in `crates/ares-core` only:

- Add a typed `SliceOptions::z_offset()` parser that reads the existing `z_offset` key as a finite millimeter float and defaults to `0.0`, matching the current registry metadata.
- Apply `z_offset` only at the G-code writer boundary in `crates/ares-core/src/gcode.rs`: the actual `G1 Z...` layer-change command uses `layer.print_z() + z_offset`.
- Keep Ares' logical layer diagnostics on unoffset `print_z`: `;Z`, `Layer::print_z()`, layer planning, segment slicing, contours, perimeters, infill, skirt, brim, movement, extrusion, and speed data remain unchanged.
- Preserve existing `gcode_comments` behavior: when both `z_offset` and `gcode_comments` are enabled, the offset Z command still receives the existing `; move to layer Z` inline comment.

This slice may add a tiny helper in `gcode.rs` if needed to keep formatting readable, but it must not move z-offset handling into planning or geometry stages. Orca applies the offset while exporting G-code, not while changing the model/layer semantics.

## Out Of Scope

- No custom G-code placeholder expansion.
- No z-hop, travel lift, retraction, wipe tower, priming tower, toolchange, or multi-extruder behavior.
- No temperature, fan, pressure advance, acceleration, or firmware flavor behavior.
- No changes to layer planning, model geometry, extrusion amounts, path ordering, speed selection, or XY coordinates.
- No new option registration or registry metadata.
- No Ares-owned pipeline redesign.

## Acceptance Criteria

- `z_offset` defaults to `0.0` through `SliceOptions::z_offset()`.
- Omitting `z_offset` preserves the existing deterministic G-code bytes for the default tested pyramid output.
- Explicit `z_offset: 0` emits the same command lines as the omitted case; only the existing `; option_count = ...` diagnostic may differ because it counts raw input keys.
- Positive `z_offset` increases every layer-change `G1 Z...` command by that amount.
- Negative `z_offset` decreases every layer-change `G1 Z...` command by that amount.
- `;Z` diagnostic lines remain the original unoffset layer `print_z` values.
- `z_offset` does not change path-following command counts, `;MOVE` diagnostics, extrusion amounts, feedrates, or XY command coordinates.
- Invalid `z_offset` values that are not numeric or are not finite are rejected through the G-code formatting path.
- Existing `gcode_comments` inline comment behavior composes with offset Z commands.
- All touched Rust source files remain at or below 400 LOC.
- Verification must include focused red/green tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository LOC gate.
