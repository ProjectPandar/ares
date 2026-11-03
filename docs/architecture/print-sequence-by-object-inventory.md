# print_sequence=by_object: upstream boundary inventory and Ares status

Date: 2026-10-08. Worker5. Spec-first review draft; **no production change**.

## Scope

`print_sequence = "by object"` (sequential printing): the upstream
boundary items, each marked 已移植 (ported) / 部分 (partial) / 缺失
(missing) in Ares, followed by the two-stock-printer × two-object
Native-vs-Ares first-divergence experiment.

## Upstream boundary items

### Option and mode selection

| # | Item | Upstream | Ares status |
|---|---|---|---|
| 1 | `print_sequence` option (coEnum `by layer`/`by object`, default ByLayer) | PrintConfig.cpp:1836-1848 | 已移植 — `ProcessPrintSequence` enum parsed (`fdm_normalization.rs:70,106`) |
| 2 | Mode branch in Print::process | Print.cpp:2475 `if (print_sequence == ByObject)` | 部分 — Ares consults `print_sequence` only for `initial_extruder` (`tool_ordering.rs:88`) and `total_toolchanges` (`first_filaments.rs:155-157`); no branch exists |

### Instance ordering and per-object ToolOrdering

| # | Item | Upstream | Ares status |
|---|---|---|---|
| 3 | Instance ordering by model order | Print.cpp:2478 `sort_object_instances_by_model_order` | 部分 — the by-layer instance chain exists; the by-model-order sequential sort is not a separate path |
| 4 | Per-object ToolOrdering construction (twice: filament census, then `sort_and_build_data`) | Print.cpp:2481, 2516-2518; GCode.cpp:3289-3297 (rebuild per object on change) | 缺失 — `schedule.tool_ordering` (`tool_ordering::build`) is built once over all objects |
| 5 | `first_layer_used_filaments` / `all_filaments` census per object | Print.cpp:2484-2493 | 缺失 — Ares derives used filaments from the global ordering |
| 6 | Auto `filament_map` recommendation over per-object layer filaments | Print.cpp:2500-2505 `get_recommended_filament_maps(all_filaments, …)` | 部分 — the recommendation exists (`filament_map_recommend`), but not fed by the by-object census shape |
| 7 | Per-object `first_extruder`/`last_extruder` handoff between objects | GCode.cpp:3291-3297 | 缺失 |

### Emission loop

| # | Item | Upstream | Ares status |
|---|---|---|---|
| 8 | By-object emit loop; **disabled when a wipe tower exists** (`ByObject && !has_wipe_tower` falls through to by-layer) | GCode.cpp:3284 | 缺失 — Ares emit is per-layer over the global ordering; the wipe-tower fallback is therefore unobservable |
| 9 | `printing_by_object_gcode` template + per-object first-layer bed/extruder temperatures | GCode.cpp:3334-3339, PrintConfig.cpp:2037 | 缺失 — the typed option exists (`printer_options/gcode_source.rs:25`) but nothing consumes it |
| 10 | `cal_most_used_extruder` per object | GCode.cpp:3342 | 缺失（deferred）— recorded in [tiq2-m73-time-placement.md](tiq2-m73-time-placement.md); the schedule carries one global count |
| 11 | `process_layers` per object with `collect_layers_to_print(object)` + instance index | GCode.cpp:3345 | 缺失 |
| 12 | Per-object flush statistics (`stats_by_single_extruder` etc.) + sorted layer filaments | GCode.cpp:3347-3355 | 缺失 |
| 13 | `finished_objects` counter + `current_object_idx` placeholder | GCode.cpp:3333, 3361 | 缺失 |
| 14 | Inter-object travel: `travel_to_z(first_layer + z_offset)` between objects; `m_cooling_buffer->reset` per object | GCode.cpp:3367, 3341 | 缺失 |
| 15 | Intra-layer object ordering (`first_layer_print_sequence`, `other_layers_print_sequence(_nums)`) | PrintConfig.cpp:1109-1121 | 部分 — handled inside the tool ordering (`first_layer.rs:129`, `reorder.rs:86-87`); independent of the by-object mode |

### Print::validate sequential checks

| # | Item | Upstream | Ares status |
|---|---|---|---|
| 16 | `sequential_print_clearance_valid` (horizontal clearance between consecutive instances, extruder_clearance_radius) | Print.cpp:613-… | 缺失 — only the option keys exist for export (`orca_block_keys/first.rs:103-105`) |
| 17 | Vertical clearance check (object heights under the extruder carriage) | Print.cpp:841 | 缺失 |
| 18 | Validate integration → rejection with the object pair/height report | Print.cpp:1298 | 缺失 — `print/validate/` holds bed_filament, extrusion_width, identity only |

## Summary

已移植: 1 (option parse). 部分: 2, 3, 6, 15. 缺失: 4, 5, 7-14, 16-18.

Ares today behaves as if `print_sequence` were always `by layer` for
scheduling and emission, with three observable exceptions
(`initial_extruder`, `total_toolchanges`, and the intra-layer ordering
keys).

## Experiment: two stock printers × two objects

Two stock printers (Creality Ender-3 0.4, Anycubic Kobra 3 0.4), two
`cube10.stl` objects, one filament, `print_sequence="by object"`
(process override), `disable_m73=1`. Exported through
`export_case_multi` (two model inputs, no filament patch), sliced by the
seeded native replay (2.4.2, seed 0) and by Ares, compared whole-file.

**Result: both cases DIVERGENT at the first real line — Ares ignores the
by-object mode and prints by layer.**

| Case | Native line 3 | Ares line 3 | Native bytes | Ares bytes |
|---|---|---|---:|---:|
| by-object/ender3 | `; total layer number: 100` | `; total layer number: 50` | 173149 | 168222 |
| by-object/kobra3 | `; total layer number: 124` | `; total layer number: 62` | 250212 | 241320 |

The first difference sits at byte 73 (line 3, column 23) — the total
layer count — after normalization of the `; generated by` line: native
prints each object completely before starting the next (2×50 / 2×62
layers), Ares prints both objects per layer (50 / 62 layers). This is
the missing emission loop (items 8/11) observable at the very first
diagnostic line; every downstream byte differs as a consequence.

Evidence artifacts:
`${TMP_ROOT}/ares-w5-gates/artifacts-byobj/case-*/` (input.3mf, orca.gcode,
ares.gcode, manifest.json; labels `by-object/ender3`,
`by-object/kobra3`). Probe: the temporary diagnostic
`orca_parity::runner::byobject_probe_tmp` (never committed), seeded
native replay 2.4.2 seed 0, dedicated target
`${TMP_ROOT}/ares-w5-gates/target-byobj`, cache
`${TMP_ROOT}/ares-w5-gates/cache-byobj`, 16 threads, branch base 3c1a79941.
Ares-as-by-layer equivalence with a native by-layer run of the same
input was not byte-verified (no zip tooling in the environment to patch
the input); the 50/62-layer counts and the missing emit loop make the
fallback conclusion structural, not incidental.

## Non-goals

No production change is proposed by this spec. The inventory only fixes
the boundary; any port slice needs its own source-cited spec and RED.
