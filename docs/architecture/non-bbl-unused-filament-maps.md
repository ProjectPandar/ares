# Non-BBL automatic filament-map publication

## Upstream boundary

This slice rewrites the static non-BBL automatic map producer and its publication from OrcaSlicer v2.5.0, revision `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Upstream owner | Included behavior | Rust destination |
|---|---|---|
| `OrcaSlicer.cpp:3337,6973–6980` | Determine physical extruder count from nozzle diameters. Grow short CLI maps with head 1, without replacing existing multi-head assignments. | Existing project materialization retains the applied seed. |
| `GCode/ToolOrdering.cpp:1752–1767,1796–1805` | Non-BBL automatic multi-extruder recommendation bypasses the filament-group optimizer. Initialize all entries to the master head and assign identity indices up to the lesser of logical filament count and physical head count. | `options/project_variants/filament_maps.rs::update_static_filament_maps`, after actual layer usage exists. |
| `GCode/ToolOrdering.cpp:3062–3091` and `FilamentGroupUtils.cpp:348–359` | Merge extruder and volume recommendations into configured vectors only at used filament indices. Publish the full derived nozzle map. | The existing static publisher and its used-filament merge. |
| `Print.cpp:4661–4726` | Publish the maps, rebuild selected filament variants and derive configuration slots from the published extruder assignments. | Existing checkpoint, `refresh_filament_map`, and configuration-block reconstruction. |

The early non-BBL multi-head branch in `options/filament_map_recommend.rs::apply_recommended_filament_map` is a temporary compatibility shell for the ToolOrdering producer. Replacing the complete seed before actual layer collection does not implement upstream publication. That branch must preserve the seed instead. This is not a new grouping algorithm or an Ares-owned pipeline stage.

## Required behavior

For four physical heads, four logical filaments, automatic mapping and configured extruder map `[1,1,1,1]`, the derived zero-based result is `[0,1,2,3]`. If only filament 1 is used, the published extruder map stays `[1,1,1,1]`. If only filament 2 is used, it becomes `[1,2,1,1]`. Equal filament colours do not determine this result. Actual logical filament usage does.

The nozzle map is the full derived result `[0,1,2,3]` in both cases. It must not be reconstructed from the merged extruder map. The volume map uses the same used-only merge as the extruder map. Existing configured entries survive for unused filaments when vector lengths match the derived result. Existing source defaults apply when lengths differ.

`filament_map_2` continues to be derived through the existing configuration-slot resolver from the final published extruder map. It is not an output-text correction. The first case therefore resolves all entries to head 1's configuration slot, while the second resolves only filament 2 to head 2's slot.

The applied map snapshot used by validation and ironing retains its existing lifecycle. Single-head automatic assignment, Manual mappings and BBL automatic grouping retain their existing independent owners. The change must not reclassify a four-head non-BBL printer as a supported BBL grouping context.

## Public verification seam

Construct a synthetic project at runtime from the repository's public `tests/parity/arachne/default-prisms.3mf`. Supply a coherent four-head non-BBL configuration with four logical filament slots and automatic map `[1,1,1,1]`. Keep the original fixture unmodified on disk and commit no binary project or Native output.

Slice identical constructed bytes through the pinned Native CLI and the public core project-slicing API. Retain the input project, Native command/status and output, public API output, and machine-readable comparisons. Compare the complete values of `filament_map`, `filament_volume_map` and `filament_nozzle_map`, plus the derived `filament_map_2`. Do not filter or rewrite these values. Include both the first-filament-only case and the second-filament-only control. Ensure the second case really consumes filament 2.

Retain complete G-code streams for inspection and compare through the existing strict generator-only comparator when the constructed configuration has no unrelated output differences. Never add a comparator mask or substitute a hand-written Native result to accommodate a mismatch.

The affected inventory includes existing Manual, single-head and BBL grouping cases. Run the same inventory against the test-only source and the implementation in both official and seeded Native environments. Previously passing cases must remain passing. Formatting, strict workspace/all-target Clippy and core/WASM compilation remain required.

## Deferred behavior

This slice does not change BBL grouping or its cost/preference model, AMS colour matching, sequential multi-object grouping, mixed filament handling, dynamic selector mapping, multi-nozzle grouping, admission policy, or the public API surface. CLI `flush_volumes_matrix` recomputation for changed nozzle-volume types or physical extruder count is a separate source boundary and is excluded.
