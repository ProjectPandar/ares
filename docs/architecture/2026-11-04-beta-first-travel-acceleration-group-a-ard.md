# First-travel acceleration ordering

## Upstream boundary

This slice ports the upstream layer-start and first-travel decisions from `OrcaSlicer/src/libslic3r` rather than defining a new Ares pipeline.

- `GCode.cpp:6683–6685` enables farthest-point timelapse only for the supported printer/timelapse combination.
- `GCode.cpp:6450–6455` identifies the selected farthest point as the photo head for a single-nozzle print.
- `GCode.cpp:7386–7392` excludes the eager layer-start lift and snapshot block when the farthest point is the photo head.
- `GCode.cpp:10288–10549` owns `GCode::travel_to`.
- `GCode.cpp:10375–10384` selects the initial-layer travel acceleration.
- `GCode.cpp:10414–10419` emits that acceleration before retraction and travel.
- `GCode.cpp:10447–10454`, `GCode.cpp:10487–10488`, and `GCode.cpp:10490–10546` preserve retraction, pending-label consumption, and physical travel order.
- `GCodeWriter.cpp:428–473` owns acceleration caching and firmware formatting.
- `GCodeWriter.cpp:936–978`, `GCodeWriter.cpp:980–1141`, and `GCodeWriter.cpp:1175–1198` own eager-lift and Z/travel materialization.

The Ares destination is the existing layer-start orchestration in `crates/ares-core/src/project_slice/gcode_emit/layers/boundary.rs` and `timelapse.rs`, with its existing lift/travel consumers in `motion/travel.rs`, `motion/travel/lift.rs`, `motion/path/start_travel/approach.rs`, `motion/state.rs`, and `motion/acceleration.rs`.

## First-difference witness

The seven witnesses all show the same ordering residual: Native's first differing command is `M204 S6000`; Ares's first differing command is `G1 Z.6 F30000`. The Ares lift and timelapse sequence runs before the existing first-travel acceleration call, so Ares emits the acceleration later rather than omitting it. The witness oracle is this labeled first-difference section; Z motion is not treated as acceleration and later command counts are not used.

- `option_coverage::print_extruder_identity::print_process_identity_default_singleton_exports_complete_output`: Native 130954 bytes; Ares 131550 bytes; first differing lines 820/819.
- `runner::object_layer_ranges::object_range_retains_modifier_parent_configuration_like_native`: Native 228221 bytes; Ares 227740 bytes; first differing lines 1442/1442.
- `runner::object_range_epsilon::complex_range_exact_below_and_outside_epsilon_preserve_complete_native_output`: Native 232641 bytes; Ares 232144 bytes; first differing lines 1455/1454.
- `runner::object_range_epsilon::complex_range_inclusive_epsilon_threshold_matches_complete_native_output`: Native 232641 bytes; Ares 232144 bytes; first differing lines 1455/1454.
- `runner::object_range_epsilon::complex_region_range_within_epsilon_matches_complete_native_output`: Native 224922 bytes; Ares 224465 bytes; first differing lines 1441/1443.
- `runner::object_range_epsilon::fast_region_range_exact_and_near_boundary_preserve_complete_native_output`: Native 183331 bytes; Ares 182843 bytes; first differing lines 1441/1440.
- `runner::object_range_regions::object_range_wall_loops_changes_only_the_selected_constant_height_interval`: Native 183331 bytes; Ares 182843 bytes; first differing lines 1441/1440.

## Included

- The `farthest_point_timelapse` and photo-head predicate needed to suppress the premature layer-start eager lift and snapshot block.
- Existing first-layer travel acceleration selection, acceleration cache semantics, firmware command spelling, retraction, pending-label consumption, and Z/travel ordering.
- Literal first-difference ordering for the seven witnesses.
- Platform-neutral implementation in `ares-core`, without filesystem, terminal, viewer, or OpenGL dependencies.

## Deferred

- The E-retraction ordering family (`M204 T4000` versus `G1 E-.56 F1800`) and the object-marker family (`M204 T4000` versus `M486 S0`).
- Custom-G-code motion-state invalidation, a separate slice, including its downstream acceleration/jerk cache effects and command-count differences.
- Broad farthest-point geometry, multi-head/photo routing, snapshot-template parity, progress/timing, and unrelated layer/object transitions.
- Any behavior not required to establish the labeled first-difference ordering above.
