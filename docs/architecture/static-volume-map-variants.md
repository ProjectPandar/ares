# Static filament volume map write-back under filament variant activation

This is a source-cited rewrite slice of `OrcaSlicer/src/libslic3r`
`Print::update_filament_maps_to_config` for the single-nozzle printers whose
filament or printer variants are active. Upstream is pinned to
`345efec64c7b95a05ce5cf96df0dde66a158835c`. It widens the static write-back
from `filament-volume-map-static-writeback.md` to the variant case that slice
left out. It adds no Ares-owned selection.

## Source boundary

- `src/libslic3r/GCode/ToolOrdering.cpp:3054-3081`, the non-dynamic static
  branch of the filament grouping: every printer that reaches it writes the
  static maps back through `Print::update_filament_maps_to_config`, in every map
  mode. The branch has no BBL condition. The dynamic regroup above it
  (`:3005-3009`, `is_dynamic_group_reorder`) is closed for every shipping profile.
- `src/libslic3r/GCode/FilamentGroupUtils.cpp:346`, `update_used_filament_values`:
  used filaments take the derived value and unused filaments keep the base value.
  On a High Flow single-nozzle printer with only filament 2 used, the volume map
  is `0,1`: filament 1 keeps the Standard base and filament 2 takes High Flow.
- `src/libslic3r/Print.cpp:4656-4711`, `Print::update_filament_maps_to_config`:
  the changed map triplet replaces both the original full config and the runtime
  config (`:4664-4670`). Without a supplied volume map, each slot is backfilled
  from its mapped extruder's volume type (`:4704-4711`).
- `src/libslic3r/Print.cpp:4684,4722-4745`: the full config is rebuilt from the
  original full config (`:4722`). Under `expand_filaments` (`:4684`, extruder count
  above one, different extruders, or `has_multi_variant_filament`,
  `PrintConfig.cpp:10829`) the per-variant filament options and
  `filament_self_index` are selected again for the written map (`:4725`,
  `:4742-4745`).

## Destination

- `crates/ares-core/src/options/project_variants/filament_maps.rs`,
  `update_static_filament_maps`:
  - Remove the early return on an active variant layout. The printer, nozzle and
    mixed-filament conditions stay as they are.
  - End through the existing `refresh_filament_map(checkpoint, map, refreshed)`
    instead of resolving views from the already-selected current config.
    `refresh_filament_map` rebuilds from the variant checkpoint and selects the
    variants again under the same activation that `resolve_activation` already
    computes for `expand_filaments`. No new condition is added. Without an
    activation it resolves the same views as today.
- `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/recommend.rs`:
  remove only the `!is_bbl` term of the static write-back condition. `is_bbl`
  keeps its other uses in that file. No other part of that file changes.

## Included

- Single-nozzle static write-back with an active variant layout, in every map
  mode, on BBL and non-BBL printers alike. Witnesses: the two-variant Bambu Lab
  X1 Carbon and P1S printers, and the Creality Ender-3 High Flow variant column.
- The volume map takes one value per filament, including the `0,1` unused-entry
  value, not only the wider vector.

## Deferred

- Printers with more than one nozzle diameter (Snapmaker J1, Bambu Lab X2D):
  their maps come from the layered nozzle group result
  (`ToolOrdering.cpp:1752-1776,1869-1908`; `GCode.cpp:10754-10770`,
  `GCode::update_layer_related_config`). They keep the current early return.
- The dynamic regroup, sequential per-object stitching, mixed filaments and
  multi-nozzle counts keep their current early returns.
- The retract override recomputation in `Print.cpp:4727-4740` is not changed.
  It is reached through the existing `refresh_filament_map` path.

## Failure modes

1. Only the activation return is removed: BBL printers still skip the write-back
   and keep a single `filament_volume_map` value.
2. The BBL condition is removed but the views are still resolved from the current
   config: a one-filament X1C project whose plate map is longer than its
   consumables gets two `filament_self_index` slots and doubled per-variant
   filament values, where Native keeps one.
3. The Standard default is broadcast to every slot: the Ender-3 High Flow column
   writes `0,0` instead of `0,1`.
4. A separate activation check is added instead of reusing `resolve_activation`,
   and it drifts from the load-time gather.
5. Multi-nozzle printers start writing static maps that their group result would
   replace.

## Tests

`crates/ares-cli/tests/orca_parity/option_coverage/static_volume_map_variants.rs`
uses the existing variant footer check, which compares Native against API and CLI:

- Ender-3 High Flow variant column (`nozzle-footer/variant-column.3mf`):
  `filament_volume_map = 0,1`.
- X1C two-filament control (`bbl-jerk-tail-precision/positive-z-control.3mf`):
  `filament_volume_map = 0,0`.
- X1C plate map longer than its consumables
  (`filament-map-length/x1c-cube-too-long.3mf`): `filament_self_index = 1`,
  `filament_volume_map = 0`, `nozzle_temperature = 220`. This already matches
  and must keep matching; it catches failure mode 2.

## Single-object by-object print sequence

- `src/libslic3r/GCode/ToolOrdering.cpp:3054`: the static branch runs when
  `print_sequence != ByObject || objects().size() == 1`. A single-object
  by-object print takes the same static write-back as a by-layer print. Only
  more than one by-object object takes the `else` branch (`:3088-3091`), which
  wraps the config map without writing it back.
- `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/recommend.rs`
  already guards the whole block with this condition. Inside it, the write-back
  also requires a by-layer sequence. Remove only that inner test, so
  `update_static_filament_maps` runs whenever the block runs. No condition is
  added.
- Included: a single-object by-object print writes the same static maps as the
  by-layer print of the same project. Witness: the Ender-3 High Flow variant
  column printed by object, `filament_volume_map = 0,1`.
- Deferred: the sequential write-back in `Print.cpp:3665-3724` for more than one
  by-object object (auto map modes, per-object tool orderings), and the
  print-wide stitching of the per-object plans. The single-object print reaches
  the same static result there, so it does not change this case.
- Failure mode: the inner test stays and the single-object by-object print keeps
  the singleton `filament_volume_map = 0`.
- Test: `static_volume_map_variants.rs` adds the by-object case on
  `static-volume-map-by-object/single-object-high-flow.3mf`, which differs from
  `nozzle-footer/variant-column.3mf` only in `print_sequence`.
