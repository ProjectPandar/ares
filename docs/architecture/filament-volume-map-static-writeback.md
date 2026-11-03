# Static filament/nozzle volume-map write-back

This is a source-cited libslic3r configuration and static nozzle-map rewrite,
not another mixed-metadata resize or an Ares-owned slicing feature. Upstream is
pinned to `345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Source boundary

The first included layout is non-sequential, non-mixed, one physical extruder,
one static nozzle and one non-Hybrid volume type. Auto For Flush, Auto For Match
and Manual use their upstream static cases; no grouping solver is invented.
Single-volume High Flow is distinct from Standard and must not be forced to0.
Multi-variant filament/process expansion is outside this first layout.

- `src/libslic3r/PrintConfig.cpp:3118`, `PrintConfigDef`: `filament_volume_map`
  is non-nullable `coInts`, default `ConfigOptionInts{nvtStandard}`. The registered
  singleton default remains unchanged. It is a per-filament nozzle-volume-type
  assignment, not a volume amount, physical-extruder ID or mixed-slot flag.
- `GCode/ToolOrdering.cpp:1752–1776,1869–1871`,
  `ToolOrdering::get_recommended_filament_maps`: the ordinary static result is
  created from the source filament-to-extruder assignment, nozzle list and actual
  used-filament set. Manual mirrors its configured map. The one-extruder auto
  case uses the configured master extruder, not a new used-tool selector.
- `GCode/ToolOrdering.cpp:1892–1908`, `build_default_nozzle_list`: a physical
  extruder contributes one logical nozzle with its diameter, extruder/group IDs
  and volume type. A missing volume-type column uses Standard as source does.
- `MultiNozzleUtils.cpp`, `LayeredNozzleGroupResult::create` for the static
  filament-map/nozzle-list overload, and `:316–369`, `get_extruder_map`,
  `get_nozzle_map`, `get_volume_map`: project the same source assignment into
  complete correlated maps. Preserve 1-based extruder IDs, 0-based nozzle IDs
  and source unassigned sentinels; do not conflate their coordinate systems.
- `GCode/ToolOrdering.cpp:3054–3084`: the non-sequential static result is written
  back in every map mode when derived maps are nonempty, including unchanged
  Manual assignments. Correctly sized config maps are the merge base. A wrong-
  sized extruder base becomes all1; a wrong-sized volume base becomes all
  Standard. This is replacement, NOT truncate/grow or first-element broadcast.
- `FilamentGroupUtils.cpp:346–358`, `update_used_filament_values`: copy the base
  and replace only valid used indices from the derived vector. Unused entries
  retain their base assignment. Preserve its explicit source index bounds.
- `Print.cpp:4656–4678`, `Print::update_filament_maps_to_config`: compare the
  correlated map triplet and synchronize the original full config and runtime
  config. Nonempty volume maps replace the vector; an empty supplied volume map
  resizes by the supplied filament map with Standard growth. Nonempty supplied
  nozzle maps replace their corresponding vector.
- `Print.cpp:4680–4745`: without a supplied per-filament volume map, existing
  slots are backfilled from their mapped extruder's volume type. Rebuild the
  source full-config snapshot and retain existing variant/retract resolution.
  The included one-extruder/single-volume layout does not add a new selector.
- `GCode.cpp:8244,8338–8351,4884–4889`: the sorted CONFIG_BLOCK serializes applied
  full-config values through the existing type-owned integer-vector writer.
  A matching printed line is an observation of this state, not its producer.

## Adjacent mechanisms and exclusions

`PrintConfig.cpp:11909–11928`,
`DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments`,
trusts a per-filament volume map only when its length equals filament count;
it can override volume type for multi-volume/Hybrid selection. That consumer's
new Hybrid/multi-volume selection is deferred. The first layout has one physical
volume and must not be represented as qualified Hybrid support.

`OrcaSlicer.cpp:6829–6866` synthesizes Manual volume maps only in its multi-nozzle,
explicit-plate context. It derives single-volume assignments from extruders and
retains per-filament choices on Hybrid extruders. Nozzle Manual requires all
explicit maps. These gates and producers are deferred, not unconditional CLI
loaded-count growth for ordinary inputs.

`OrcaSlicer.cpp:7117–7134`, including `:7126`, reads final engine maps back into
plate metadata after processing, gated to multi-extruder printers and source
map modes. Plate export/persistence is deferred. Core gains no filesystem or
project-export lifecycle, and this slice does not reopen temperature snapshots.

Also defer BBL multi-extruder grouping, multi-nozzle/Nozzle Manual modes,
per-layer or dynamic selectors, sequential/per-object stitching, compatibility
optimization, mixed virtual-filament expansion and GUI preset composition.
`Print.cpp:4798–4818`'s static result write-back establishes the shared source
semantics; its dynamic expansion and additional callers are not included.

The completed seven-member `resize_mixed_filament_metadata` helper remains
independent and unchanged. Its loaded-count gate and false/empty growth do not
own or implement any of the above volume-map producers.

## Rust destination and temporary scaffold replacement

The existing `project_slice/gcode_emit/tool_ordering/recommend.rs::build_recommended`
collects actual layer usage, but refreshes configuration only for its current
changed/auto-map condition. That condition is a temporary compatibility shell,
not the complete current upstream non-sequential map write-back. Replace that
included-layout behavior with the cited static map projection and source
triplet comparison, even when the extruder assignment itself did not change.
Preserve tool ordering, extrusion paths, initial-tool choice and M140 behavior.

Put the bounded source map projection and `update_filament_maps_to_config` /
`update_used_filament_values` logic in a private
`options/project_variants/filament_maps.rs` module. Its map payload is only the
static projection of `LayeredNozzleGroupResult`, not a new grouping engine or
policy. The source nozzle list and used set determine its values; no fabricated
empty group result or hardcoded zero-filled assignment substitutes for them.

Extend the existing `options/project_variants.rs::refresh_filament_map` and
`MaterializedProjectVariants` / `ProjectConfigViews` plumbing for the correlated
original/full/runtime map state. These Ares views remain temporary compatibility
carriers around `Print::update_filament_maps_to_config` and its source snapshots,
not a separately designed pipeline. Keep their existing variant/retract
application order and original-index retention; do not duplicate resolution.

Represent needed applied map vectors through the existing `OrcaInts` and
project G-code option/wire fields (`options/gcode_fields.rs`), preserving complete
values, signed sentinels, ordering and canonical arrays. Only directly necessary
field ownership metadata may change; registered type/default/count remain
unchanged and registry regeneration is excluded.

`options/config_export/full_config.rs` and the existing `write_config_block`
call must consume the synchronized applied full view. Preserve banned/internal
keys and generic integer serialization. No late raw/default fallback, rendered-
line substitution, special-case `filament_volume_map = 0,0`, or separate CLI-only
repair. Core remains browser WASM/Windows/macOS/Linux byte/API logic; Native
processes and file artifacts belong only to the test adapter.

## Failure inventory

1. A singleton registry default is widened instead of applying source map state.
2. Logical-filament, used-filament, physical-extruder or nozzle counts are mixed;
   an unused slot disappears or becomes a used-slot-derived duplicate.
3. A wrong-sized base is grown while retaining an invalid old prefix, rather
   than replaced as the static write-back caller specifies.
4. A correctly sized base loses its unused assignments, or an unassigned derived
   sentinel leaks into unused slots instead of the used-entry merge.
5. Standard0 is forced for a qualified High Flow case, or extruder ID1 is
   mistaken for its volume type or 0-based nozzle ID.
6. Only changed/auto extruder maps trigger write-back; unchanged Manual or auto
   assignments leave volume state stale.
7. Original, full, runtime, checkpoint or CLI/API views diverge, or retract/
   variant state is recomputed in the wrong order.
8. A generic serializer or output token is changed to hide a missing producer;
   matching the selected field is misreported as complete-output parity.
9. Deferred Hybrid/Nozzle Manual, multi-nozzle or sequential behavior is silently
   treated as qualified by the included ordinary-layout evidence.

## Original-input public qualification

The two existing authored inputs are shared independently with Native and Ares:

- `tests/parity/m140-initial-bed/nonzero-initial-tool.3mf`, SHA256
  `44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a`.
- `tests/parity/m140-initial-bed/unused-hotter-filament.3mf`, SHA256
  `893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0`.

Both authored archives specify by-layer, Auto For Flush, one 0.4mm physical
extruder, Standard volume, filament map1/1 and two logical colors. Neither
provides volume/nozzle maps. Actual repeated Native model activity only on
logical filament1 must remain independently qualified. Applied output has two
volume entries even though only one filament prints.

Contrast witnesses must independently qualify a valid single-volume High Flow
case and preservation of a distinct correctly sized unused assignment. These
may be combined only if Native establishes both properties; otherwise they are
two authored-settings-only contrasts. Successful stable Native behavior must
establish the complete expected maps and used sets; expected vectors are not
inferred by repeating0 or1. Native rejection or unstable output does not qualify
a public implementation failure.

Qualified Native first/replay must succeed with the existing exact generator
normalization; Ares consumes original bytes,
never Native exports. Require API success, actual CLI0 and raw API=CLI, complete
applied vector equality, and unchanged strict complete-output comparisons.
The strict tests must retain exact first-difference positions and original bytes.
A field/vector or static-map observation is labelled separately and cannot
establish whole-output parity or silently include a later differing field.
Artifacts retain original input hashes, source/provider identities, actual map
mode/layout/used set, complete maps, Native command statuses, API/CLI bytes and
strict comparison results. Failed setup has no Native/parity coverage credit.
