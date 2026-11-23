# Sequential-object first-layer nozzle temperature uses the configured G-code flavor

## Upstream boundary

OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/GCode.cpp:4694–4702`, the sequential-object transition in
  `GCode::_do_export`, processes `printing_by_object_gcode` and then calls
  `_print_first_layer_bed_temperature` and
  `_print_first_layer_extruder_temperatures` with `wait = false` before the
  next object's first layer.
- `src/libslic3r/GCode.cpp:5581–5614`,
  `_print_first_layer_extruder_temperatures`, skips generated commands when
  custom G-code already controls the extruder temperature. Otherwise it emits
  the first printing extruder for SEMM, or each printing extruder with the
  ooze-prevention standby adjustment, through `m_writer.set_temperature`.
- `src/libslic3r/GCodeWriter.cpp:317–373`, `GCodeWriter::set_temperature`,
  formats the command from the configured flavor. Its instance overload omits
  the tool for single-extruder and SEMM prints; the RepRapFirmware formatter
  emits `G10 S<temperature>` and uses `P<tool>` when a tool is supplied.
  Non-waiting output uses the default `set nozzle temperature` comment and
  does not append a wait command.

The affected call is distinct from the ordinary second-layer transition. It
is the first-layer restart between sequential objects, after the completed
object's layers and before the incoming object's first-layer motion.

## Rust destination and scope

In `crates/ares-core/src/project_slice/gcode_emit/layers/transition.rs`,
replace the private `write_nozzle_temperature` formatting at lines 268–285,
called by the sequential transition at lines 228–253, with
`gcode_emit::temperature::append`. Pass the resolved `gcode_flavor`, the
positive temperature after the existing eligibility check, `wait = false`,
and an optional tool only when the existing multiple-extruder decision says
that the tool is part of the command. The shared formatter owns the flavor,
parameter, comment, and RepRapFirmware `P` syntax; do not add another
flavor-specific writer.

Preserve the existing sequential transition timing, custom-G-code scan,
SEMM and ooze-prevention selection, temperature indexing, positive-target
gate, and surrounding bed/template output. Defer IMEX physical-heater
mapping as in the second-layer nozzle-temperature slice, along with any
unrelated temperature call sites, writer-cache changes, and custom-G-code
parsing changes. The current Rust code remains a compatibility shell around
the cited upstream `GCode` and `GCodeWriter` boundaries.

## End-to-end verification

1. Build a synthetic two-object `print_sequence = by object` project with
   RepRapFirmware, no temperature-setting start G-code, and distinct positive
   first-layer and ordinary nozzle targets. Compare Native and the public Ares
   slice in both configured parity modes. At the incoming object's first-layer
   restart, require the flavor-aware command and no hard-coded `M104`.
2. Compare the whole output for the single-extruder and SEMM cases, both of
   which omit the tool argument. A third project configures two tools and
   prints both objects with the second tool, exercising `P1` without a tool
   change at the object boundary. Compare its restart-span nozzle commands
   directly against the Native run. That project has a separate discrepancy
   in the serialized maps for the unused tool (`filament_nozzle_map` and
   `filament_volume_map`), outside this rewrite. All three command checks
   use `printing_by_object_gcode` to locate the restart, not startup.
3. Run the focused sequential/temperature parity families in both modes,
   followed by the broad paired run. Require zero new regressions, formatting,
   all-target Clippy, and the WASM check. Do not commit Native output or
   broaden comparator normalization.
