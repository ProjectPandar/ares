# FilamentGroup FlushMode enumeration

## Decision and upstream boundary

Rewrite the non-TPU, static dual-extruder BBL FlushMode enumeration path. Replace the early all-master recommendation with source-owned grouping over actual layer usage, followed by used-entry publication and column rematerialization. This is not a new Ares optimizer or slicing pipeline.

The source is OrcaSlicer 2.5.0-beta, revision `345efec64c7b95a05ce5cf96df0dde66a158835c`, under `src/libslic3r/` unless stated otherwise.

| Source | Included behavior |
| --- | --- |
| `GCode/ToolOrdering.cpp:1477–1534` | `prepare_flush_matrices`, `build_nozzle_groups` |
| `GCode/ToolOrdering.cpp:1537–1672` | `build_filament_group_context`: layer usage, constraints, capacities, master, filament/nozzle records and change-time parameters |
| `GCode/ToolOrdering.cpp:1681–1747` | `can_swap_extruder_groups`, `apply_master_extruder_preference` |
| `GCode/ToolOrdering.cpp:1752–1850`, especially `:1838–1846` | Non-TPU BBL automatic grouping dispatch |
| `FilamentGroup.hpp:27–119` | `FGStrategy`, `FGMode`, `MemoryedGroup`, `FilamentGroupContext` |
| `FilamentGroup.cpp:36–374` | Enumeration estimate, nozzle deduplication, evaluation, traversal and preference ranking |
| `FilamentGroup.cpp:427–580,901–922,1036–1051,1253–1268` | Remembered candidates, AMS postselection, used IDs and FlushMode composition |
| `FilamentGroup.cpp::rebuild_nozzle_unprintables` | Convert extruder/volume restrictions to nozzle-assignment restrictions |
| `FilamentGroupUtils.cpp:88–226,346–358` | AMS capacities, usable inventory, combined restrictions and used-entry merge |
| `GCode/ToolOrderUtils.cpp:116–216,748–795,797–1141,1324–1494` | Cost-only matching, sequence ordering, flush accumulation and static sequence assembly |
| `MultiNozzleUtils.cpp::LayeredNozzleGroupResult::{create,get_extruder_map,get_volume_map,get_nozzle_map}`, `:467–661` | Static result conversion and `simulate_filament_change_time` |
| `GCode/ToolOrdering.cpp:3053–3084` → `Print.cpp:4656–4678,4722–4753` | Map publication, used-entry merge and configuration-column rebuilding |
| `PrintBase.hpp:518`, `GCode.cpp:8244–8247` | Export the resulting full configuration |

## Identity and input contract

Logical filament IDs, physical extruder IDs, global nozzle-group IDs and materialized filament configuration columns are distinct. Engine filament/extruder IDs are zero-based; the published extruder map is one-based. A nozzle-map entry is a global nozzle-group ID, not a head-local slot. Volume-map publication retains the source vector's full logical-filament shape.

The seed map is the saved/applied configuration. A recommendation is computed from actual usage. Publication merges only used recommendations into the seed; unused configured entries remain unchanged. Remembered groups are alternative assignments admitted by the source preference/gap rules, not cached final maps.

Borrow the applied settings and actual ordered layers from `collect_layers`. Preserve supplied custom sequences and printable constraints. Do not infer usage from colour count or add a second layer traversal. Matrices contain upstream `float` entries segmented by physical extruder, including asymmetric and self-flush values. Select the Normal/Fast multiplier once, without exporter rounding or a second scaling pass.

### CLI machine context

`src/OrcaSlicer.cpp:6937–6969` prepares per-head AMS counts `1#0|4#1` and synthetic filament records before `Print::apply`. The records omit `tray_name`; `FilamentGroupUtils.cpp:106–155::build_full_machine_filaments` therefore rejects them as usable colour inventory. Capacity is four per head, but usable colour inventory is empty. Do not fabricate trays or colour matches.

Move the existing `prepare_multi_extruder_cli_defaults` policy to `options/filament_map_recommend.rs`. Invoke it from `project/effective_config/phases.rs` on applied settings, after the applied-map snapshot and before consumers. `options/config_export/transform.rs` must not maintain a second export-only preparation. Export serializes the same counts consumed by grouping. Raw project counts alone do not describe the CLI context. Live-printer discovery and UI state are deferred.

## Included and deferred behavior

| Included | Deferred |
| --- | --- |
| Two BBL extruders, one physical nozzle each, static mapping, non-TPU used filaments, Auto For Flush | Other head/nozzle counts, dynamic reassignment, mixed nozzle inventory and hardware search |
| Existing late ToolOrdering gate: by-layer, or one object in by-object mode | Multi-object by-object production in `Print.cpp` |
| `estimate_dedup_enum_count < 10000` and its enumeration branch | K-medoids (`FilamentGroup.cpp:376–404`), stochastic restarts, wall-clock budgets and estimates at least `10000` |
| FlushMode preference ranking, remembered candidates, AMS postselection and master preference | Top-level MatchMode (`FilamentGroup.cpp:1053–1251`) and its exception dispatch |
| `MatchModeGroupSolver` as the helper called by FlushMode AMS postselection | Exposing that helper as top-level MatchMode support |
| Supplied static nozzle/volume and physical/geometric constraints | New collision analysis or a replacement constraint producer |
| No-switcher/no-preload context; configured `group_algo_with_time` values 0 and 1 | Enabled switcher/preload context |
| Actual load/unload change-time simulation | General G-code processor time estimation or time-header changes |
| Normal/Fast multiplier selection for grouping cost | Prime-tower geometry, purge commands and exporter-format changes |
| Used-entry map/volume/nozzle publication, selected columns, nullable overrides and config-block regeneration | General dynamic variant migration, process/printer reselection and getter-selector changes |

Preserve existing Manual, single-head, non-BBL and TPU branches as controls. Do not claim they are newly rewritten by this slice. The upstream estimator, not an arbitrary filament-count limit, defines the enumeration boundary. Outside the included automatic path, use the public slicing-error channel rather than all-master success, an uncomputed seed, another mode or unbounded enumeration.

## Algorithm contract

1. Collect sorted, deduplicated used filament IDs from actual layers. Use the source wrapper's BestCost strategy, gap `0.01`, zero-based master and Hybrid volume requests. Do not invent omitted usage-type or print-time producers.
2. Build static nozzle records and constraints with source handling of conflicting unprintable sets, volume exclusions and capacity rewards. Do not convert every preference into hard rejection.
3. Compute the deduplicated estimate and enter enumeration only below `10000`. Do not turn the deferred K-medoids dispatch into all-master success.
4. Traverse assignments in source order: the lowest used-filament position changes fastest. Preserve nozzle/group fingerprints and fixed-width hash arithmetic independently of Rust `usize` width. Random container iteration must not change the first winner.
5. Preserve preference arithmetic: printable placement reward `10000`, capacity reward `5000`, and support preference `100` only where supplied by the source context. BestFit is not an independently exposed feature of this wrapper.
6. Preserve `full_evaluate`: cost-only sequence ordering across actual layers, custom sequence, initial/nozzle state and prior filament; then static extruder sequence assembly and load/unload simulation. Forecast is disabled in `ToolOrderUtils.cpp:1025–1141`; do not enable its dormant search.
7. Preserve float/int conversion points and evaluation order. Flush score is `flush * 1.26 * 180 * 2 / 1000`, plus actual change time. Read `group_algo_with_time` normally. `ToolOrdering.cpp:1532,1604` supplies no `filament_print_time`, so `FilamentGroup.cpp:191–197,621–631` contributes zero print time for either flag. `evaluate_score` still uses `with_time=true`. Do not add a print-time table or replace simulation with generic machine-switch-time multiplication. Apply the source master occupancy penalty `10` at the same stage.
8. Select highest preference first, then strictly lower score. Preserve remembered candidates' relative gap `0.01`, or absolute gap `10` near zero, and source heap/tie behavior (`FilamentGroup.cpp:518–570`). Do not introduce a lexicographic tie-break.
9. Prepend the primary assignment to remembered candidates and run AMS postselection. Preserve missing-inventory penalties, colour threshold `20`, type/support unlink rules, integer min-cost-flow conversion and first-minimum selection. Reuse existing colour-distance math through `calc_color_distance`, matching `FlushVolPredictor.hpp:32`; do not duplicate or change the formula/parser.
10. Apply master preference outside the engine, after AMS selection, only in FlushMode. Swap only when non-master occupancy is strictly greater and printability/capacity permit it. Equal occupancy is not a swap trigger.
11. Convert assignment results explicitly to extruder, volume and global nozzle maps. Merge only used entries, rematerialize from the retained checkpoint, resolve nullable overrides, then regenerate the full config block from that configuration. A getter change cannot replace publication.

These sequence helpers score candidate assignments; they do not replace emitter path ordering or add motion heuristics. The forced `group_with_time=false` assignments at `ToolOrdering.cpp:3026,3171` belong to deferred dynamic planning, not the included static flag behavior.

## Rust destination and replacement

Use `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/recommend.rs` and a private `tool_ordering/filament_group` module. Separate context, enumeration, scoring/sequence, AMS/matching and static-result concerns before a file exceeds 400 LOC. Expose one internal computation over borrowed context, returning a source-equivalent static assignment. No optimizer framework, new crate or public service abstraction is needed.

`recommend.rs::build_recommended` owns actual usage, dispatch, post-engine master preference, used-entry merge, refresh and config-block regeneration. Preserve `filament_variant_checkpoint` and the original applied-map snapshot. The refresh boundary must carry volume/nozzle maps as well as the extruder map.

Delete the dual-BBL all-master result branch from `options/filament_map_recommend.rs::apply_recommended_filament_map`. Early preparation preserves the applied seed until actual layers exist; it does not publish a computed recommendation. Do not wrap the old branch or retain it as a fallback. Upstream single-head all-master behavior remains separate.

Filament/printer column gathering remains owned by `options/project_variants/{filament,printer}.rs`. These two files are read-only for this slice; consume their refresh contract rather than adding motion-side materialization. Preserve physical-index reads that are source-owned physical data. No filesystem, terminal, UI, OpenGL, native clock or platform-specific core dependency is included. Numeric simulation must work on browser WASM, Windows, macOS and Linux.

## Public-output test contract

Keep the complete retraction suite and input hashes. Producer witnesses use the inputs named:

- `x2d_pla_concentric_ironing_auto_map`
- `x2d_pla_auto_map_toolchange_boundary`
- `x2d_pla_variant_order_stream`
- `x2d_pla_variant_order_extrusion_speed`
- `x2d_pla_toolchange_without_virtual_move_comments`
- `x2d_pla_toolchange_with_virtual_move_comments`

Assert published extruder/volume/global-nozzle maps, self-indices, materialized nullable retraction lengths and the literal first retract. Preserve complete Native/API/CLI outputs, Native double replay with the existing generator-line-only comparator and public API/CLI byte equality. Independent generator-line-only whole-output comparisons remain strict diagnostics; map or prefix equality does not imply whole-program parity.

Before implementing each behavior, establish public Native discriminators for asymmetric/self costs, physical matrix segments, Normal/Fast multipliers, actual change-time scoring, layer/custom sequences, initial/prior state, estimate/dedup/ties, preference/remembered groups, AMS selection, master occupancy/constraints and unused seed preservation. A pair producing the same map does not demonstrate mapping discrimination. Do not manufacture an answer through Manual mode or fixture map edits.

Preserve the bounded retraction witnesses, photo-head guards, toolchange/wipe/Z-hop/first-extrusion boundaries and existing strict comparisons. `printer_variant_order_machine_speed_e_matches_native` remains supported with flag 1 and its public E-speed/API/CLI contract; `cli_custom_gcode_failure_preserves_nonempty_upward_machine_header` retains its intended failure/header contract. Invalid matrix validation precedes grouping and retains `Flush volumes matrix do not match to the correct size!`.

Test deferred boundaries as declared limitations, not Native parity passes. Use `cargo nextest run` with fixed selected inventory and repeatable output artifacts. Core qualification includes rustfmt, workspace/all-target Clippy without warnings and browser-WASM compilation; broader whole-program parity is a separate claim.
