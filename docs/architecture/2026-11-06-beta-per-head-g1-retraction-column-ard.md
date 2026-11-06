# Per-head G1 retraction configuration columns

## Source boundary and Rust destination

This test-only slice qualifies the configuration column consumed by ordinary G1 retraction. The owning OrcaSlicer sources are:

- `src/libslic3r/GCode.cpp:5039–5052,6609,10905,11239`: install the active filament configuration index at layer and toolchange boundaries.
- `src/libslic3r/Print.cpp:5068–5070,5113–5147`: `get_filament_config_indx` and `get_config_index` resolve filament identity and active nozzle extruder/volume identity.
- `src/libslic3r/Extruder.hpp:33–38` and `Extruder.cpp:197–253`: `m_config_index` and shared configuration getters.
- `src/libslic3r/GCodeWriter.cpp:1457–1477`: consume ordinary and toolchange retraction values.
- `src/libslic3r/PresetBundle.cpp:7582–7650`, `PrintConfig.cpp:11884–11999` and `src/OrcaSlicer.cpp:2774–2820,3557–3656`: compose included presets and gather per-filament variants.

The Rust destination is `crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip/nullable_vectors/retraction_column.rs`. Inputs are under `tests/parity/per-head-retraction-column/`. The existing motion adapter remains a compatibility shell around these upstream concepts. This slice changes no production getter, motion code or materializer.

## Configuration-column contract

A logical filament ID, physical extruder ID and materialized configuration-column index are distinct. Included preset layers must be composed before selecting each filament's extruder/volume variant. Selected self-indices must then identify the logical filament represented by each gathered column.

For `x2d_pla_bowden_inner_wall`, two six-column filament blocks select Bowden Standard columns 3 and 9. The resulting self-indices are `1,2` and retraction lengths are `3,3`. The first-layer command prefix is `G92 E0` followed by `G1 E-3 F1200`. Substituting physical index 1 for those configuration columns selects different values.

Firmware retraction is scalar. Filament cost uses `m_id`, and travel slope uses physical-extruder data. Those reads do not establish a configuration-column selector for retraction getters.

## Included public-output witnesses

The bounded passing set consists of these tests:

| Test | Behavior |
| --- | --- |
| `x2d_pla_bowden_inner_wall_nil_override` | Nullable inner-wall override selects the Bowden column |
| `x2d_pla_bowden_inner_wall` | Inner-wall output uses the mapped Bowden head |
| `x2d_pla_bowden_nozzle_placeholder` | Nozzle placeholder preserves the selected retraction column |
| `x2d_pla_bowden_nozzle_slot` | Nozzle slot resolves the selected retraction column |
| `x2d_pla_bowden_multi_value_overlay` | Multi-value overlay preserves column identity |
| `x2d_pla_bowden_nil_overlay` | Nil overlay preserves the inherited column |
| `x2d_pla_bowden_singleton_overlay` | Singleton overlay preserves the selected column |
| `ender3_single_head_nil_retraction_overrides` | Explicit length `.7` and retract/deretract speeds `31`/`29` produce `G1 E-.7 F1860` |

Each test requires matching Native double replay under the existing generator-line-only comparator, then compares validated public project-slicing output. The assertion covers ordered literal commands from the fixture-declared first layer marker through its first plain `G1 E-`. X2D inputs use `; CHANGE_LAYER`; the single-head control uses `;LAYER_CHANGE`. Marker fallback, command suppression, resynchronization and numeric normalization are excluded.

Complete Native and API output hashes and independent strict generator-line-only whole-output diagnostics accompany the prefix assertion. Prefix equality does not establish whole-program parity. The other controls and automatic-map counterexamples remain registered in the same 23-test suite.

## Deferred behavior

These automatic-map cases retain their prefix assertions:

- `x2d_pla_concentric_ironing_auto_map`
- `x2d_pla_auto_map_toolchange_boundary`
- `x2d_pla_variant_order_stream`
- `x2d_pla_variant_order_extrusion_speed`
- `x2d_pla_toolchange_without_virtual_move_comments`
- `x2d_pla_toolchange_with_virtual_move_comments`

Their project and plate metadata save `[1,1]` in Auto For Flush mode. Native computes `[2,1]`, selecting Bowden/Direct Drive retraction lengths `3,0.4`. The current dual-head automatic-map shell produces `[1,1]` and `0.4,0.4`. The owning producer is `ToolOrdering.cpp:1752–1850`, through `FilamentGroup::calc_filament_group`. Publication and rematerialization belong to `ToolOrdering.cpp:3053–3084` and `Print.cpp:4656–4678,4722–4753`. This producer gap is outside the getter qualification.

Flow ratio, wipe timing, Z-hop, restart extra, toolchange retraction and general getter-selector parity require separate discriminating public-output tests. Nonzero `retract_after_wipe` distribution, wipe geometry, custom-G-code cache invalidation, cooling, acceleration and arcs/scarf behavior are also outside this slice.

Preserve G1 formatting, feedrates, wipe/toolchange/Z-hop boundaries, first-extrusion synchronization and existing strict comparisons. Do not replace the computed-map producer with a fixed map, force Manual mode, modify fixture bytes or introduce a motion-side materializer.
