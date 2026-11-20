# Per-plate G-code statistics

Port OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `libslic3r/GCode/GCodeStatistics.cpp::gcode_statistics_json`. `PrintConfig.cpp:13153–13158` declares the opt-in bool; `OrcaSlicer.cpp:1621–1622,7184–7215` writes the sidecar after successful FFF G-code export and checking, only with an output directory. The sidecar replaces the `.gcode` extension with `.statistics.json` (`GCodeStatistics.cpp:185–188`), so the path is `DIR/plate_<n>.statistics.json`, with the physical one-based plate number. On write failure the G-code remains; Native records `CLI_EXPORT_STATISTICS_ERROR` (`OrcaSlicer.cpp:219,7211–7214`; `Utils.hpp:68`, −25 / POSIX 231).

## JSON contract

All references in this table are to `GCodeStatistics.cpp`. Compare parsed values recursively with exact key sets, ordered arrays, and exact values; object key order and JSON whitespace are immaterial. No numeric tolerances, array sorting, field omissions, or NaN coercion in comparison. The only normalized value is the top-level `generator`: Ares identifies itself rather than claiming to be an Orca binary. Assert both generator identities before comparing. `version` must match exactly.

| Field | Lines | Source / representation |
| --- | --- | --- |
| `version` | 175–176 | Integer `1`; denotes the schema, not the app release. |
| `generator` | 177 | App identity and version. |
| `units.time`, `.length`, `.volume`, `.weight` | 178 | `s`, `mm`, `mm3`, `g`. |
| `summary.time`, `.prepare_time` | 138–139 | Normal mode processor totals, 3 decimals. |
| `summary.travel_time` | 89–94,140 | Sum of Normal-mode travel vertices, 3 decimals. |
| `summary.travel_distance` | 141 | Processor total travel distance, 2 decimals. |
| `summary.layer_count` | 85–88,142 | Length of dense processor layer vector, not model layer count. |
| `summary.volume`, `.weight`, `.filament_cost` | 110–126,143–145 | Totals of unrounded filament figures, including flush; 3, 4, 2 decimals. |
| `summary.filament_changes` | 146 | Processor integer count. |
| `summary.filaments[].id`, `.length`, `.volume`, `.weight`, `.cost` | 114–131,147 | Iterate actual entries of the ascending, zero-based processor filament map; publish `id + 1`, not a dense ordinal. Length is volume divided by filament cross section. Precision 2/3/4/2, respectively; cost is weight in kg times configured price (zero when absent). |
| `summary.filaments[].model`, `.support`, `.prime_tower`, `.flush` | 116–120,132–135 | Each has `volume` and `weight`, 3/4 decimals. Missing bucket is zero. Prime tower uses wipe-tower volumes. |
| `summary.features`, `layers[].features` | 43–64,95–107,148,157 | Keys are upstream role strings (`ExtrusionEntity.cpp:693–719`), not enum names or IDs. Each value has `time`, `volume`, `weight` at 3/3/4 decimals. Only non-internal extrusions contribute. |
| `layers[].z`, `.time`, `.travel_time`, `.volume`, `.weight` | 73–107,150–157,180 | Index by `move.layer_id` from zero, including holes initialized to zero; no explicit `id` key. Z is the last non-custom, non-internal extrusion's Z (otherwise zero). Time includes every move, including internal vertices, and travel time only Travel. Volume and weight exclude internal vertices. Precision 3/3/3/3/4. |
| `objects[].name`, `.instance`, `.volume`, `.weight`, `.support_volume`, `.support_weight` | 159–173,181 | Object-mass vector order; name verbatim, zero-based instance index **within its model object** (`GCode.cpp:2737–2746`), not a G-code label ID. Final masses paired by vector index; missing support is zero. Object/support volume at 3 decimals and mass converted mg→g at 4. Brim belongs to plate, not object (`GCodeProcessor.cpp:7644–7699`). |

`rounded` uses `std::round(value * 10^n) / 10^n` (ties away from zero, lines 29–33); convert to double before summing as in lines 85–107. The function does not substitute zero for NaN/Infinity: non-finite floating values become JSON `null` at `ordered_json::dump(2)` (lines 27,182, upstream nlohmann serializer behavior). Weight is `0.001 * volume * density`, using a positive configured density or the `1.245f` default (lines 35–41; `GCodeProcessor.hpp:42`). Use binary bytes, indented by two spaces, with no appended newline (190–197).

## Boundaries

`ares-core::gcode_processor` owns statistics data and serialization, using the same `ProcessedGCode`, recorded `MoveVertex` data and `PrintEstimatedStatistics` produced for export. Carry any additional processor fields and object attribution through that pass; do not rerun slicing. `ares-cli` owns flag dispatch, artifact paths and writes. Use the direct native-style invocation for STL and an explicitly selected saved FFF project plate; existing STL placement and preset handling are unchanged. Neither the CLI nor the core adds ZIP members: `OrcaSlicer.cpp:7340–7350` and the `PartPlate`/`bbs_3mf` archive path are separate. Also outside this slice are invoking statistics without slicing, temporary G-code paths without `--outputdir`, and new arrangement or project transform behaviors.

## Public comparison

Run `--slice 1 --outputdir out --export-statistics` on synthetic single STL, two fixed-placement STL objects, and a saved 3MF project, using Native in both official and seeded modes. Compare the entire sidecar per the contract above and verify enabling it does not alter generated G-code. Register the comparison via `case_support.rs`.
