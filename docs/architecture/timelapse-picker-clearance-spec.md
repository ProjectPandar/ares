# Timelapse picker: camera clearance and smooth by-object position

## Upstream boundary

`TimelapsePosPicker` at OrcaSlicer v2.5.0-beta `345efec64`:

- `TimelapsePosPicker.hpp:83` declares `int m_nozzle_clearance_radius`, and
  `TimelapsePosPicker.cpp:36` assigns the floating-point
  `extruder_clearance_radius` to it, truncating toward zero.
- `get_limit_area_for_camera` (`:361`) computes
  `float radius = m_nozzle_clearance_radius / 2`, an integer division, and
  `:363` inflates the object bounds by `sqrt(2) * radius`. The object projection
  is separate: `expand_object_projection` (`:274-292`) reads the configuration
  value directly and is not truncated.
- `pick_pos` (`:469-478`) takes `pick_pos_for_all_layer` when
  `m_based_on_all_layer` is set, which `:42` does for a smooth timelapse. That
  function returns `DefaultTimelapsePos` for a by-object print (`:587-589`), so
  `GCode.cpp:6545` defines `has_timelapse_safe_pos` as false.

## Ares destination

`crates/ares-core/src/project_slice/gcode_emit/timelapse_pos.rs`:

- `camera_limit_polygon` inflates by the unscaled half radius without either
  integer step, which differs from upstream unless the configured clearance is an
  even whole number.
- `position` has no smooth by-object branch.

The change is in the shared picker, so every path naming a position gets it, not
only the layer-change render from
[the layer-change configuration](timelapse-layer-change-config-spec.md).

## Included

- The camera limit inflates by `sqrt(2)` times the truncated clearance after
  integer division by two.
- `position` returns no position for a smooth by-object print.

## Deferred

These stay as recorded in the layer-change configuration and are unchanged here:

- The rest of `pick_pos_for_all_layer`: the smooth by-layer area, including the
  intersection of extruder areas and the height gap.
- Several objects, by-object collision and rod areas, the tower area and raft
  height.
- `clear_to_x0` for a by-object print with several objects.

## Verification

Tests use the public API and the runtime Native oracle, in official and seeded
modes, and store no Native values.

- The all-variables template in `custom_gcode_timelapse.rs` names
  `timelapse_pos_x` and `timelapse_pos_y` again, and adds a smooth by-object print
  to its comparison. Complete output must match Native.
- A runtime clearance sweep on the same fixture compares complete output for an
  odd whole number, a non-integer value and a value that moves the pick to another
  edge of the area, proving the truncation and division rather than one rounding.
- The layer-change configuration drops its record of this gap.
- The delta against the base has no pass-to-fail entry in either mode.
