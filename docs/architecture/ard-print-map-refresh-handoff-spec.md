# Print::update_filament_maps_to_config: conditional late-refresh handoff

Date: 2026-10-10. Status: proposed, offline only; no production change.
Rust tree audited: `f2cdadb05e72147cf39b25c2c20bc8e482fe83a1`, before the
actual-used map correction.

## Owning upstream state transition

`Print.cpp:3166-3200`, **Print::update_filament_maps_to_config**, is the
map-changed publication/refresh boundary, not the recommendation algorithm:

1. Only when incoming `f_maps` differs from `m_config.filament_map`, update
   both original-full map and runtime map.
2. Reset `m_full_print_config` from `m_ori_full_print_config`; select the
   filament variant fields with the new physical mapping through
   `PrintConfig.cpp:9805+`,
   **DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments**.
3. For each `print_config_def.extruder_retract_keys()`, compute the mapped
   nullable override against the new full-machine defaults and old runtime
   values (`PrintConfig.cpp:10300-10336`, **compute_filament_override_value**).
4. Apply the filament variant key set to runtime. Only changed retract keys
   update placeholder configuration and runtime overrides. Mark the auto-map
   result even if the map did not change.

The original-full snapshot is **not** raw authored project JSON:
`PrintApply.cpp:1165-1172` already materializes printer variant-one,
printer variant-two and process ID/variant fields **before** saving
`m_ori_full_print_config`; only filament variant selection follows the save.
Consequently this late transition does **not** rerun process identity lookup,
change master selection, recommend another map, or repeat ToolOrdering.
The initial process-ID behavior (`2,1` request -> `1,2` physical view) remains
owned by `PrintConfig.cpp:9004-9055,9634-9683`, a separate initial-apply slice.

Filament selection asks for `(logical filament index+1, active variant of the
mapped physical extruder)`; the physical map ID is not itself the logical
filament-self ID. See `PrintConfig.cpp:9832-9858`. Do not conflate those IDs.
Source miss/short-vector handling must be reproduced only on a separately
observed admitted case; this document permits no Ares fallback.

## What the current A2 evidence does and does not prove

Historical A2 on `c948c292a` / main `f45b81941`:
Native `native/plate_1.gcode` versus `ares.validated.gcode` already agree on:

- print IDs `1,2`, paired quoted `Direct Drive Standard` process variants;
- paired quoted `Direct Drive Standard` filament variants;
- `extruder_type = Direct Drive,Direct Drive`,
  `nozzle_volume_type = Standard,Standard`;
- `retraction_length = 0.8,0.8`, `retraction_speed = 30,30`,
  `deretraction_speed = 30,30`.

Maps differ (`2,1` Native, `2,2` Ares). This is NOT evidence of an independent
process-variant or retraction RED, nor proof that fixing the map closes the
whole stream. Do not attribute the first duration-header mismatch to this
refresh without the post-correction unchanged-input residual.
These are offline reads of historical outputs, not new Native execution.

## Conditional included slice and exact Rust destination

After the GREEN/admission correction, retain A2's same input and strict
whole API/CLI comparators. If a residual implicates refreshed values, report
its exact field/first byte and destination before production edits:

- `crates/ares-core/src/options/project_variants.rs`: existing initial
  printer/process-before-filament ordering and original-full checkpoint.
- `options/project_variants/index.rs::resolve_filament_indices` and
  `options/project_variants/filament.rs::materialize`: select from the
  pre-filament checkpoint using the accepted new map, **not** from an already
  collapsed filament view. `printer.rs::materialize_process` is initial
  identity only; do not invoke it again just because the map changed.
- `options/project_config_views.rs::resolve_project_config_views` and
  `options/project_config_views/retract.rs::apply`: fresh full/runtime/
  runtime_gcode views with mapped nullable retract overrides from full
  machine defaults, not from previously overridden runtime vectors.

The current `materialize_project_variants` returns only the collapsed result;
`ProjectConfigViews` stores full/runtime/runtime_gcode but no original-full
checkpoint. For a genuinely changed active filament variant, lifetime capture
of the upstream checkpoint is required. Its smallest representation/capture
seam must be specified before adding state or changing callers.
Do not reconstruct it from exported raw input, replay profile inheritance,
re-run initial materialization on collapsed vectors, or add a second slicing
pass as a substitute. No checkpoint/state change is made now.

If actual A2 proves filament selection invariant under its map change, a
bounded retraction-only residual may reuse its existing full view and the
existing view resolver; prove this invariant on both producers first. Do not
silently generalize that optimization to different active variant flavors.

The late map handoff supplies the accepted one-based map after
actual-layer/dontcare collection and before reorder. This slice returns/refills
refreshed config views before template/file-start consumption; existing
`write_config_block` serializes those views, never patches the footer text.
The necessary invocation/lifetime seam in the staged map caller is a
**fixed handoff**, not license to edit the map-recommendation files.
`project/effective_config/phases.rs`, schedule/emit integration, public
`SliceError` and CLI `failure.rs` are shared: specify the exact proposed diff first.

## Deferred, tests-first, and nonnegotiable fences

Deferred: recommendation/master admission/TPU shortcut and printability,
ToolOrdering collection/reorder/partitions, AMS/ordinary-group solver,
geometry/flow algorithm changes, generalized refresh contexts beyond the
observed residual, new public APIs and defensive validation. The
`filament_map_recommend` and `tool_ordering` files are not edited here.
The current phase scaffold is temporary upstream compatibility state, not
an independent Ares pipeline to extend with invented refresh passes.

Failure checklist before code: wrong checkpoint stage; reselection from
collapsed vectors; stale map in one view; double nullable override; logical
ID confused with physical map ID; repeated process rematerialization;
placeholder vs runtime disagreement; footer rebuilt from stale full;
unchanged-map doing unnecessary reselection; earlier guard/error reclassified.
Use the preserved A1/A2 E2E inputs/strict channels/full JSON/no-export or
whole-Gcode comparisons without altering them. Additional control, only after
an explicit scope amendment: same map, mapped nil/default versus explicit retract value,
and an admitted changed-flavor case if checkpoint capture is actually needed.

No compile/setup failure counts as RED, no fallback/extra normalization/new
golden/cache/sweep hook, no core I/O/process/shell. Required Native absence
fails by name; optional absence visibly skips. Keep modules <=400 LOC.
A runtime residual is a
scope barrier, not permission to automatically implement this spec.
