# TimelapsePosPicker current-layer height context

## Rewrite boundary

Pinned upstream: Orca `345efec64c7b95a05ce5cf96df0dde66a158835c`.

Included source:

- `src/libslic3r/GCode/TimelapsePosPicker.cpp:35–43,59–60` initializes the optional integer nozzle-height gap and liftable configured-head ID when nozzle and height vectors both have multiple entries. The lower-height head is liftable. Equal heights select head 1 and a zero gap.
- `pick_pos_for_curr_layer:514–524` applies the negative gap for differing current/picture configured heads only when the picture head is not liftable.
- `collect_object_slices_data:144–159` computes float z_low from the actual double Layer::print_z plus the signed float range. A z_low <= 0 makes the whole bed unplaceable.
- `GCode.cpp::timelapse_gcode_config:6516–6550` supplies the current layer and configured heads to the picker, then publishes its safe flag and coordinates.

Rust destination: `crates/ares-core/src/project_slice/gcode_emit/timelapse_pos.rs::position` and the `timelapse.rs` caller. The traversal-based picker is a compatibility shell for this upstream boundary, not a separate Ares-owned pipeline. Its context carries the current layer and configured heads. Core behavior remains platform-neutral and byte-oriented.

## Non-negotiable decisions

1. Use the active configured physical head and ToolOrdering's most-used configured head, not IDs mapped through physical_extruder_map for template publication.
2. Read typed resolved nozzle diameters and numeric extruder_printable_height values. Preserve upstream integer gap truncation, float signed range and double-add-before-float conversion. No raw overlays, variant keys, rounded 5 mm cutoff or epsilon.
3. Apply the current-layer guard only to the current-layer route. Preserve the smooth/all-layer by-object return and clearance-radius truncation.
4. A whole-bed unplaceable projection leaves no safe region. Return the existing no-position result so the original timelapse template and writer-Z adoption operate without retract, CoolingBuffer or clock compensation.
5. Preserve the five existing complete Native/API/CLI cases, fixture builders, assertions, generator-only comparator, scripts, geometry, clock headers, G29 and M73. The test belongs outside runner.rs.

## Tests

The P0 control archive changes only farthest_point_timelapse to 0 and retains the original timelapse script. It compares complete Native, public validated API and CLI output using the existing generator-only normalization. The original five cases and their fixture builders remain unchanged.

The height boundary depends on configured current/picture heads and actual print_z. Differing heads with a non-liftable picture head exercise the negative range. Complete output keeps the clock headers, G29 and M73 in the comparison.

Failure modes:

- Differing heads with a non-liftable picture head incorrectly publish safe coordinates on low layers if the negative range is missing.
- Equal current/picture heads or a liftable picture head receive a spurious negative range.
- Unsupported vector lengths incorrectly engage the optional gap.
- Fractional configured heights lose integer-member truncation.
- Rounded emitted Z replaces actual print_z at a layer boundary.
- Physical-map reversal changes configured-head geometry indexing.
- Smooth/all-layer handling or clearance truncation changes.

An upstream P0 trace has heights [320,325], liftable head 0, current/picture heads 0/1 and range -5. Displayed Z5 is internally 5.000000000000002. The source expression reconstructs a positive z_low of 1.7763568394002505e-15, explaining the 24 blocked layers. The optimized-out z_low local is not directly measured.

## Deferred behavior

The optional plate-relative farthest target and its distance objective belong to the boundary described in `timelapse-picker-farthest-target-context.md`. Nil-height/native NaN-to-integer behavior, object/bbox enumeration, raft/rod/path-collision algorithms, geometry joins and sampling/tie behavior are outside this height-context boundary.
