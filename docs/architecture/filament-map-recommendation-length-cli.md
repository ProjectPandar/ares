# CLI map length and exact recommendation assignment

## Source boundary

OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `OrcaSlicer.cpp:3851-3862` grows the palette to the selected filament-preset count. `:3925-3933` rejects a support-vector count different from that palette count.
- `OrcaSlicer.cpp:6974-6981` grows a short plate map and resets its logical prefix for a single extruder, but does not truncate an overlong map.
- `GCode/ToolOrdering.cpp:1752-1810,3063-3089` builds the automatic recommendation from the colour count and writes the resulting maps back.
- `Print.cpp:4661-4755` assigns the supplied map without repeating CLI map growth, then rebuilds filament variants and retract overrides.

The Rust destination remains `options/project_variants.rs::refresh_filament_map` and `project_variants/filament_maps.rs::update_static_filament_maps`. These are temporary compatibility shells around the upstream configuration lifecycle; this change adds no independent slicing behavior or production mapping policy.

## Test migration

The earlier two-diameter/one-colour input was written against older upstream behavior. It also names two filament presets. The current CLI grows its palette to two colours, then rejects its one-entry support vector at `OrcaSlicer.cpp:3932`. Adding another support entry makes that input legal but also produces two colours and map `[1,1]`; it cannot demonstrate a one-entry recommendation.

Use the existing synthetic `filament-map-length/x1c-cube-too-long.3mf` instead. Its single preset, diameter, colour and support entry satisfy the current admission boundary. The plate supplies map `[1,1]`, retained by CLI normalization, while the automatic recommendation contains one entry. This preserves the original exact-assignment intent with a legal current input, rather than changing expected output to accommodate an invalid premise.

The public Native/API/CLI test must retain the fixture hash, assert the source plate's two-entry map, qualify two Native outputs, assert their one-entry exported map and compare the entire output with only the existing generator-line normalization. No Native outputs are stored in the repository.

The unchanged-recommendation control must preserve its original public project slice and layer-reset override. Replace its historical implementation-output hash with a current Native whole-output comparison when Native can produce output; if the current Native cannot process that input, retain explicit observable unchanged-map and layer-reset behavior instead. A newly generated Ares hash is not an independent expectation.

Included: test input/source assertions and current Native comparison. Deferred: changes to CLI exit-code families, ToolOrdering algorithms, Print variant reconstruction, entry map growth, and unrelated geometry or export behavior. Any production discrepancy requires separate source localization before a port.
