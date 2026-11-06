# ARD-0111: Acceleration and jerk variant materialization

## Status

Proposed source-cited rewrite slice. Production behavior remains within the destination boundary defined below.

## Upstream boundary

This slice ports the OrcaSlicer `libslic3r` variant-selection contract rather than introducing an Ares-owned variant model.

- `src/libslic3r/PrintConfig.cpp:2339-2349` defines `bridge_acceleration` as nullable `coFloatsOrPercents`, with its percentage relative to `outer_wall_acceleration`.
- `src/libslic3r/PrintConfig.cpp:10127-10146` includes the acceleration and jerk keys in `print_options_with_variant`:
  - acceleration: `default_acceleration`, `bridge_acceleration`, `travel_acceleration`, `initial_layer_travel_acceleration`, `initial_layer_acceleration`, `outer_wall_acceleration`, `inner_wall_acceleration`, `sparse_infill_acceleration`, `internal_solid_infill_acceleration`, `top_surface_acceleration`;
  - jerk: `default_jerk`, `outer_wall_jerk`, `inner_wall_jerk`, `infill_jerk`, `top_surface_jerk`, `initial_layer_jerk`, `travel_jerk`.
- `src/libslic3r/PrintConfig.hpp:1406-1419` declares the corresponding object fields as nullable float or float/percent vectors.
- `src/libslic3r/GCode.cpp:9277-9293` reads the active bridge value with `get_nozzle_config_index()` and `bridge_acceleration.get_at(nozzle).get_abs_value(outer_wall_acceleration.get_at(nozzle))`; the other acceleration and jerk paths use the same variant-indexed configuration contract.
- `src/libslic3r/GCode.cpp:10390-10393` applies the selected acceleration to emitted G-code, while `src/libslic3r/GCodeWriter.cpp:599-637` is the related jerk/acceleration command writer boundary.

The retained parity witness is Native `; bridge_acceleration = 2000,2000` versus Ares `; bridge_acceleration = 2000`, proving that the selected variant vector is not materialized before the generator consumes the option. Each included key has an independent public output-line parity assertion.

## Ares destination boundary

The destination is the existing process/object option and variant-materialization path:

- scalar object fields in `crates/ares-core/src/options/object_fields.rs` become variant-selected values through the existing `options/project_variants` boundary;
- process/object option assembly remains in `crates/ares-core/src/project_slice/gcode_emit/motion/options.rs` and `motion/options/build/fields/assemble.rs`;
- acceleration selection remains in `crates/ares-core/src/project_slice/gcode_emit/motion/state.rs` and the existing motion emission path;
- no new planner, processor, firmware, or global variant state is introduced.

## Included behavior

1. Preserve the upstream `print_options_with_variant` membership for the ten acceleration and seven jerk keys listed above.
2. Materialize the selected variant column for each key before the existing Ares motion/G-code emission reads it.
3. Preserve nullable values, float/percent ratio semantics, and existing acceleration/jerk fallback behavior.
4. Public E2E coverage asserts for each key that Native and Ares retain the same multi-value variant line at the config boundary.

## Deferred behavior

- `default_junction_deviation` remains deferred to the machine-envelope slice and the established JD planner boundary.
- `machine_max_*` envelope vectors remain deferred to the machine-envelope slice.
- Filament variant keys, filament volume-map normalization, adaptive pressure advance, hotend-rate variants, retraction/FlushMode, pre-cooling, and M400 skip behavior remain outside this slice.
- Timing behavior, planner behavior, and comparator policy are outside this source boundary.

## Boundary

This slice is limited to the listed option materialization and its per-key parity assertions. Adjacent configuration families and downstream motion semantics remain separate boundaries.
