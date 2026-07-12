# Consume Relative E G-code Design

## Goal

Consume OrcaSlicer `use_relative_e_distances` as emitted G-code behavior, not only as registered option metadata. Ares must use the option to select extrusion-axis mode and emitted `E` values for print moves.

## Upstream Boundary

Line citations are pinned to the checked-out `OrcaSlicer` revision `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1418` declares `ConfigOptionBool use_relative_e_distances`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6980-6987` defines "Use relative E distances" with default `true`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:90-105` emits `M83 ; use relative distances for extrusion` when the option is true and `M82 ; use absolute distances for extrusion` when false.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:480-501` only emits `G92 E0` from `reset_e()` when `use_relative_e_distances` is false.
- `OrcaSlicer/src/libslic3r/GCode.cpp:890` and `OrcaSlicer/src/libslic3r/GCode.cpp:7800` expose the option as the `relative_e_axis` placeholder.

## Ares Destination Boundary

- `crates/ares-core/src/options/relative_e.rs`: add a focused typed accessor for `use_relative_e_distances`, defaulting to `true` and accepting only JSON booleans. `crates/ares-core/src/options.rs` only declares the module to keep the file under 400 LOC.
- `crates/ares-core/src/gcode_writer.rs`: add an extrusion-axis mode to `GCodeWriter`, emit `M83` or `M82` preamble commands, and use the mode when formatting print move `E` values.
- `crates/ares-core/src/gcode.rs`: read the option once in `format_gcode`, configure the writer, and emit the writer preamble before layer movement. If additions would push the file above 400 LOC, move the private print-move helper into `crates/ares-core/src/gcode/print_move.rs` before adding behavior.
- Tests remain in focused files to keep Rust source files under the repo's 400 LOC cap.

## Included Behavior

1. Default `SliceOptions::default()` behavior uses relative E distances, matching Orca's default.
2. `{"use_relative_e_distances": true}` emits `M83 ; use relative distances for extrusion`.
3. `{"use_relative_e_distances": false}` emits `M82 ; use absolute distances for extrusion`.
4. Absolute mode emits `G92 E0` immediately after `M82`, matching Orca's preamble/reset path for absolute extrusion.
5. Relative E mode emits each print move's delta extrusion as the `E` field while still tracking total internal E position for Ares diagnostics and later absolute calculations.
6. Absolute E mode preserves current Ares output semantics: `E` fields are cumulative positions.
7. Non-boolean `use_relative_e_distances` is rejected with `SliceError::InvalidInput`.

## Deferred Behavior

- Orca placeholder integration for `relative_e_axis` is deferred because Ares does not yet execute custom G-code placeholder scripts.
- Object-label E reset behavior from `GCodeWriter::add_object_end_labels()` is deferred because object labels are not in this slice.
- Wipe tower compatibility behavior is deferred because Ares has no wipe tower pipeline yet.
- Firmware/flavor-specific suppression for MakerWare, Mach3, Sailfish, and related writer branches is deferred because Ares currently emits one minimal Marlin-like G-code flavor without a parsed `gcode_flavor` behavior surface.

## Docs Impact

This spec and the implementation plan are the required documentation for the slice. No roadmap update is required because this work continues the current option-consumption milestone without changing milestone ordering or exit criteria.

## Acceptance Criteria

- A unit test proves writer relative mode emits consecutive print moves as `E0.12346` then `E1`, not cumulative `E1.12346`.
- A unit test proves writer absolute mode still emits consecutive print moves as cumulative `E0.12346` then `E1.12346`.
- An integration test proves default slice output includes `M83 ; use relative distances for extrusion` and does not include `G92 E0`.
- An integration test proves explicit false includes `M82 ; use absolute distances for extrusion`, then `G92 E0`, and a later extrusion command shows cumulative E.
- An options test proves default true, explicit false, explicit true, and non-boolean rejection.
- Existing targeted and full `ares-core` tests pass, plus `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the 400 LOC gate.

## Safety

This slice changes default emitted G-code because Ares currently defaults to absolute E output despite the already registered Orca default being relative E. The change is intentional and bounded to extrusion-axis mode selection and print move E formatting.
