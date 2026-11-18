# Type1 last-change height eligibility

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `src/libslic3r/GCode/WipeTower.cpp`, owns this rule. Type1 framework copies the first layer's block depths into later layers (`:4621–4629`), so the final tool change can leave space for a finish block. Remaining space is necessary but not sufficient to emit it (`:5057–5093`).

## State and predicate

- Retain the configured `extruder_printable_height` vector and one optional last-change layer index per physical extruder in the Type1 plan (`:2013,2073`). After planning the layers (`:4861–4862`), record each physical extruder's last planned change by visiting both the old and new logical filaments on each layer (`:4496–4507`). A layer without a change does not update the index. Use the physical-extruder mapping, not a logical filament index or the last tower layer index.
- For a selected filament, layer index, and Z, `is_valid_last_layer` (`:5528–5533`) rejects only when its physical extruder has a height slot, that extruder's last change is on this layer, and Z is strictly greater than the slot's stored height. An unmapped extruder or missing height slot does not fail the predicate. The check does not depend on framework being enabled and does not replace a stored zero with the global printable height.
- Consume the predicate in `type1/generate.rs::finish_remaining` after selecting the finish filament but before either solid or sparse finish emission. Initialize `finish_depth` before checking eligibility, as upstream does. Do not change the plan's depth calculation, finish-tool selection, or G-code writer to compensate.

For an initial change with `old_tool == -1`, Native skips the outgoing head (`WipeTower.cpp:4503`) but still checks the incoming head (`:4505–4506`). Ares starts `current` at `first_tool` and has no `-1` tool ID. Its `(current, current)` layer registration (`type1/planning.rs:51–64`) creates no `Change` (`type1/plan.rs:121–122`), so registration alone does not update a last-change index.

## Deferred behavior

- Per-layer nozzle grouping: Native `WipeTower::get_extruder_id(tool, layer_id)` delegates to `m_multi_nozzle_group_result` (`WipeTower.cpp:5563–5565`). It resolves the filament through the layer's nozzle map to the physical extruder (`MultiNozzleUtils.cpp:380–388,448–464`). Ares Type1 uses the same static, zero-based `Plan::nozzle_map` derived from `filament_map` on every layer (`type1/plan.rs:76–83`). Porting the per-layer lookup for both last-change recording and eligibility is deferred.
- Wall-filament candidate selection at `WipeTower.cpp:4983–4988` qualifies the first outgoing filament before preferring it or its category. This slice does not change wall selection.
- Ramming-depth planning at `WipeTower.cpp:3425` zeros nozzle-change depth for an ineligible outgoing filament. This slice does not change planned depth.
- Ramming emission at `WipeTower.cpp:3503–3507` qualifies the outgoing filament before ramming. This slice does not change tool-change emission.

The Type1 plan owns the shared last-change state and predicate so these consumers can use the same rule when ported. Type2, filament retraction inputs, the timing processor, and the comparator are outside scope.

## Public witnesses

Use the existing two-filament X1C Type1 project with framework off as a whole-output control. Modify only `prime_tower_enable_framework` at runtime for the failing counterpart: the extra final sparse block and its time must disappear without losing the rib wall. Check strict ordered Native/Core API/CLI output with generator-only normalization and require raw API/CLI equality.

Seek a stable dual-head Type1 project with both physical extruders used and nonzero printable-height slots. It must exercise the final change rather than only report two configured heads. Keep input and outputs outside the repository; report any pre-existing divergence and do not relax the whole-stream comparison for the X1C witness.
