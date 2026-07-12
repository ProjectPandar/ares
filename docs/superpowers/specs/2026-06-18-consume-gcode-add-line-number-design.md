# Consume G-code Add Line Number Design

## Goal

Implement a source-cited OrcaSlicer rewrite slice that makes the parsed `gcode_add_line_number` option change generated Ares G-code. When enabled, Ares should prefix each emitted G-code output line with the same `N{line_number} ` line-number marker used by Orca's post processor.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1353` declares `gcode_add_line_number` as a `GCodeConfig` boolean option.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3618-3622` registers `gcode_add_line_number`, labels it "Add line number", describes the `Nx` prefix, and defaults it to `false`.
- `OrcaSlicer/src/libslic3r/GCode/PostProcessor.hpp:28-30` declares `gcode_add_line_number` as a G-code post-processing function.
- `OrcaSlicer/src/libslic3r/GCode/PostProcessor.cpp:198-225` implements the behavior: if `gcode_add_line_number` is false, return unchanged; otherwise read the generated file line by line, prepend `N{line_number} ` to each line, start numbering at 1, increment once per input line, and write the transformed G-code back.
- `OrcaSlicer/src/slic3r/GUI/BackgroundSlicingProcess.cpp:905` calls the post processor after export.

## Ares Destination Boundary

- `crates/ares-core/src/gcode.rs` owns final in-memory G-code formatting for the platform-neutral core API.
- A new focused `ares-core` G-code post-processing module will own `gcode_add_line_number` parsing and line-prefix transformation, keeping `SliceOptions` and `gcode.rs` under the 400 LOC repository limit.
- The behavior remains in `ares-core` because the WASM API also returns bytes and cannot depend on CLI filesystem post-processing.

## Included Behavior

- Missing `gcode_add_line_number` and explicit `false` preserve existing output bytes except for the existing `option_count` difference when the option is present.
- Explicit `true` prefixes every line yielded by `str::lines()` with `N{line_number} `, starting at `N1`.
- Numbering includes comment lines and command lines, matching Orca's file-level post processor.
- The transformed output ends each numbered line with `\n`, matching Orca's `new_gcode += ... + "\n"` behavior.
- Non-boolean `gcode_add_line_number` values return `SliceError::InvalidInput("gcode_add_line_number must be a boolean")`.

## Deferred Behavior

- External post-processing scripts remain out of scope.
- File-based in-place rewriting remains out of scope because `ares-core` is platform-neutral and returns bytes.
- Line-number checksums, serial streaming, `M110` reset behavior, and firmware resend handling remain out of scope.
- No new G-code writer abstraction, CLI-only behavior, crates, or dependencies are introduced.

## Acceptance Criteria

- A direct unit test of the post-processing helper shows `G90\n;comment\nM2\n` becomes `N1 G90\nN2 ;comment\nN3 M2\n` when enabled.
- The same helper returns its input unchanged when disabled.
- Slicing with `gcode_add_line_number = true` produces output whose first line starts with `N1 `, whose later command lines include numbered `G90`, `M73`, and `M2`, and whose line numbers increase by one per output line.
- Slicing with `gcode_add_line_number = false` keeps command lines unnumbered.
- Slicing with a non-boolean `gcode_add_line_number` is rejected with the option name in the error.
- Existing deterministic-output tests remain valid for the default path.
- No Rust file under `crates/` exceeds 400 LOC.
