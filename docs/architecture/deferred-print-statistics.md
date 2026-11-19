# Deferred print statistics in G-code templates

Port the deferred bindings and final substitutions from OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`). The stock Anker M5 template
uses `total_weight` and `used_filament` at file start; both are unknown to
the current Ares table even though the processor computes material statistics.

## Registration and substitution

`src/libslic3r/GCode.cpp:4111–4126` registers 14 names for 12 reserved tags.
`GCode.cpp:4230–4231` expands the file-start template with the shared parser.
Extend Ares `project_slice/gcode_emit/processor/stats.rs` and reuse
`placeholders.rs::base_config` and `expression/deferred.rs::Deferred::of`.
These are the current counterparts of the upstream parser's deferred binding
and output-only operand handling. Keep the existing parser restrictions.

| Name | In Ares before this port | Native replacement in `GCode/GCodeProcessor.cpp:1387–1422` |
| --- | --- | --- |
| `print_time_total_sec` | yes | Nonnegative normal total seconds, two decimals |
| `print_time_day` | yes | Whole days |
| `print_time_hour` | yes | Hours within the day |
| `print_time_minute` | yes | Minutes within the hour |
| `print_time_sec` | yes | Seconds within the minute |
| `used_filament_length` | yes | Processor total material length, converted from mm to m, two decimals |
| `used_filament` | no | Same tag and replacement as `used_filament_length` |
| `print_time` | no | Alias of `normal_print_time` |
| `normal_print_time` | no | `get_time_dhm(Normal)` (`:4042–4044`), `Utils.hpp:573–642` compact duration |
| `silent_print_time` | no | `get_time_dhm(Stealth)` when enabled; otherwise `short_time("N/A")` returns `0s`. The pinned exporter resets all machines and enables only Normal (`:796–810`); do not add a stealth estimator. |
| `total_weight` | no | Processor total filament grams, two decimals |
| `total_cost` | no | Processor filament cost plus hourly `time_cost` × normal seconds ÷ 3600, two decimals |
| `total_wipe_tower_cost` | no | Writer's accumulated tower material cost, two decimals |
| `total_wipe_tower_filament` | no | Writer's accumulated tower filament, in mm (not m), two decimals |

Use the reserved strings in `GCodeProcessor.cpp:122–133`. At post-processing,
reuse the processor's per-slot volume conversions in
`processor/material_statistics.rs`: native `run_post_process` computes mm,
cm³, grams and cost from processor volumes and the configured f32 diameter,
density and price (`GCodeProcessor.cpp:1181–1196,3207–3211`). Sum the same
values used by the material comments, without parsing the emitted text.
Apply the configured time cost only for `total_cost` (`:1416`). Do not use the
preview's separate double-precision statistics refresh (`GCode.cpp:2574–2599`).

The two tower substitutions read `PrintStatistics` from emission, not the
processor's per-role volumes. Accumulate the writer's per-extruder tower
contribution at `finish.rs::append_filament_stats` and carry it through the
existing `orchestration_finish` handoff to `processor.rs`. Follow the
`GCode.cpp:3251–3253,3281–3282` length and extruded-volume differences, including
its `2.4052f` factor and the writer's density and price. Do not alter tower
planning or the existing footer statistics. `processor/time.rs` owns the
compact time formatting alongside its current duration formatting.

## Included and deferred

Include registration in existing template contexts, shared deferred alias
behavior, and the corresponding final G-code substitutions. Keep running
`extruded_*` variables, filename-format variables, template grammar and RNG,
preview/GUI statistics, scheduling, and active stealth estimation outside
this slice. Do not add keys outside `GCode.cpp:4111–4126`.

## Public parity

Select the upstream stock Anker M5 0.4 nozzle preset and its default process
and filament at runtime. Export the tracked cube with Native; run public API
and CLI slices against the same Native-sliced archive without storing a
Native product in the repository. In a derived case, add file-start comments
that consume the new bindings and set a nonzero hourly cost. Assert that the
reference actually expands them, then compare all G-code in both Native
modes, normalizing only the guarded generator line. Also run both modes on
the existing template, placeholder, file-start, print-statistics and header
cases without allowing a baseline match to fail.
