# Consume G-code Comments Design

## Goal

Port the OrcaSlicer `gcode_comments` runtime switch into Ares G-code command emission so the existing registered option changes concrete G-code output instead of remaining metadata-only.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1626` declares `ConfigOptionBool gcode_comments`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3845-3851` registers `gcode_comments` as a boolean option labeled "Verbose G-code" with default `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2036` assigns `GCodeWriter::full_gcode_comment = print->config().gcode_comments` for export.
- `OrcaSlicer/src/libslic3r/GCodeWriter.hpp:266-270` appends command comments only when comments are allowed and the comment text is non-empty.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:587-597`, `601-617`, `831-846`, and `933-952` route `GCodeWriter::full_gcode_comment` through speed, travel, Z travel, and extrusion move formatting.

## Ares Boundary

Implement the runtime slice in `crates/ares-core` only:

- Add a typed `SliceOptions::gcode_comments()` parser that reads the existing `gcode_comments` key as a strict boolean and defaults to `false`, matching the existing registry metadata.
- Extend `crates/ares-core/src/gcode_writer.rs` with command comment support for the existing `set_speed`, `travel_to_z`, `travel_to_xy`, and `extrude_to_xy` command writers.
- Thread the parsed `gcode_comments` value through `crates/ares-core/src/gcode.rs` so command comments are appended only when `gcode_comments` is `true`.
- Keep the existing structured Ares diagnostic comments, such as `;SPEED`, `;EXTRUSION`, `;MOVE`, `;LAYER`, `;CONTOUR`, and summary comments, unchanged for both `true` and `false`. These comments are currently part of Ares' test and debug surface and are not equivalent to Orca's inline explanatory command comments.
- Add concise command comment text for current Ares commands:
  - speed-only command: `; set speed`
  - layer-change Z travel: `; move to layer Z`
  - XY travel: `; travel`
  - extrusion move: `; extrude`
- Preserve command output bytes when `gcode_comments` is omitted or explicitly `false`. The existing Ares `; option_count = ...` diagnostic still reflects raw input key count, so explicitly passing `gcode_comments: false` may change only that diagnostic line.

This slice may add small helper methods or optional comment parameters to `GCodeWriter`, but it must not introduce a global mutable writer flag. Ares should keep comment behavior explicit per formatting call to remain WASM-safe and deterministic.

## Out Of Scope

- No removal or gating of existing Ares structured diagnostic comments.
- No changes to path geometry, path ordering, extrusion amounts, speed selection, feedrates, or layer generation.
- No object-label, exclude-object, start/end/custom G-code, temperature, fan, acceleration, jerk, pressure advance, or firmware-flavor comment behavior.
- No new option registration or registry metadata.
- No Ares-owned pipeline redesign.

## Acceptance Criteria

- `gcode_comments` defaults to `false` through `SliceOptions::gcode_comments()`.
- `gcode_comments` omitted preserves existing deterministic G-code bytes for the default tested pyramid output.
- `gcode_comments: false` emits the same command lines as the omitted case; only the existing `; option_count = ...` diagnostic may differ because it counts raw input keys.
- `gcode_comments: true` appends inline explanatory comments to emitted `G1` speed, Z travel, XY travel, and extrusion commands.
- `gcode_comments: true` does not change numeric coordinates, E positions, feedrates, structured diagnostic comment counts, or path-following command counts.
- Invalid `gcode_comments` values that are not JSON booleans are rejected through the G-code formatting path.
- Existing speed, extrusion, and movement behavior remains unchanged when comments are disabled.
- All touched Rust source files remain at or below 400 LOC.
- Verification must include focused red/green tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository LOC gate.
