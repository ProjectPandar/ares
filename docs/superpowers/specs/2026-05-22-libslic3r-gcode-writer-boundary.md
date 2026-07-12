# libslic3r GCodeWriter Boundary Spec

## Rewrite gate
This milestone follows `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`: it is a source-cited rewrite slice of `OrcaSlicer/src/libslic3r/GCodeWriter.*`, not a new Ares pipeline feature. Existing Ares output scaffolding may only call this upstream-aligned writer boundary.

## Goal
Port the first platform-neutral `libslic3r::GCodeWriter` / `GCodeFormatter` boundary into `ares-core` and route current executable movement command emission through that writer, without adding a new pipeline stage or changing the public `ares_core::slice` / CLI contracts.

## Upstream source scope
Implemented in this milestone:
- `OrcaSlicer/src/libslic3r/GCodeWriter.hpp`: `GCodeWriter` movement API shape (`set_speed`, `travel_to_z`, `travel_to_xy`, `extrude_to_xy`) and `GCodeFormatter` axis formatting constants.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp`: movement-line formatting, XY offset application, position updates, delta extrusion accumulation, F-axis emission where upstream emits it, and formatter axis rounding/trimming.

Deferred from this milestone:
- Full `GCodeConfig`, `Extruder`, retraction, toolchange, temperature/fan/acceleration/jerk, firmware flavor branching, arcs, Z-hop, pressure advance, cooling buffer, custom G-code, machine-limit behavior, and flavor-specific `postamble()` branching.
- Adding or changing slicing pipeline stages.
- Segment visualization `G0` commands are not upstream `GCodeWriter` output; this milestone removes those executable debug moves and keeps `;SEGMENT` comments as diagnostics.

## Functional requirements
1. Add an internal `gcode_writer` module in `ares-core` with source-cited Rust `GCodeWriter` and formatter helpers.
2. The writer tracks current XYZ position, current feedrate, current absolute E, and XY offset for the subset implemented now.
3. Formatter helpers use upstream export precision constants: XYZ/F axes use 3 digits, E uses 5 digits, with rounded trailing-zero-trimmed output.
4. `travel_to_z(z, feedrate)` emits exactly `G1 Z{z} F{feedrate}\n` and updates Z/feedrate. The feedrate is an Ares adapter parameter until `GCodeConfig` travel-speed options are ported.
5. `travel_to_xy(point, feedrate)` emits exactly `G1 X{x} Y{y} F{feedrate}\n`, applies XY offset, and updates X/Y/feedrate. The feedrate is an Ares adapter parameter until `GCodeConfig` travel-speed options are ported.
6. `extrude_to_xy(point, delta_e)` mirrors upstream shape by taking a delta extrusion amount and accumulating absolute E internally. For non-zero deltas it emits exactly `G1 X{x} Y{y} E{absolute_e}\n`; for effectively zero deltas it emits exactly `G1 X{x} Y{y}\n`. It does not emit F.
7. `set_speed(feedrate)` emits exactly `G1 F{feedrate}\n` and updates current feedrate. `gcode.rs` uses this before print moves when the requested print feedrate changes, because upstream `extrude_to_xy` does not emit F.
8. `reset_e()` is internal state reset only for this milestone. It must not emit `G92 E0`; upstream `reset_e` output is deferred until flavor/config/extruder state is ported.
9. Keep existing final `M2\n` in `gcode.rs` as Ares' current placeholder end line. Do not present it as a port of upstream `postamble()` until flavor support exists.
10. `gcode.rs` must use `GCodeWriter` for all executable movement command lines that correspond to upstream writer movement: layer Z moves, travel moves, speed changes, and extrusion moves. Existing executable segment debug moves are removed; `;SEGMENT` comments remain as diagnostics.
11. Existing diagnostics remain present. Expected command-byte changes are: segment `G0` debug moves are removed, executable travel moves become `G1`, print feedrate is emitted as a separate `G1 F...` line before writer `extrude_to_xy` output when needed, zero-delta extrusion emits no E axis, and executable non-zero E values use 5-digit writer precision.
12. `ares_core::slice(input, options) -> Result<Vec<u8>, SliceError>` and `ares slice --options option.json -o output.gcode input.stl` remain unchanged.

## Non-functional requirements
- No new crates or third-party dependencies.
- `ares-core` remains WASM-safe and filesystem/UI/OpenGL-free.
- Modified Rust files remain under 400 LOC; split `gcode.rs` responsibilities if needed.
- No legacy fallback path: executable movement commands should have one writer route after this milestone.
- Verification passes: `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`.

## Tests
- Unit tests cover formatter axis precision/rounding/trimming, `set_speed`, `travel_to_z`, `travel_to_xy`, `extrude_to_xy` delta-to-absolute E accumulation, zero-delta no-E output, XY offsets, current position/feedrate updates, and internal E reset.
- Existing core/CLI G-code tests are updated only for expected writer command formatting changes.

## Documentation updates
- Add `docs/milestones/m24-libslic3r-gcode-writer-boundary.md`.
- Update `docs/roadmap.md` so M24 is this `libslic3r` writer boundary, with WASM/browser and E2E parity shifted later.
