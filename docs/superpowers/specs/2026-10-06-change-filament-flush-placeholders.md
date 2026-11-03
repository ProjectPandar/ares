# change_filament_gcode flush/placeholder family — port `GCode::set_extruder` dynamic config

Matrix finding (bridge_over_infill multi-filament E2E, X2D fixture
`input-multi-used.3mf`): seeded orca emits the stock BBL X2D
`change_filament_gcode` template, which references the `flush_length` /
`flush_length_N` / tower-interface placeholder family. Ares'
`tool_transition::change::dynamic::insert` (port of `GCode.cpp:7858-7898`)
never registers those keys, so the template renders `L`/`T`/`P` values
from the *static* config (all-zero / wrong) or fails — the X2D lint case
is blocked on this family. Slice boundary per coordinator: implement the
`change_filament_gcode` placeholder family only; SEMM `M104` emission,
Type2 tower activation, and EXCLUDE_OBJECT outline differences are
separate divergences owned elsewhere (see
`${TMP_ROOT}/ares-matrix-clusters.md`).

## Upstream boundary

`GCode::set_extruder` builds the `DynamicConfig` for the
`change_filament_gcode` render in two places:

- Tower path (`WipeTowerIntegration::append_tcr2`, `GCode.cpp:824-975`):
  `purge_volume = max(tcr.purge_volume, 100)` when
  `tcr.purge_volume > EPSILON`, else 0 (`:874`, `g_min_purge_volume=100`,
  `GCode.cpp:92`); `purge_length = purge_volume / filament_area` with
  `filament_area = PI/4 * filament_diameter[new]^2` (`:875-876`).
  `first_flush_volume`/`second_flush_volume` = `purge_length/2`
  (`:921-922`); `flush_length = purge_length` (`:950`);
  `flush_count = min(4, round(purge_volume/135))`,
  `flush_length_{1..flush_count} = purge_length/flush_count`, the rest
  zeroed (`:957-972`, `g_max_flush_count=4`,
  `g_purge_volume_one_time=135`). Also `wipe_avoid_perimeter` =
  `prime_tower_skip_points` (`:817,951`), `wipe_avoid_pos_x` from
  `get_wipe_avoid_pos_x(wt_box)` (`:882-887,952`),
  `is_prime_tower_interface = tcr.is_contact` (`:953`),
  `filament_tower_interface_purge_volume[new]` (`:954`),
  `filament_tower_interface_print_temp` — interface temp, or
  `nozzle_temperature_range_high[new]` when -1 (`:900-904,955`);
  on contact it also overrides `temperature`/`nozzle_temperature` and
  `first_layer_temperature`/`nozzle_temperature_initial_layer` entries
  for `new_filament_id` (`:905-915`), and zeroes
  `filament_cooling_before_tower` (`:935-936`).
- Non-tower path (`GCode.cpp:7825-7948`): same names sourced from
  `wipe_volume` — `flush_matrix[old_filament*N + new_filament] *
  flush_multiplier[new_extruder]`, minus `grab_length[new_extruder]*2.4`,
  clamped `>= 0` (`:7831-7843`); when the physical extruder changes and
  the new extruder has no prior filament, `wipe_volume = 0`
  (`:7829-7830`). `wipe_length = wipe_volume / filament_area` (`:7853`);
  `is_prime_tower_interface = false` (`:7897`),
  `filament_tower_interface_purge_volume`/`_print_temp` still set
  (`:7898-7904`), `wipe_avoid_perimeter = false`, `wipe_avoid_pos_x` still
  computed from the wipe-tower bbox (`:7895-7896`).
- `travel_point_{1..3}_{x,y}`: from `get_path_of_change_filament(print)`
  — filled once at export start when `m_writer.multiple_extruders`
  (`GCode.cpp:3212-3220`); `x/y/z_after_toolchange` =
  `tool_change_start_pos`/`nozzle_pos` (the XY where the toolchange
  sequence began).

Ares' `tool_transition::change::dynamic::insert`
(`gcode_emit/tool_transition/change/dynamic.rs`) already covers the
`:7861-7893` middle block (`previous_extruder`, `next_extruder`,
`current_hotend`, `next_hotend`, `layer_num`, `layer_z`,
`toolchange_z`, `max_layer_z`, `relative_e_axis`, `toolchange_count`,
`fan_speed`, `old/new_retract_length{,_toolchange}`, `old/new_filament_temp`,
`x/y/z_after_toolchange`, `old/new_filament_e_feedrate`). Missing:

- `flush_length`, `flush_length_1..flush_length_4`,
  `first_flush_volume`, `second_flush_volume` — from the `wipe_volume`
  computation (`:7831-7843`) since ares' tool_transition currently has no
  tcr (tower emit goes through `wipe_tower::emit` → `tool_transition::tower`
  which reuses the same `change::append`; the tcr's `purge_volume`
  equivalent is `plan.wipe_volume` /
  `ToolChangeResult` — needs threading through).
- `travel_point_{1,2,3}_{x,y}` — ares has no `get_path_of_change_filament`
  port; register zeros when absent (upstream leaves stale/zero values
  when `points.size()!=3`; X2D comments them out anyway but they must not
  fail template evaluation if referenced).
- `wipe_avoid_perimeter` (`prime_tower_skip_points`), `wipe_avoid_pos_x`
  (needs tower bbox; 0 when no tower).
- `is_prime_tower_interface` (tcr.contact on tower path / false
  otherwise), `filament_tower_interface_purge_volume[new]`,
  `filament_tower_interface_print_temp` (interface temp or
  `nozzle_temperature_range_high[new]` fallback at `:900-904`).
- `filament_cooling_before_tower` — already a registered option
  (`filament_options/gcode_source.rs`); upstream zeroes it on contact /
  layer 0 (`:935-936`) and exposes it as the post-defaults float list —
  mirror that in the dynamic config.
- Tower-contact `temperature`/`first_layer_temperature` overrides
  (`:905-915`) — only when `enable_tower_interface_features && contact`.

## Ares destination boundary

- `gcode_emit/tool_transition/change/dynamic.rs`: extend `insert` (or add
  a `flush` submodule) to compute `wipe_volume`/`wipe_length` per
  `:7825-7853` and register the full family. The tower branch's
  `tcr.purge_volume` flows via a new optional field on
  `tool_transition::TowerChange` (populated from
  `wipe_tower::emit`'s `ToolChangeResult`/`plan.wipe_volume`) — when
  `None` (non-tower toolchange), use the `flush_volumes_matrix` path.
- `wipe_tower::emit.rs`: pass `result`-level purge volume / `contact`
  into `TowerChange` so `dynamic.rs` can pick the `:874-975` branch.
- `motion` state: `m_writer.filament(new_extruder_id)` equivalent — the
  "old filament in new extruder" lookup at `:7829` needs
  `state.extruders`' per-extruder filament memory; check
  `tool_selection`/extruders state for an existing map before adding one.
- Placeholder values must land in the same `value::Config` types
  (`Value::Float`/`Integer`/`List`) the template evaluator already
  handles; `flush_length_N` keys are Float.

## Included

Everything above needed for the X2D stock template to render byte-
identical output on the lint fixture: the flush_length family,
`first/second_flush_volume`, `wipe_avoid_*`, `travel_point_*`,
`is_prime_tower_interface`, `filament_tower_interface_*`,
`filament_cooling_before_tower` dynamic zeroing, tower-contact
temperature overrides.

## Deferred (explicitly out of scope)

- `need_travel_after_change_filament_gcode` travel-avoid path
  (`GCode.cpp:1004-1050`) — `prime_tower_skip_points` geometry rerouting;
  only the `wipe_avoid_*` *placeholders* are in scope.
- SEMM bare-`M104` per-toolchange emission and the
  `EXCLUDE_OBJECT_DEFINE` outline polygon (5- vs 6-vertex) — separate
  divergences recorded in `${TMP_ROOT}/ares-matrix-clusters.md`, owner
  coordinator-assigned (not this slice).
- Type2 tower activation (`normalize_fdm_2` `used_filaments` counting /
  `tool_ordering.has_wipe_tower`) — pi-tower.
- `wipe_avoid_pos_x` real value (`get_wipe_avoid_pos_x` over the rotated
  tower bbox) — only needed when a template uses it AND a tower exists;
  register 0.0 otherwise as upstream does pre-tower.

## Verification

- RED/E2E: the staged bridge fixture
  `crates/ares-cli/tests/orca_parity/bridge_over_infill_filament.rs`
  (`input-multi-used.3mf`, X2D) slices past the guard removal AND renders
  `change_filament_gcode` with `L[flush_length]` resolved — byte-compare
  `plate_1.gcode` vs seeded oracle modulo `; generated by`. This test
  currently fails before this slice (X2D template) and doubles as the
  placeholder slice's E2E: RED already committed as `f0678956a`/`b9beda11a`
  history shows the guard failure; the placeholder failure is the next
  layer uncovered.
- Unit: `wipe_volume` math (`flush_matrix` index, `grab_length`*2.4
  clamp, `flush_multiplier`, `filament_area` divisor) and
  `flush_count`/`flush_length_N` bucketing (`min(4, round(v/135))`,
  zero-fill remainder) — table tests in `tool_transition` tests.
- Regression: single-filament + no-toolchange projects unchanged
  (placeholders registered but unreferenced).

## Ordering

Merges before the `bridge_over_infill` impl (`ac64d15cc`) lands; the
bridge E2E then verifies byte-identical on the X2D fixture. pi-tip's
O6 two-plates fixture test (branch `pi/o6-original-fixture-e2e`) is
cherry-picked at bridge merge time and must turn green.
