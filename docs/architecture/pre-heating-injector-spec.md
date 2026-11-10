# Pre-heating and pre-cooling injector

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/GCode/GCodeProcessor.cpp` and `GCodeProcessor.hpp`:

- `GCodeProcessor::finalize` (`:3998-4005`): after `run_post_process`, when
  `m_enable_pre_heating` is set (`apply_config`, `:3154`; `:3352`), it runs
  `run_second_pass_injection`.
- The usage blocks built during `run_post_process`, all gated on `m_enable_pre_heating`:
  - setup (`:1624-1630`): the filament and extruder block vectors are cleared, both machine
    G-code line ids are set to the `(unsigned)-1` sentinel, and one empty extruder block is
    seeded;
  - SKIPPABLE ranges (`:1607`, `:1824-1827`): `; SKIPPABLE_START` opens a
    `[start, 0]` range and `; SKIPPABLE_END` closes the last open range. These ranges are
    collected for every printer;
  - `build_usage_blocks` (`:1689-1771`), run on each raw input line before placeholder
    replacement (`:1832-1833`):
    - `; MACHINE_START_GCODE_END` and `; MACHINE_END_GCODE_START` record their line ids;
    - `T<fid>`, `;VT<fid>` and `M1020 S<fid>` each call `handle_filament_change`. Each may
      carry ` H<nozzle>`;
    - `; NOZZLE_CHANGE_START`/`END` and `; CP_TOOLCHANGE_WIPE` update the extruder blocks;
  - `handle_filament_change` (`:1638-1656`): it skips filament changes before the start
    marker or after the end marker, closes the previous filament block at the current line
    and opens `FilamentUsageBlock(filament, extruder, nozzle, line, -1)`
    (`GCodeProcessor.hpp:490-498`);
  - the close-out (`:1881-1908`): the last filament block ends at the end-G-code start;
  - `remap_gcode_id` (`:1151-1160`), applied at `:1925-1943`: every block id, every
    SKIPPABLE id and both marker ids move from input line ids to output line ids, using the
    exact-match table of `synchronize_moves`. The sentinel stays unchanged.
- `run_second_pass_injection` (`:1960-2126`):
  - the active time mode is the first enabled machine, Normal (`:1975-1981`);
  - nothing is injected when `m_result.nozzle_group_result` is not a
    `LayeredNozzleGroupResult` (`:1986-1987`);
  - the injector reads a float prefix sum of `moves[i].time[mode]` in move order, and the
    per-move times are restored afterwards (`:2005-2014`, `:2042-2043`);
  - the injector runs (`:2016-2039`) with `inject_time_threshold = 0`;
  - the finished G-code is rewritten: the lines registered for output line `k` are appended
    after line `k` (`:2087-2108`);
  - `handle_offsets_of_second_process` (`:2130-2141`) shifts every move by the number of
    lines inserted at keys below its `gcode_id`.
- `PreCoolingInjector` (`GCodeProcessor.hpp:955-1063`, `GCodeProcessor.cpp:2150-2473`):
  - `process_pre_cooling_and_heating` (`:2150-2193`): free blocks are grouped per extruder
    in `std::map` order. The last free block of each extruder is cooling-only. The current
    temperature is `get_nozzle_temp(last, false, true, false)`, which gives 140 for no
    filament. The target is `get_nozzle_temp(next, false, false, !ignore_cooling_before_tower)`,
    which gives 0 for no filament and subtracts `filament_preheat_temperature_delta` when the
    last argument is set. The filament path forces `ignore_cooling_before_tower`, so it never
    subtracts the delta;
  - `build_extruder_free_blocks` (`:2198-2204`): with at most one extruder block it takes
    the filament path;
  - `build_by_filament_blocks` (`:2429-2473`): blocks are grouped per extruder, framed by a
    start block `[0, start marker)` and an end block `[end marker, UINT_MAX)`. Consecutive
    pairs form free blocks, with `last_nozzle = next_nozzle` when the last is -1 and the
    partial range collapsed to `free_lower`. `ignore_cooling_before_tower` is forced true,
    and the blocks are sorted by `(free_lower, free_upper)`;
  - `inject_cooling_heating_command` (`:2217-2424`):
    - the move window comes from `lower_bound` on `gcode_id` (`:2295-2319`);
    - the post-extrusion cooling (`:2376-2381`). On the filament path the partial range is
      empty, so it writes `M104 … ;Multi extruder pre cooling in post extrusion` at
      `free_lower` with the current temperature whenever the last filament's
      `filament_pre_cooling_temperature` is above 0 and below its nozzle temperature;
    - the cooling-only branch (`:2383-2390`). `process_pre_cooling_and_heating` always sets
      pre-cooling, so the heating-only branch (`:2391-2405`) is never taken and is not ported;
    - the cool-then-heat branch (`:2406-2423`): `mid_temp`, the back-solved heating start
      found with `upper_bound` on the cumulative time, and the start relocated before a
      SKIPPABLE range by `adjust_iter` (`:2265-2290`);
    - the line format of `format_line_M104` (`:2334-2367`):
      `M632 S<next>[ H<nozzle>][ N R] W`, `M400`,
      `M104 T<physical extruder> S<temp> N0 ;<comment>` and `M633`;
    - the ranges are measured from 25 °C room temperature (`:2374`);
    - the `float` and `int` conversions are kept as upstream has them: `curr_temp` and
      `target_temp` are passed as `int` to `add_M104_lines`, `int(real_cooling_time *
      ext_cooling_rate)`, and the rates are `value_at_or_front(rates, extruder, 1.0)` narrowed
      to `float` (`:2207-2212`, `:2329-2331`).

### Static nozzle grouping

On the normal export path, the processor's `nozzle_group_result` is the print's layered
grouping (`GCode.cpp:3009`). That grouping is published for by-layer prints from
`ToolOrdering.cpp:3098-3102` and for by-object prints from `Print.cpp:3711`. For a printer with
one nozzle per extruder (every `extruder_max_nozzle_count` entry is 1, no nozzle stats), the
static grouping builds one nozzle per extruder in extruder order (`build_nozzle_groups`,
`ToolOrdering.cpp:1506-1527`; `build_nozzle_list`, `MultiNozzleUtils.cpp:25-40`). A filament
then resolves to nozzle id `filament_map[f] - 1`, and nozzle id equals extruder id
(`ToolOrdering.cpp:3093-3097`). The injector reads four lookups from that grouping:

- `get_nozzle_id` (`MultiNozzleUtils.cpp:454-458`);
- `get_nozzle_from_id(...).extruder_id` (`:442-446`);
- `get_first_nozzle_for_filament` (`:417-440`);
- `is_support_dynamic_nozzle_map`, which is false for this grouping.

Ares already models the same grouping for the `H` word of `M1020`
(`gcode_emit/motion/tool_selection.rs`, `nozzle`).

## Rust destination

- New `crates/ares-core/src/project_slice/gcode_emit/processor/pre_heating.rs`, the port of
  the gated usage-block builder and the orchestration of `run_second_pass_injection`. Its
  inputs are the processor input lines, the input-to-output line map of `ExportLines`, the
  recorded moves and `ProjectSettings`. It returns the rewritten output and shifts the move
  ids.
- New `pre_heating/injector.rs`: `PreCoolingInjector` on the filament path:
  `build_by_filament_blocks`, `process_pre_cooling_and_heating`,
  `inject_cooling_heating_command` and the line format.
- `gcode_emit/processor.rs`:
  - `ProcessorConfig` gains a `pre_heating` flag with a builder method (`:65-113`);
  - `process` keeps the move recorder and the line map when the flag is set (`:157-173`), and
    runs the injection after `ExportLines::finish` (`:260-264`). Recorded moves are returned
    only when the caller asked for them;
  - not touched: `process_gcode` and the load-time arguments (`:114-151`).
- `gcode_emit/motion/tool_selection.rs`: `nozzle` takes the project settings, so the usage
  builder reads the same static grouping as the `H` word of `M1020`.
- `gcode_emit/orchestration.rs` (`:374-390`): the export path sets the flag from
  `runtime_gcode.enable_pre_heating`, which is upstream's `finalize(true)` after
  `GCode::_do_export`. The flag does not depend on `disable_m73`.

## Premises

Measured on `07a20e194` in both modes.

1. Native injects only for the four profiles with `enable_pre_heating = 1` (H2C, H2D,
   H2D Pro and X2D), and only when the print uses both extruders. Of the 440 Native outputs
   with the start marker in the full-suite artifacts, all are H2D or X2D. 192 have injected
   lines, and every one uses both extruders (counting `T`, `;VT` and `M1020 S` selections
   between the markers). Each of the other 248 uses one extruder. None of the 440 has a
   `; NOZZLE_CHANGE_START`
   or `; CP_TOOLCHANGE_WIPE` line, so the extruder-block path is never reached. 149 have
   SKIPPABLE ranges.
2. In every Native output with injected lines, the suite's printers have one nozzle per
   extruder: `extruder_max_nozzle_count = 1,1` and empty nozzle stats.
   `has_filament_switcher` is 0, and no shipped profile sets `handle_hotend_as_extruder` or
   `has_filament_switcher`. Only H2C sets `extruder_max_nozzle_count` above 1 (`1,6`).
3. No existing test turns from FAIL into PASS from this slice alone. In each failing case
   with injected lines and a saved Ares output, removing the injected lines from Native still
   leaves a difference against Ares. Examples are the tool-change retraction split
   (`G1 E-1.80952` in Native vs `G1 E-2.80952` in Ares), the header time, and object labels
   around the tool change. This holds for the six `modifier_feature_filament` tests as well.
4. A Snapmaker J1 two-filament print, two extruders with `filament_map = 1,2`, has a prime
   tower off and both extruders in use. Ares already matches its G-code body. The fixture is
   `compatible_regions/tool_templates/fixture.rs`. With `enable_pre_heating = 1` set on the
   machine, Native writes the start marker and 29 injector lines in 14 `M632 … M633` heating
   groups and 15 `M400` + `M104` cooling pairs. The last cooling line is
   `M104 T1 S25 N0 ;Multi extruder pre cooling` before the end G-code. Removing those lines
   leaves Native equal to the Ares output everywhere except three config-block lines:
   `default_nozzle_volume_type`, `extruder_variant_list` and `print_extruder_id`. The first
   two lines differ with pre-heating off as well. Both modes give the same executable block.
   The config block follows the executable block, so the output line numbers of every
   injected line match between Native and the expected Ares output.

## Observable behavior

With `enable_pre_heating` set, after the `M73` pass, Ares inserts the injector lines of the
filament path after the output lines that upstream keys them on. One example is the
`M400` / `M104 T0 S209 N0 ;Multi extruder pre cooling` pair right after the `T1` line that ends
extruder 0's usage. The header times and `M73` lines are unchanged, because the injection
follows the time pass. Recorded moves move down by the number of lines inserted before them.
Without the flag, nothing changes. The flag is unset for every printer except the four above.

## Failure modes

- Injecting without the flag, or gating on `disable_m73` instead of the export path.
- Counting line ids after `M73` insertion for the builder, or skipping the remap, which
  shifts every insertion.
- Treating filament changes inside the start or end G-code as usages. This includes the
  end G-code's `T65279`/`T65535`, which fail the `fid < 255` test.
- Prefix-summing the move times in f64, or using `Estimate`'s f64 elapsed times instead of
  the float prefix sum of the per-move times. Either can move a back-solved heating start by
  one move.
- Dropping the `int` truncations of the temperatures, or the `std::map` and sort order that
  fix the order of lines registered on the same output line.
- Missing the 140 / 0 defaults for a missing filament, or the start and end frame blocks.
  The frame blocks produce the final cooling-only `S25` line.
- Physical extruder mapping: `physical_extruder_map[e]` when `e` is in range, else `e` itself.
  The J1 map has one entry.
- Splicing before line `k` instead of after it, or forgetting the move-id shift.

## Test plan

New test `injected_temperature_lines_and_positions_match_native` in
`crates/ares-cli/tests/orca_parity/runner/compatible_regions/tool_templates/pre_heating.rs`.
It builds the J1 fixture with `enable_pre_heating = 1` as a machine override, verified in
the exported project. The test checks two things:

- **API and CLI agree:** the API and CLI outputs are byte-equal.
- **The injection matches Native exactly:** the test computes the same projection on the
  Native output and on the Ares output, and requires the two to be equal.
  - An injection is a maximal run of consecutive injector groups. A heating group is an
    `M632 …` line followed by `M400`, an `M104 … N0 ;Multi extruder pre heating` line and
    `M633`. A cooling pair is `M400` followed by an `M104 … N0 ;Multi extruder pre cooling…`
    line.
  - The projection lists every injection in order, as three items: the 1-based output line
    number of the line right before the run, that line's text, and the run's lines.
  - So the test fixes the line contents, their order and their places in the output.

Runtime premises, with no stored Native values:

- Native has `; MACHINE_START_GCODE_END`;
- Native has at least one cooling line and one heating line;
- Native's `T` lines select both extruders.

The test does not compare the whole file against Native. The existing whole-output
comparisons stay as they are and keep failing honestly on the gaps of premise 3. The three
config lines of premise 4 are a separate follow-up slice, owned upstream by the following:

- `extruder_variant_list`: `extend_extruder_variant` (`PrintConfig.cpp:10630-10670`), which
  pads the list to the extruder count with its first value.
- `default_nozzle_volume_type`: the extruder-option resize of `set_num_extruders`
  (`:10672-10686`).
- `print_extruder_id`: `ensure_process_variant_columns` (`:11631-11659`), which rebuilds the
  process columns from the printer's `extruder_variant_list` before
  `update_values_to_printer_extruders` expands them (`PresetBundle.cpp:5581`,
  `PrintApply.cpp:1429`).

| Test | Before the port | After the port |
|---|---|---|
| `injected_temperature_lines_and_positions_match_native` (new) | F | P |

The test commit adds one FAIL per mode. The port commit is expected to turn it into PASS in
both modes, with no PASS turning into FAIL. In particular, the passing enable-pre-heating
tests where Native injects nothing must stay PASS: the H2D and X2D `prime_tower_export`
sweep, `sync_time`, `power_loss_recovery` and the flush-matrix reconstructions. Any further
F→P is listed with the port.

## Deferred

- The extruder-block path, which needs `; NOZZLE_CHANGE_START`/`END` from the wipe tower
  (`WipeTower.cpp:3646`). Its pieces are `ExtruderUsageBlcok`, `build_by_extruder_blocks`,
  the tower's partial-free range, `ignore_cooling_before_tower` from
  `; CP_TOOLCHANGE_WIPE` and the first-nozzle lookup of the close-out. Neither Native nor
  Ares writes those tags in the suite (premise 1). The Rust builder takes the filament path,
  as upstream does with at most one extruder block.
- Multi-nozzle and dynamic grouping (H2C, `extruder_max_nozzle_count > 1`): per-layer nozzle
  maps, the `H<nozzle>` and ` N R` words of `M632`, and nozzle ids that are not extruder ids.
  This belongs with the ToolOrdering multi-nozzle grouping.
- Three branches that read keys Ares does not type yet:
  - `handle_hotend_as_extruder` (`M104 I<filament>`);
  - the filament switcher target cap (`has_filament_switcher` with mixed `extruder_type`,
    `:2164-2189`);
  - the `extruder_max_nozzle_count` part of the line format.

  No shipped profile changes the first two, and only H2C changes the third. Ares writes
  their default forms.
- The null-grouping cases: an auto map mode whose derived map is empty returns before the
  grouping is published (`ToolOrdering.cpp:3058-3061`).
- `process_gcode` (the public processor entry, `processor.rs:114-151`): upstream's
  reprocess path also injects after `ensure_nozzle_group_result`. Wiring the flag there
  touches the load-time area and waits for that work.
- Unread inputs: `filament_types`, `filament_nozzle_temps_initial_layer` (every call passes
  `is_first_layer = false`), `filament_max_temperature_drop_when_ec`, `is_multiple_nozzle`, and
  the `lines_ends` rebuild (Ares keeps no line-end table).
- Undefined upstream reads are not ported. A filament id from custom G-code at or above the
  filament count indexes past `filament_nozzle_temps`. Ares reads the first entry, as
  `ConfigOptionVector::get_at` does.
