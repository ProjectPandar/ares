# Static FlushMode engine: source boundary and public failing-test contract

## Decision

Port the static, non-TPU dual-BBL FlushMode call chain from OrcaSlicer
2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`. This is a refinement
of the [enumeration ARD](2026-11-06-beta-filament-group-flush-auto-map-ard.md)
and [implementation plan](2026-11-06-beta-filament-group-flush-auto-map-implementation-plan.md),
not an independent optimizer. No production engine is added by this spec.

Included: Auto For Flush, two physical heads with one nozzle/volume type per
head, actual by-layer usage or one-object by-object usage, source deduplicated
estimate below `10000`, no mixed filaments, switcher or preload. Preserve both
`group_algo_with_time` values. Manual, single-head, non-BBL and TPU behavior
remain independent controls. Dynamic/multi-nozzle, top-level MatchMode,
K-medoids, multi-object sequential production and nonempty inventory are
outside this verified slice; do not silently return an uncomputed seed or
all-master result for unsupported automatic contexts.

## Upstream ownership, in execution order

Paths below are relative to `src/libslic3r/`. The owning wrapper is
`GCode/ToolOrdering.cpp::ToolOrdering::reorder_extruders_for_minimum_flush_volume`
(`:2849`), not footer export or retraction emission.

| Source function/boundary | Required Rust destination |
| --- | --- |
| `ToolOrdering.cpp:2893–2917,3053–3084`: collect actual layers/used IDs, obtain restrictions, static dispatch and publication | `project_slice/gcode_emit/tool_ordering/recommend.rs::build_recommended`, after `collect_layers`, before `finish_layers` |
| `Print.cpp:3593–3604,4908–4976`; `PrintObject.cpp:349–515`: geometry union, physical masks, declaration-based volume restrictions | Private `tool_ordering/filament_group/constraints.rs` and `constraints/geometry.rs` |
| `ToolOrdering.cpp:1477–1672`: `prepare_flush_matrices`, `build_nozzle_groups`, `build_filament_group_context` | Private `filament_group/context.rs` |
| `ToolOrdering.cpp:1752–1872`: `get_recommended_filament_maps`, non-TPU automatic dispatch | `recommend.rs` plus private `filament_group/mod.rs` |
| `FilamentGroup.cpp:1036–1051,1253–1268`: `calc_filament_group` → `calc_filament_group_for_flush` | Private engine entry; FlushMode only, no MatchMode exception fallback |
| `FilamentGroup.cpp:901–922,36–374`: `calc_min_flush_group` → estimate → `calc_group_by_enum` → `full_evaluate` | Private `enumeration.rs`, `score.rs`, `sequence/{ordering,change_time}.rs` |
| `ToolOrderUtils.cpp:797–1141,1324–1494`; `MultiNozzleUtils.cpp:467–661`: cost-only order, carried layer state, sequence assembly and actual change-time simulation | Private sequence scorer, without altering emitter path ordering |
| `FilamentGroup.cpp:427–566,1253–1268`: heap retention, primary-first candidate composition, `select_best_group_for_ams` | Private `remembered.rs`, `ams.rs`; empty-inventory behavior only is runtime verified |
| `ToolOrdering.cpp:1681–1747`: `apply_master_extruder_preference` and `can_swap_extruder_groups` | Wrapper after solver/AMS selection, not a solver tie-break |
| `MultiNozzleUtils.cpp:144–163,319–369`: full assignment and explicit head/volume/global-nozzle conversion | Private `result.rs` |
| `ToolOrdering.cpp:3066–3081`; `FilamentGroupUtils.cpp:346–358`; `Print.cpp:4656–4753` | Used-entry merge, publication, existing refresh and config export in `recommend.rs` |

## Replace temporary Ares behavior, not downstream symptoms

`project/effective_config/phases.rs` currently captures the applied map, prepares
CLI AMS counts and invokes `apply_recommended_filament_map` before actual layers
exist. The included dual-BBL branch in `options/filament_map_recommend.rs` returns
all-master. Do not run that early computation for this source boundary; retain the
applied snapshot, normalization, materialization, AMS preparation and independent
control routes. Do not replace the shell with another seed-based optimizer.

At the late boundary, `build_recommended` currently calls TPU handling and
`update_static_filament_maps`. The latter accepts only one physical head and is
not a dual-head solver. Add the real automatic solver at the late boundary while
preserving that existing single-head/static writeback and its unconditional
refresh contract. Keep `crate::print::validate::project_is_bbl` for selected/system
identity rather than reconstructing identity from a model-name prefix.

Do not repair this discrepancy in retraction, ironing, stats, headers or getters.
The stats runner is a downstream observation when the engine is passing, not a
second mapping implementation or a promise of time-header/full-output parity.

## Non-negotiable data and numeric contracts

- Real restrictions are mandatory: first-forbidden physical-head masks, volume
  declarations/self IDs from the retained full checkpoint, actual layer print Z,
  region role/entity presence, instance transforms and plate origin. Use actual
  fill polygons and raw-slices-minus-fill polygons with source intersection;
  bounding-box tests alone and empty constraint stand-ins are not ports.
- The existing horizontal-shell envelope retains `fill_expolygons`; classic
  traversal alone does not. Borrow that envelope through `recommend.rs` and
  `layers/schedule.rs` for the geometry consumer. Do not clone retained geometry
  or widen APIs.
- Existing bed-exclusion parsing/intersection remains read-only. Its public
  parser accepts `DynamicPrintConfig`; constructing that adapter with `new`
  serializes the entire settings object. Reuse that config adapter and its
  registered wire tokens for restriction inputs. Do not introduce a second
  adapter, duplicate parsing or widen retained-data access.
- Decode nozzle statistics with `PrintConfig.cpp:724–731,940–972` and the
  explicit `PrintConfig.hpp::NozzleVolumeType` values (index `4` is reserved).
  Recognize all six names; reject valid types outside Standard/High Flow
  static grouping rather than dropping them. Counts consume signed C-locale
  decimal prefixes. Tokens with fewer than two fields or unknown names skip.
- Distinguish logical filament IDs, physical heads, nozzle-list indices, global
  nozzle groups and materialized columns. Enumerate lowest-position-fastest,
  retain source fixed-width hash/heap ties and preference-before-cost ranking.
- Engine matrices partition by physical head, cast raw entries to `float`, apply
  the selected Default/Fast double multiplier and round to `float` once. Resize
  multipliers with `1`; do not apply footer integer rounding or scale twice.
- `full_evaluate` always includes actual load/unload simulation. The wrapper's
  print-time table is empty for either time flag. Read and preserve that flag;
  do not replace simulation with switch counts or generic machine-switch time.
- The static call at `ToolOrdering.cpp:3054` supplies no initial nozzle-status
  argument; its default is empty. The grouping callback at `:1808–1824` reads
  other-layer ranges in reverse precedence. Do not substitute the outer final
  reorder callback at `:2970–3004`, inject live loaded state, or reset carried
  state between scored layers. Preserve actual collected first-layer order.
- Remember source gap rules: relative `0.01`, absolute `10` for zero best cost,
  preference-only retention and source heap extraction. Primary precedes the
  remembered array for AMS first-minimum selection. CLI inventory lacks
  `tray_name`; capacity is not usable colour inventory. Do not fabricate trays
  or claim nonempty matching verification from these tests.
- Use the unchanged `refresh_filament_map(checkpoint, map, current)`. Merge used
  head/volume entries into correctly sized seeds; synthesize defaults only on
  length mismatch. Publish the full derived nozzle map directly. Compare all
  three maps and refresh even if heads are unchanged. Prepublish into current
  and checkpoint, rebuild columns/nullable overlays, regenerate the config
  block, and preserve the immutable application-time map used by ironing.
- Protected filament/printer materializers, motion, custom G-code, toolchange,
  wipe, Z-hop and printable-area validation are not edited by this slice.
  Preserve exact malformed-matrix text and its precedence before dispatch.

## Failing tests and acceptance

The seam remains fixed 3MF bytes → validated public API/CLI output, compared
against runtime Native output after verified double replay. Native literals
are premise checks, not Ares expected-output goldens. Do not change existing
fixture bytes, assertions, order, skips, mode or validation outcomes.

| Witness | What it discriminates; what it does not |
| --- | --- |
| Original six producer identities plus four context cases | Physical assignment/publication, Fast matrix selection and flag-0 load-time effects; all ten maps must become passing |
| Three unused-entry cases | Preserve unused head/volume seeds and publish full derived nozzle map; no independent volume-restriction scoring credit |
| Existing tie/bias, mask and custom/prior tests | Traversal preference, masks and carried ordering; a shape-only failure is not scoring failing |
| [Remembered map pair](2026-11-06-beta-flush-remembered-map-pair.md) | Native `111→121` versus all-master `111→111`, both endpoints sampled before map equality; composed remembered/empty-inventory seam, not isolated heap/relative-gap/nonempty AMS coverage |
| Exclusion-prism case and paired second-head height/area cases | Real geometry-driven assignment; three original first-head validation errors remain separate residuals, not successful slices or scoring failing |

An exploratory Standard/High Flow declaration pair returned the same Native
map and effective variants at both endpoints. It does not establish a public
volume-restriction failing. The declaration/import checkpoint boundary and any
additional shared access require investigation before that mechanism is
claimed; do not reinterpret unchanged maps as verification or broaden scope.

Preserve all 23 retraction identities, both public controls, exact malformed
controls, four independent import tests and nominal strict matrix comparisons.
Do not normalize other differences or infer whole parity from fields.
Files stay at most 400 LOC, with no unused engine scaffolds.
