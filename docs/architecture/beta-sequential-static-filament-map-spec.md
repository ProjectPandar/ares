# Sequential static filament-map publication

## Source boundary

This slice ports the single-head automatic part of OrcaSlicer 2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c` sequential static publication.
It completes an omitted caller of the existing static map producer, not a
new Ares grouping algorithm or export policy.

All source paths below are relative to `OrcaSlicer/src/libslic3r/`.

| Upstream owner | Behavior | Rust destination |
| --- | --- | --- |
| `Print.cpp:3665–3734`, `Print::process` | Collect actual per-object layer usage; obtain the static group result; in automatic modes publish its three maps | `project_slice/gcode_emit/tool_ordering/recommend.rs::build_recommended` |
| `GCode/ToolOrdering.cpp:1752–1770,1801–1802,1872–1873` | A single physical head uses the trivial master assignment without the multi-head optimizer | Existing `options/project_variants/filament_maps.rs::update_static_filament_maps` |
| `GCode/ToolOrdering.cpp:1892–1907`, `build_default_nozzle_list` | One nozzle per head; `group_id` and `extruder_id` equal the zero-based head index | The existing static producer's head, volume and nozzle projections |
| `MultiNozzleUtils.cpp:319–369`, `get_extruder_map/get_nozzle_map/get_volume_map` | Extruder export is one-based; nozzle export is the zero-based global group ID; volume export is the nozzle volume type | The same static producer, without another conversion layer |
| `Print.cpp:4656–4753`, `Print::update_filament_maps_to_config` | Compare all three maps, apply them to current and original full config, then rebuild the full config | Existing map publication and `refresh_filament_map(checkpoint, map, current)` |
| `GCode/ToolOrdering.cpp:3010–3014,3053–3084` | Multi-object sequential orderings do not own print-wide publication; the owning caller is `Print::process` | The caller eligibility in `build_recommended` |

## Existing Rust gap

`build_recommended` currently executes static publication only for a
non-sequential print or a print with one object. The same condition skips
single-head automatic multi-object sequential prints. The downstream
config writer now emits the actual `filament_nozzle_map`, so the missing
producer leaves its initial `{1}` visible in those exports.

`PrintConfig.cpp:3146–3153` defines that initial value as `{1}`. The default
is not the final assignment. With one physical head, master extruder `1`
and one fixed nozzle, the upstream trivial result has head `1`, volume
`0` for Standard, and nozzle group `0`. An unchanged head map does not make
the nozzle-map update optional.

The old caller restriction is a temporary compatibility shell around the
upstream sequential boundary. Replace only that restriction for the
included single-head automatic case. Do not bring back a synthetic footer
value to hide the missing state transition.

## Contract

- Include multi-object `ByObject` prints in the ordinary static path when
  there is one physical head and the mode is `AutoForFlush` or
  `AutoForMatch`. Existing mixed-filament and multiple-nozzle guards remain.
- Use actual collected layer usage and the existing colour-count rule for
  the full assignment. Do not derive cardinality from geometry, physical
  head count or a fixed single-element output.
- Reuse `update_static_filament_maps` for the trivial master assignment,
  used-entry head/volume merge and complete nozzle map. Keep its three-map
  comparison and existing refresh behavior, including an unchanged head.
- Publish through the existing current/config checkpoint boundary and
  regenerate the config block from the resulting `views.full`. No second
  checkpoint, snapshot owner, serializer override or normalization pass.
- Preserve the existing by-layer and one-object paths. For multi-object
  sequential Manual mode, keep the `Print.cpp:3711–3715` no-write gate.
  This slice does not make that path use automatic assignment.
- Keep `PrintConfig` defaults, option kinds, membership and all materializer
  rules unchanged. Preserve the application-time map used by ironing.
- Leave the multi-head solver, parser, matrix scaling, diagnostic precedence,
  geometry, object order, motion and G-code templates unchanged.
- Keep the Rust destination platform-neutral for browser WASM, Windows,
  macOS and Linux. No filesystem, UI, terminal or native viewer behavior.

## Included behavior

The included upstream behavior is automatic sequential static publication
for one physical head with one fixed nozzle. The ordinary master assignment
is a source-defined result even though the multi-head grouping engine is
not invoked. Used head/volume entries and the complete nozzle vector reach
the same applied-full export path as in the non-sequential case.

For the Ender-3 examples with master `1`, Standard volume and one filament,
the resulting maps are `filament_map = 1`, `filament_volume_map = 0` and
`filament_nozzle_map = 0`. These values follow from the upstream conversion
rules, not a replacement option default or a fixed footer constant.

## Excluded behavior

No new multi-head multi-object automatic solver support, dynamic selector,
mixed-filament grouping, multiple-nozzle inventory or top-level MatchMode
optimizer is included. Their current eligibility and diagnostics remain.
No Manual multi-object publication, process-vector port, prime-tower export
change, option registry change or generalized lifecycle refactor is included.
The existing single-head/static helper remains the destination for this
upstream slice; no independent Ares sequential pipeline is introduced.

## Public behavior checks

The existing byte-input tests retain their Native, validated byte API and
CLI comparisons, original fixtures and complete-output assertions:

- `byobject_owner_brim::byobject_owner_brim_combined_target`
- `byobject_owner_brim::byobject_owner_brim_width0_control`
- `byobject_perobject_multi_copy_brim::byobject_perobject_multi_copy_brim_combined_control`
- `byobject_perobject_skirt::byobject_perobject_skirt_combined_control`

Both Native modes must complete successfully on the original inputs.
Compare maps and complete streams without normalization or a private state
observer. The same public fixtures exercise distinct object-local brim,
multiple-copy brim and skirt behavior without substituting geometry or
switching the project to Manual mode.
