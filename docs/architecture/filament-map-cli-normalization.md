# CLI filament-map normalization and cardinality

## Upstream rewrite boundary

This slice ports `OrcaSlicer.cpp:6027-6034` before `Print::apply`, together
with the overlong-map admission exercised by that entry path. Its Rust
normalization destination is `options/project_variants.rs::expanded_filament_map`.
The additional API admission boundary is
`project/effective_config/cardinality.rs::validate`.

`PrintApply.cpp:1162-1172` materializes variants, and `1184-1185` collects
`filament_map` without rejecting an overlong vector. The multiple-filament
materializer explicitly uses the map's size (`PrintConfig.cpp:9817-9837`).
An exact map-length-equals-diameter-count gate is therefore not this upstream
entry contract. The existing Ares equality gate is replaced by a minimum
length gate, not by input truncation.

## Non-negotiable decisions

- Grow a short map to `filament_count` using default `1` only when shorter.
- Never truncate an overlong map at this normalization boundary.
- For one physical extruder, set only entries `0..filament_count` to `1`.
  Do not overwrite the overlong tail.
- Require enough map entries for the logical filaments. Check raw source IDs
  before normalization can reset single-extruder slots, preserving the existing
  API positive/in-range gate (`print_applied_map_invalid_input_still_rejected_before_print`).
  Keep the materialized entry-range and variant-index bounds checks as well.
- Refresh recommended-map variants only in auto mode, as in
  `ToolOrdering.cpp:1291-1303` / `Print.cpp:2485-2492`; within that mode refresh
  when the applied map differs (`Print.cpp:3166-3175`). A manual map must not
  be rematerialized by this recommendation pass.
- Assign the recommended map exactly during refresh (`Print.cpp:3171-3175`).
  Do not reuse the earlier CLI grow/reset operation here. ToolOrdering sizes its
  recommendation from `filament_colour` (`ToolOrdering.cpp:1114,1151`), which
  need not have the same count as the configured filament-diameter records.
- Preserve current normalized non-filament settings during refresh. Upstream
  `Print.cpp:3174-3195` restores `m_full_print_config` from its checkpoint but
  updates runtime `m_config` using only filament variant keys and retract
  overrides. Replacing the entire Ares views from the earlier checkpoint
  undoes late `normalize_fdm_2` (notably re-enabling the prime tower). The Ares
  full/runtime views remain a temporary compatibility shell: the existing
  37-field materializer now accepts separate source and destination settings.
  Read exactly `filament_options_with_variant` (`PrintConfig.cpp:8375-8414`)
  from the checkpoint, select variants, and assign only those fields into
  current normalized settings. Do not copy the entire filament family or
  reconcile arbitrary fields. Existing config export retains raw
  normalized-option values; do not introduce a new snapshot layer.
- Do not add a fallback at an error or change variant selection. The raw-ID
  guard is an existing Ares API admission constraint, not a claimed upstream
  validation port.
- `expanded_filament_map` remains a temporary compatibility-shell name around
  the cited CLI normalization; it is not a new Ares mapping policy.
- This platform-neutral logic remains within `ares-core`: no filesystem,
  process, terminal, or platform-dependent behavior is added.

## Refresh operation mapping

| Upstream | Rust step / config object |
| --- | --- |
| `PrintApply.cpp:1165-1172` | Existing `filament_variant_checkpoint`, captured after printer/process variant selection and before filament selection, represents `m_ori_full_print_config`. |
| `ToolOrdering.cpp:1291-1303` | `build_recommended`: refresh only for auto mode, after layer usage is known. |
| `Print.cpp:3168,3171-3172` | Compare the applied/recommended maps; `refresh_filament_map` assigns the supplied recommendation unchanged to the cloned current settings' map, without CLI-entry regrowth. The new map is passed explicitly to variant selection rather than mutating the saved checkpoint. |
| `Print.cpp:3174-3175` / `PrintConfig.cpp:9817-9863` | Read raw checkpoint values and resolve the new map's variant indices. |
| `Print.cpp:3191-3192` | `filament::materialize(destination, checkpoint, indices)` selects and assigns exactly the existing 37 variant-key fields, leaving process/printer/non-variant filament fields untouched. This fuses materialization into the `apply_only` destination without copying unrelated checkpoint values. |
| `Print.cpp:3177-3188,3193-3195` | Existing `resolve_project_config_views` computes retract overrides from the newly selected filament keys and unchanged machine settings, then constructs effective runtime/G-code views. The override computation is pure; its inputs are unchanged by assignments to the disjoint filament variant keys. |

Native disables the TiQ2 tower earlier: GDB records the cold normalization as
`(num_objects, used_filaments) = (0, 0)` and the late call as `(1, 1)`.
`PrintApply.cpp:1620-1621` gathers used filaments and calls `normalize_fdm_2`;
`PrintConfig.cpp:8703-8706` takes the single-used-filament branch and sets
`enable_prime_tower = false` (without smooth timelapse or wrapping).
`PrintApply.cpp:1651` applies those changed keys to runtime `m_config`.
The later `Print.cpp:3191-3192` key set does not include `enable_prime_tower`
or `independent_support_layer_height`. Ares already performs the corresponding
late `normalize_fdm_2` in `effective_config/phases.rs`; this correction stops
replacing its result with an earlier checkpoint.

## Included and deferred behavior

Included: CLI grow-only behavior, the single-extruder prefix reset, admission
of valid overlong vectors, and strict end-to-end Native/API/CLI verification.
Deferred: changing which physical IDs the API accepts, malformed map recovery,
variant fallbacks, and unrelated PrintApply behavior. Manual recommendation
refresh remains excluded, rather than introducing a new manual-map policy.

## Verification

The frozen X1C cube has one consumable/one nozzle and plate map `1 1`.
The canonical `runner::native_stream_case::check` registration has no opt-in,
seeded-only skip, expected mismatch pin, or bespoke runner. Official and seeded
modes each qualify two Native outputs before strict complete generator-line-only
API/CLI comparisons and raw API/CLI equality. The richer KSR's known official
object-ID nondeterminism does not grant this distinct fixture an exception.

The failing public API was independently observed in GDB entering the
cardinality owner with map length 2, diameter count 1, and nozzle count 1;
that owner returns the reported `InvalidInput` before slicing. RED precedes
any production edit.

For the TiQ2 regression, the existing input-rejection and two TiQ2 canonical
checks are mandatory official+seeded controls alongside the overlong-map witness;
no assertions or fixture bytes change. A Native GDB probe at
`Print::update_filament_maps_to_config` confirms the TiQ2 auto-map update runs
from `ToolOrdering.cpp:1303` with runtime `enable_prime_tower == false`, while
its exported config still records `enable_prime_tower = 1`. Thus auto-mode
filtering alone is insufficient: preserving normalized runtime state is the
required correction.

## Entry growth versus late recommended-map assignment

The frozen Ender-3 refresh witness has two filament-diameter records but one
filament colour and an initial map `[1]`. CLI entry grows the map to `[1,1]`
(`OrcaSlicer.cpp:6027-6034`). Later, ToolOrdering constructs a one-entry map
from the colour count (`ToolOrdering.cpp:1114,1151`), converts its IDs to one-based,
and supplies `[1]` to `Print::update_filament_maps_to_config` (`:1302-1303`).
`Print.cpp:3171-3175` retains that exact vector; it does not repeat CLI growth.
Rust's `refresh_filament_map` must likewise preserve the recommendation, while
initial `materialize_project_variants` keeps the CLI grow-only operation.

The config footer is not a duplicate-ID compression policy.
`GCode.cpp:5591-5643` serializes the full print config via
`ConfigBase::opt_serialize` (`Config.cpp:543-547`) and the ordinary
`ConfigOptionIntsTempl::serialize` comma-join (`Config.hpp:1023-1032`).
Consequently `[1]` exports as `1`, and `[1,1]` exports as `1,1`.
No serializer, Manual gate, variant-selection algorithm or admission rule changes
are included in this refresh correction; the existing views remain a temporary
compatibility shell around the cited upstream config owners.

### RED/GREEN qualification contract

Freeze the existing two-diameter/one-colour project bytes and SHA without changing
its configuration. The canonical Native-double-replay/public-API/CLI witness
must assert Native diameter count 2, colour count 1 and map `[1]`, then compare
complete output with the existing generator-line-only comparator. Run the witness
with genuine official and seeded producers, without an opt-in or seeded-only
skip.

The older `bookkeeping_export` test is **not** an official agreement claim:
its lines 22-25 return with `seeded-only reference` unless the seeded-reference
environment is active. Its official PASS therefore skips the Native comparison.
Keep that existing guard unchanged and exercise its four subcases in seeded mode;
only the new unguarded frozen witness establishes a real official comparison.
Retain all earlier filament-map, admission and TiQ2 controls.
