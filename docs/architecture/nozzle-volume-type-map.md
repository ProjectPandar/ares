# NozzleVolumeType config type-map rewrite

## Upstream boundary

Authority: OrcaSlicer v2.5.0, commit
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

- `src/libslic3r/PrintConfig.hpp:619–630`: `NozzleVolumeType` has six
  states. Their persisted integers are Standard 0, High Flow 1, Hybrid 2,
  TPU High Flow 3, E3D High Flow 5, Extra High Flow 6. Integer 4 is reserved.
- `PrintConfig.cpp:724–732`: `s_keys_map_NozzleVolumeType` binds all six
  wire names to these integers.
- `PrintConfig.cpp:6283–6324`: both `nozzle_volume_type` and
  `default_nozzle_volume_type` bind the shared type map and display all six
  values, labels, and integer keys, in that order.
- `PrintConfig.cpp:6363–6378`: `extruder_nozzle_volume_type` binds the same
  six-state type map, but its GUI list omits Hybrid: it lists the five
  physical types with integer keys 0, 1, 3, 5, 6.

The Rust destination is the existing option enum, registry definitions and
metadata, plus direct conversions of nozzle-volume names and integers.
This replaces the three-state approximation; it is not a new slicing pipeline.

## Loading is not GUI selection

`ConfigOptionEnumsGenericTempl::deserialize` (`Config.hpp:2269–2291`)
looks up each trimmed token in `keys_map`. It does not consult the GUI
`enum_values`. A missing key returns false. With forward-compatible
substitution enabled, `ConfigBase::set_deserialize_raw`
(`Config.cpp:686–700`) then replaces the entire option with its default,
not just its invalid element. A recognized type-map key must not trigger
this substitution.

Ares already distinguishes `OptionDefinition::enum_type_keys` from GUI
metadata in `options/json_deserialize.rs` and
`options/model_config_deserialize.rs`. Bind all three nozzle-volume
options to the complete shared type-key set. Keep unknown-key substitution
unchanged. In particular, Hybrid must load for `extruder_nozzle_volume_type`
even though it remains absent from that option's physical GUI choices.

The two logical/default GUI lists must also become six-state lists, including
labels and the non-contiguous integer keys. The physical GUI list already
matches upstream and remains five-state. No positional renumbering is allowed.

## Direct consumer audit

Paths below are relative to `crates/ares-core/src`.

| Rust consumer | Required treatment and upstream owner |
| --- | --- |
| `options/printer_options/remaining/enums.rs` | Add the three missing named states with explicit stable discriminants; retain vector serialization. `PrintConfig.hpp:619–630`, `PrintConfig.cpp:724–732`. |
| `options/registry/definitions/table/{pre_middle_defaults,late_tail_after_material,pre_middle_process}.rs` | Bind the shared six-key type map for the default, current, and physical options respectively. `PrintConfig.cpp:6283–6324,6363–6378`. |
| `options/registry/definitions/metadata.rs` | Update only the two logical/default GUI blocks from generator output; retain the physical GUI block. Same upstream definitions. |
| `options/extruder_index.rs` | Accept all five physical names. Do not turn Hybrid into a physical preset variant. `PrintConfig.hpp:680–692`, `PrintConfig.cpp:777–797`. |
| `options/update_printer_extruders.rs`, `options/update_printer_extruders/multiple_filament.rs` | Forward names to the existing index resolver; no independent whitelist or conversion to change. |
| `options/project_variants/index.rs` | Preserve each new physical name. Keep Hybrid-to-Standard preset lookup. `PrintConfig.cpp:10860–10863`. |
| `options/flush_matrix.rs` | Preserve new physical names for dataset lookup; keep the same Hybrid preset rule. Same upstream preset lookup boundary. |
| `options/flush_matrix/context.rs` | Typed vector reload/comparison inherits the complete enum. Standard remains the absent-option/resize default, not a replacement for known new states. |
| `options/different_extruders.rs` | Admit all six type names before comparing per-head values. `DynamicPrintConfig::is_using_different_extruders`, `PrintConfig.cpp:10771–10805`. Do not broaden different-extruder slicing support. |
| `options/project_variants/filament_maps.rs` | Retain the enum's actual integer in static volume maps. `MultiNozzleUtils.cpp:319–369`, `ToolOrdering.cpp:1752–1776,1869–1908`. |
| `project_slice/gcode_emit/tool_ordering/nozzle_group.rs` | Retain the same actual integer in static physical groups. `MultiNozzleUtils.cpp:319–369`. |
| `project_slice/gcode_emit/tool_ordering/filament_group/context.rs` | Recognize the new physical suffixes when resolving variants. `convert_to_nvt_type`, `PrintConfig.cpp:912–935`. Retain existing unsupported-context boundaries. |
| `project_slice/gcode_emit/tool_ordering/filament_group/context/inventory.rs` | Already recognizes all six names and stable integers; keep physical inventory checks unchanged. `PrintConfig.cpp:940–972`. |
| `options/config_export/full_config.rs` | Uses the canonical variant resolver; inherits its new physical names without an extra conversion. |
| `options/{gcode_fields,project_runtime_options/wire,project_runtime_options/gcode_source/wire,printer_options/wire,printer_options/remaining/wire}.rs` | Existing typed vector serializers inherit the enum additions; defaults remain Standard. |
| `options/typed_legacy/actions/rules.rs`, `options/legacy/token.rs` | Existing Normal/Big Traffic aliases and structured-token handling do not rewrite the new names; no changes. |
| `options/registry/key_lists.rs`, `profiles/{project_refresh/keys/machine,catalog/includes/defaults}.rs`, public re-exports | Key membership/ownership only; no state-specific logic or edits needed. |

## Included and deferred behavior

Included: complete enum representation, shared config admission, exact wire
spelling, stable integer conversion, source-equivalent GUI metadata, and
existing direct preset/static-group name conversions.

Deferred: additional automatic multi-nozzle planning, TPU-specific assignment
policy (`FilamentGroup.cpp:1094,1342`), unsupported inventory contexts,
perimeter/support behavior, and unrelated registry-generation drift. Existing
bounded grouping/slicing scaffolding remains a temporary compatibility shell;
this enum slice does not claim parity with those deferred upstream algorithms.
Do not add a fallback that maps a recognized physical type to Standard.

## Generated metadata

Verify the `OrcaSlicer` source checkout is at the authority commit above.
Generate the complete metadata without editing production files first:

```sh
node scripts/gen-configdef.mjs --dump > "$CONFIGDEF_JSON"
node --input-type=module -e '
import { readFileSync, writeFileSync } from "node:fs";
import { renderMetadata } from "./scripts/gen-configdef/output.mjs";
writeFileSync(process.argv[2], renderMetadata(JSON.parse(readFileSync(process.argv[1], "utf8")).options));
' "$CONFIGDEF_JSON" "$CONFIGDEF_METADATA"
```

This uses the entry point's complete source parse/projection and its unchanged
metadata renderer. If only the two target keys differ, use the complete output.
If other keys drift, replace only the two target blocks, byte-for-byte from that
output, and report the remaining drift separately without fixing it here.
`metadata.rs` is an existing generated-file exception to the module line limit.

## Test-first acceptance

Before implementation, add public-API differential tests using Native CLI
`--export-settings` (`OrcaSlicer.cpp:6326–6334`) and runtime-mutated project
settings. Native serialization of the complete tested option vector is the
reference, not a fitted hash or a hand-authored replacement producer.

Failure modes to exercise before adding isolated assertions:

1. Each of the three new valid names is silently defaulted or rejected.
2. An existing name changes spelling, ordering, or its stable integer.
3. Integer 4 is accidentally assigned to E3D High Flow.
4. Physical GUI membership is incorrectly reused as config admission: Hybrid
   is accepted by Native's shared map despite being absent from that GUI list.
5. An invalid token stops replacing the whole option with its Native default.
6. A direct name/ID consumer rejects, erases, or misnumbers a known new type.
7. GUI labels/values/keys diverge or assume contiguous positional integers.

Cover all six states through Native config export and public Rust project
loading/config codecs, checking typed discriminants and vector round trips.
Keep existing states and invalid-token substitution as unchanged controls.
Check both public GUI metadata lists and the physical-list exclusion separately.
Exercise direct public index/comparison consumers where exposed; retain
relevant static-map, grouping, flush, and preset regressions.

Run identical relevant subsets on the exact base and final implementation in
both Native producer modes; stop on a passing-to-failing transition. Preserve
raw failing results rather than weakening existing comparisons. Run formatting
and Clippy on the final implementation. No new dependency, framework, fixture
fitting, or unrelated full-workspace differential run is part of this slice.
