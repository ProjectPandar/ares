# Final 2.5.0 remembered groups and empty inventory

## Authority and Rust boundary

Port the bounded static flush-mode slice of Orca commit `a2acec57c4784d6cdb3c3806b2c73286e6edd495` (loaded-filament grouping), as present at release `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, not a new Ares grouping policy.

Pinned sources under `src/libslic3r/`:

- `FilamentGroup.cpp:103-112`: shared flush-volume-to-score conversion.
- `FilamentGroup.cpp:245-270,273-414`: `calc_one_change_budget` and its use by `calc_group_by_enum`.
- `FilamentGroup.cpp:472-568`: `select_best_group_for_ams`, especially empty-inventory first-candidate return at489-495.
- `FilamentGroup.cpp:571-623`: `FilamentGroupUtils::update_memoryed_groups` (implemented in this file, despite its namespace).
- `FilamentGroup.cpp:1313-1329`: prepend the primary flush solution to remembered candidates.
- `GCode/ToolOrdering.cpp:1675-1748,1839-1856`: master-extruder preference and reselect between preferred/original maps; empty inventory keeps the preferred first candidate.

Rust destination: `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/{enumeration.rs,sequence.rs,mod.rs}`. Existing `Context` remains a temporary compatibility shell for the source's static dual-head, one-nozzle-per-head, Auto For Flush subset. It is not a replacement pipeline or a live printer-inventory model.

## Included

1. Compute a one-change budget from all physical-head flush matrices: each unordered pair of used logical filaments contributes the larger directional `f32` flush. Select the upper median, convert with the shared double constant `1.26 *180 *2 /1000`, then multiply by1.2. Fewer than two used filaments yields zero. Existing context guarantees the two matrices are present.
2. Preserve heap ordering, preference priority, first-best enumeration order and heap rebuilding. Retain same-preference candidates when absolute score difference is at most `max(10*6, one_change_budget*5, best_score*0.01)`, replacing beta's special zero-score10 / relative1% rule.
3. Preserve primary-before-remembered candidate order. With both printer inventories empty, choose the first candidate. Do not rank candidates by occupied-head count. This is the relevant branch of `select_best_group_for_ams`, not an invented inventory fallback.
4. Retain existing master preference: under the supported empty-inventory context, upstream's final selection between `[preferred, original]` necessarily selects `preferred`. Update the citation/comment; no new swapping heuristic.
5. Use the same constant expression as final upstream score conversion in both existing score functions.

The retained-budget computation is ported and exercised during enumeration, but empty-inventory E2E results do **not** independently prove which candidates the heap retained: the final source branch selects the primary. Do not claim a live-inventory budget witness.

## Deferred / unchanged

Nonempty or partially populated AMS/tray inventory, per-filament missing-slot penalties in that branch, live colour/type matching and `filament_id` subtype tie-breaking, Auto For Match, KMedoids and its budget plumbing, dynamic/switcher/preload grouping, Hybrid nozzle groups and larger enumeration contexts remain outside this slice. Do not add inventory APIs or enlarge `Context` merely to port adjacent hunks. Existing rejection boundaries remain.

Physical-extruder count handling, bridge/curl speed and tower/sync behavior are separate upstream slices.

## Test seam and failure modes

Use the existing public API + real CLI + runtime Native2.5 E2E cases:

- `filament_group_flush_mechanisms::x2d_pla_three_filaments_remembered_small_gap` checks16 configuration fields and complete API/CLI identity.
- `filament_group_flush_remembered_pair::x2d_pla_remembered_gap_changes_empty_inventory_assignment` checks two unchanged authored3MF inputs, fixture SHA/only-flush-matrix delta, Native filament use, both endpoint maps and API/CLI identity.

The second stable test ID describes the historical beta distinction. At final2.5 both endpoints are `1,2,1`, so the flush-entry3-versus4 pair no longer changes the empty-inventory assignment. Native and Rust must agree on both endpoint maps and the affected nozzle/retraction fields, with unchanged fixture hashes and runtime Native comparison. These cases use public slicing interfaces rather than private-function unit tests or stored output goldens.

Possible mismatches: beta endpoint assertions differ from final Native results; occupied-head ranking picks `1,1,1`; median uses the lower middle/one direction/one physical matrix; budget applies across different preference levels; score conversion uses beta multiplication ordering; empty inventory loses to a retained candidate; master preference is absent. Existing sibling cases cover traversal, explicit physical restrictions, custom order and carried state; retain them as directed controls. Partial/live inventory and isolated heap-budget coverage are explicitly unclaimed.
