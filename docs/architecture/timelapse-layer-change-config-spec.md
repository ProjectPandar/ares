# Timelapse layer-change configuration

## Upstream boundary

At OrcaSlicer v2.5.0-beta `345efec64`, `GCode::timelapse_gcode_config`
(`GCode.cpp:6516-6550`) builds one placeholder configuration for every
`time_lapse_gcode` render. Two call sites use it:

- `GCode::generate_timelapse_gcode` (`:6476-6477`), behind the BBL layer-start
  insert, the traditional inserts and the inline photo.
- The non-BBL render at the layer change (`:6710-6713`), with
  `skip_pos_pick = false`.

The configuration defines `layer_num`, `layer_z`, `max_layer_z`,
`most_used_physical_extruder_id`, `curr_physical_extruder_id`,
`timelapse_pos_x`, `timelapse_pos_y`, `has_timelapse_safe_pos`,
`timelapse_inline_photo`, `farthest_point_timelapse_enabled` and `clear_to_x0`.

`GCode.cpp:3395-3397` scans the template once: the position picker runs only for
a template naming `timelapse_pos` or `has_timelapse_safe_pos`, and
`TimelapsePosPicker::get_is_clear_to_x0` only for one naming `clear_to_x0`.
`:6535` falls back to `DefaultTimelapsePos` otherwise, and `:6545` defines
`has_timelapse_safe_pos` as `timelapse_pos != DefaultTimelapsePos`.

Both call sites name the template `time_lapse_gcode` for
`placeholder_parser_process`, and that name reaches the parser diagnostic.

## Ares destination

`crates/ares-core/src/project_slice/gcode_emit/timelapse.rs` keeps three entry
points, which differ only in writer bookkeeping and `skip_pos_pick`:

- `append_untracked` is the layer-change render.
- `append_and_track` is the `generate_timelapse_gcode` path.
- `append_inline` is the inline photo.

`Overlay` still models the v2.4.2 split, in which the layer-change render
defined only the three layer variables. It is replaced by the `skip_pos_pick`
argument of the upstream function, so no path has its own variable set.

## Included

- Every variable above on all three paths.
- The `:3395-3397` template scan and the `:6535` fallback, so a template naming
  no position does not run the picker per layer.
- `has_timelapse_safe_pos` as defined at `:6545`.
- `clear_to_x0` where `TimelapsePosPicker.cpp:641-683` is decided by source:
  - By-layer: `is_curr_obj` is true for every object, the unclear area stays
    empty, and the result is `true`.
  - By-object with one project object: every listed object is the back of the
    list, with the same result.
- The template name `time_lapse_gcode` in the parser diagnostic and the inline
  failure report.

## Deferred

- `clear_to_x0` for a by-object print with several objects. It needs the
  printed-object list and 3D instance bounds with raft height. Ares leaves the
  variable undefined there, so a template naming it is rejected as today, rather
  than rendering an unproven `true`.
- `TimelapsePosPicker` internals, which this slice calls unchanged and which were
  already outside `timelapse_pos.rs`:
  - `pick_pos_for_all_layer` for a smooth timelapse (`m_based_on_all_layer`,
    `TimelapsePosPicker.cpp:42`), including its by-object `DefaultTimelapsePos`.
  - Several objects: `footprint::model_bounds` reads the first object only.
  - By-object collision and rod areas, the tower area, raft height and the
    height gap between extruders.

  Non-BBL templates naming a position now reach these gaps on the layer-change
  path, where they were previously rejected. Each difference measured with the
  change is reported, not folded in.
- The legacy STL path in `gcode_placeholders.rs`, which is not this render.

## Verification

Tests use the public API and the runtime Native oracle, in official and seeded
modes, and store no Native values.

- `custom_gcode_timelapse_overlay_mk4_matches_native` keeps the
  `mk4-tlmpe.3mf` fixture. It requires one rendered marker per layer in Native,
  matching the layer-change branch of the hook gate from
  [the hook specification](timelapse-layer-change-gate-spec.md), and compares
  complete output.
- A runtime template naming every variable compares complete output for
  traditional, smooth and single-object by-object prints. Native must resolve
  each variable.
- A runtime template naming an unknown variable compares exit code, stdout,
  stderr and `result.json` with Native, and retains no G-code. This keeps the
  rejection coverage the fixture no longer provides.
- The delta against the base has no pass-to-fail entry in either mode.
