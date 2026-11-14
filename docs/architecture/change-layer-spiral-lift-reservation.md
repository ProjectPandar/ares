# Change-layer Spiral lift reservation

## Upstream behavior

OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19` defines the change-layer reservation in `src/libslic3r/GCode.cpp:8465–8469`, `GCode::change_layer`. The selected filament must enable `retract_when_changing_layer`, and the writer must satisfy `will_move_z(z)`. Auto selects SpiralLift for this reservation.

`GCode.cpp:10839–10896`, `GCode::retract`, schedules the lift when retraction-length, firmware, and surface conditions permit it. A fully retracted nozzle can still reserve a lift because lift eligibility does not depend on emitting additional E retraction.

`GCodeWriter.cpp:901–931`, `lazy_lift`, keeps an existing reservation. `travel_to_xyz` at `1000–1055` consumes the saved lift type. A later travel-local Slope selection therefore does not replace an earlier change-layer Spiral reservation.

## Rust boundary

The destination is the change-layer lift-reservation predicate in `crates/ares-core/src/project_slice/gcode_emit/layers/boundary.rs`, before `advance_for_layer`.

Use `state.options.retract_when_changing_layer` with `will_move_z`. The `pending_layer_retract` flag is a compatibility mechanism for deferred retraction, not an upstream prerequisite for this reservation. The corresponding retract/wipe decision is in `motion/travel.rs::flush_pending_retract_wipe`.

Keep hop height, retraction-length/firmware eligibility, existing lifted/pending reservation checks, and surface enforcement. Evaluate eligibility using the live writer Z. Keep physical-layer-index advancement and reservation ordering unchanged.

## Included and deferred behavior

This slice covers only the configuration predicate for change-layer lift reservation. Empty-pass layer-end ownership, Auto overhang queries, lift emission and rounding, object labels, tower restoration, and processor timing remain outside this boundary.

An empty normal-pass tail can bypass layer-end flag arming. That control-flow possibility explains why the pending flag cannot represent the upstream configuration predicate. It does not establish the live flag value for a particular slice.

## Test method

Use the public Native-double/API/CLI comparison in `mm_flush_into_infill_labels_target_matches_native` with its existing project fixture. At the first layer-boundary lift, compare Native `G17` / `G3 Z1.6 I-.534 J1.094 P1  F30000` against the Slope move `G1 X131.115 Y123.75 Z1.6 F30000`. Compare the complete output with the existing generator-identity normalization, including time headers and M73 commands.

Exercise flush/tower and Z-hop/lift/spiral cases in both official and seeded Native modes. Include disabled layer retraction, no Z movement, a fully retracted nozzle, live-Z and surface eligibility, existing reservations, ordinary travel, firmware-zero, tower approach, and spiral-vase controls. Check that control cases match Native.
