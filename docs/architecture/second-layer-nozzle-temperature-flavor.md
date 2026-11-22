# Second-layer nozzle temperature uses the configured G-code flavor

## Upstream boundary

OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/GCode.cpp:6874–6881,6943–6966`, `GCode::process_layer`:
  the ordinary (non-IMEX-parallel) transition out of the first-layer band
  visits the writer's extruders and calls `m_writer.set_temperature` without
  waiting when the ordinary target is positive and differs from the initial
  target. SEMM and ooze prevention restrict the visit to the current tool.
  The nozzle command precedes the bed-temperature transition.
- `src/libslic3r/GCodeWriter.cpp:317–373`, `GCodeWriter::set_temperature`:
  the instance overload passes the configured flavor to the formatter and
  omits a tool argument for single-extruder and SEMM prints. The formatter
  emits `G10 S<temperature>` for RepRapFirmware, with ` P<tool>` when the
  caller supplies a tool. Marlin, for example, uses `M104 S<temperature>`
  and ` T<tool>`. The non-waiting call writes the default `set nozzle
  temperature` comment and no wait command.
- `src/libslic3r/GCode.cpp:4381–4387,4464–4466,4698–4702,5581–5614`:
  `_print_first_layer_extruder_temperatures` is a separate caller of the
  same writer. Its startup, post-start wait, and sequential-object restart
  behavior are not the ordinary-temperature transition.

In the affected stock prints, the generated temperature command is at the
second `;LAYER_CHANGE` section, before the bed command. It is not emitted by
machine-start G-code or the first-layer temperature scan.

## Rust destination and scope

In `crates/ares-core/src/project_slice/gcode_emit/machine.rs`, replace the
hard-coded `M104` formatting in `append_second_layer_transition` with the
existing `gcode_emit::temperature::append` writer equivalent. Pass the
resolved printer's `gcode_flavor`, the positive target after the existing
eligibility check, `wait = false`, and the same optional tool ID the current
call site selects.
The flavor-aware formatter already owns command choice, argument syntax,
and the default comment; do not add a parallel formatter or a preset-specific
branch. The surrounding transition is a compatibility shell around the
upstream `GCode::process_layer` boundary.

Preserve transition timing, tool selection, temperature indexing and
eligibility, and the bed command and its cache. Defer temperature-tool
enumeration and physical heater mapping, the IMEX-parallel branch, startup
and sequential-object restart output, and all other temperature call sites.
There is no change to custom G-code parsing, option validation, the CLI plate
selector, or the comparator. No platform-specific I/O is needed.

## End-to-end verification

1. Use an authored single-tool synthetic project with at least two printed
   layers, RepRapFirmware, distinct positive initial and ordinary nozzle
   targets, a changed bed target, and synthetic start G-code. Compare Native
   and public Ares slice output at the second layer: exactly one tool-less,
   non-waiting `G10` for the ordinary target precedes the bed transition,
   with neither `M104` for that target nor an added `M116`. Require the
   fixture to expose the mismatch before changing the call site.
2. Exercise two active tools in a RepRapFirmware print: where the writer
   supplies a tool ID, require `G10 S<temperature> P<tool>` rather than
   `T<tool>`. Keep an equal-target control (no automatic nozzle command
   there) and a Marlin changed-target control (still `M104`). Inspect the
   layer boundary so startup commands cannot satisfy the checks.
3. Run the focused paired checks against both official and seeded Native.
   Rerun the affected stock cases with the existing ordered-byte comparator
   and require matching streams under its existing generator normalization.
   Do not commit Native output or broaden the normalization.
