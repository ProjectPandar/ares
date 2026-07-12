# M24: libslic3r GCodeWriter boundary

## Goal
Port the first platform-neutral movement writer boundary from `OrcaSlicer/src/libslic3r/GCodeWriter.*` into `ares-core`, then route existing executable G-code movement commands through that upstream-aligned writer without adding new Ares pipeline stages.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `libslic3r::GCodeWriter` / `GCodeFormatter`; existing Ares output scaffolding may only be reused as a compatibility caller of that writer.

## Exit checklist
- `ares-core` has an internal source-cited `gcode_writer` module mapped to `OrcaSlicer/src/libslic3r/GCodeWriter.hpp` and `.cpp`.
- The module ports the first writer subset: G-code axis formatting precision, XY/Z travel commands, print speed setting, XY extrusion commands, zero-delta no-E output, XY offset application, current position tracking, and absolute E accumulation from delta extrusion input.
- Deferred upstream behavior is explicit: full `GCodeConfig`, `Extruder`, firmware flavors, retraction, toolchange, temperatures, fan, acceleration, jerk, arcs, Z-hop, pressure advance, cooling buffer, and flavor-specific postamble behavior.
- Existing executable movement command strings in `gcode.rs` are emitted through the writer route when they map to upstream writer movement; segment debug `G0` moves are removed and preserved only as `;SEGMENT` diagnostics.
- No new pipeline stage, crate, or third-party dependency is added.
- `ares_core::slice(input, options) -> Result<Vec<u8>, SliceError>` and `ares slice --options option.json -o output.gcode input.stl` remain unchanged.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check` pass.
