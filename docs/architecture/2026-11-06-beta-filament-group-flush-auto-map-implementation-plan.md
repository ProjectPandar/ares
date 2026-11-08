# FlushMode engine implementation plan

## Source boundary

This plan implements the [FlushMode enumeration ARD](2026-11-06-beta-filament-group-flush-auto-map-ard.md), not an independent optimizer or Ares slicing pipeline. Upstream is OrcaSlicer 2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`; source paths below are relative to `src/libslic3r/`.

The included path remains non-TPU, static dual-BBL, one physical nozzle per head, Auto For Flush, by-layer or one-object by-object, no switcher/preload, and the source deduplicated estimate below `10000`. Both `group_algo_with_time=0` and `1` are included. Preserve existing Manual, single-head, non-BBL and TPU control branches without claiming new parity for them. Dynamic/multi-nozzle planning, mixed filament/nozzle inventory, multi-object sequential production, top-level MatchMode, K-medoids and live inventory discovery remain deferred. Unsupported automatic contexts must not fall through to an all-master map or an uncomputed seed.

## Destination and source ownership

The private directory is `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/`. Split files before they exceed 400 LOC; the following are source concerns, not new public abstractions.

| Rust destination | Owning upstream boundary | Responsibility |
| --- | --- | --- |
| `mod.rs`, `context.rs` | `FilamentGroup.hpp:27–119`; `ToolOrdering.cpp:1477–1672` | Borrowed wrapper inputs, static nozzle records, BestCost/FlushMode context and numeric preparation |
| `constraints.rs`, `constraints/geometry.rs` | `Print.cpp:3593–3604,4908–4976`; `PrintObject.cpp:349–515`; `FilamentGroupUtils.cpp:156–226`; `FilamentGroup::rebuild_nozzle_unprintables` | Source physical, volume and geometric restriction sets, conflict handling and nozzle translation |
| `enumeration.rs` | `FilamentGroup.cpp:36–374` | Deduplicated estimate, nozzle fingerprints, source assignment traversal and preference evaluation |
| `sequence.rs`, `sequence/ordering.rs`, `sequence/change_time.rs` | `ToolOrderUtils.cpp:116–216,748–1141,1324–1494`; `MultiNozzleUtils.cpp:467–661` | Cost-only layer ordering, static sequence construction and actual load/unload simulation |
| `score.rs`, `remembered.rs` | `FilamentGroup.cpp:191–197,427–631` | Flush/time score, preferred candidate selection and remembered heap behavior |
| `ams.rs`, `matching.rs` | `FilamentGroup.cpp:427–518,901–922,1036–1051,1253–1268`; `ToolOrderUtils.cpp:116–216` | FlushMode AMS postselection, including its MatchMode helper and integer matching |
| `result.rs` | `MultiNozzleUtils.cpp:25–41,144–163,319–369` | Static result and explicit extruder, volume and global nozzle-group map conversion |
| Existing `recommend.rs` | `ToolOrdering.cpp:1681–1747,1752–1850,3053–3084`; `Print.cpp:4656–4753` | Actual usage, dispatch, post-engine master preference, publication, refresh and config block |

The computation accepts the existing actual layer filament lists and borrowed resolved/checkpoint data; it returns a full logical-filament assignment into the source nozzle list. It does not expose a new service, trait framework, crate, public API, clock or platform dependency. No emitter path-ordering or motion change belongs to the sequence scorer.

## Publication and refresh assessment

The existing `options/project_variants.rs::refresh_filament_map(checkpoint, map, current)` is sufficient for the included one-nozzle-per-head boundary:

1. It clones `current`, changes its extruder map, selects filament columns from the saved checkpoint, remaps self-indices and calls `resolve_project_config_views`.
2. `project_variants/filament.rs::materialize` writes filament option columns, not project volume/nozzle maps. `project_config_views.rs` retains the full configuration and rebuilds runtime nullable overlays separately.
3. `project_variants/index.rs::active_variants` selects the physical head's extruder/volume variant. With exactly one volume type per physical head, the source `extruder_volume_type_count > extruder_count` override branch is not needed. This is not a claim about mixed or multi-nozzle materialization.

Use that API unchanged. Prepublish all three source maps into the current full configuration and retained full-config checkpoint, then call refresh with the new extruder map. Do not copy/reconstruct `resolved.applied_filament_map`: ironing must continue to borrow the application-time snapshot.

The publisher must follow `ToolOrdering.cpp:3066–3081` exactly:

- Obtain the full result through `LayeredNozzleGroupResult` conversion; do not equate a nozzle index, global group ID, physical head or configuration-column index.
- For extruder and volume merge bases, retain the supplied vector only when its length equals the derived map length; otherwise use the source defaults of one-based head `1` and Standard volume `0`.
- Apply `FilamentGroupUtils.cpp:346–358::update_used_filament_values` to extruder and volume maps only. Unused configured entries stay unchanged.
- Publish the full derived global nozzle map directly, not a used-only merge or `extruder_id - 1` substitute. `LayeredNozzleGroupResult::create` retains the full assignment, including unused entries.
- Compare all three maps, not only the extruder map. A volume/nozzle-only change must not be lost. Rebuild selected filament columns, self-indices and nullable runtime overrides even when an extruder assignment happens to be unchanged, as the source publication function does.
- Regenerate `traversal.config_block` from the refreshed views through the existing exporter. Retain the selected prime-volume context and its independently owned footer arithmetic.

No signature or getter/materializer change is proposed in `options/project_variants.rs`, `options/project_variants/{filament,printer}.rs` or `project_config_views`. Any newly discovered insufficiency requires a separate exact-hunk proposal rather than a motion-side materializer.

## Constraint producer dependency

The current late recommendation function does not receive the source's physical/geometric/volume restriction sets. Supplying empty sets merely because the ten witnesses are centrally located is not a port.

The proposed explicit dependency is a literal restriction adapter inside the private `constraints` modules:

- `Print::get_physical_unprintable_filaments` (`Print.cpp:4908–4941`) preserves its used-filament scan and first forbidden physical-head behavior; do not independently reinterpret printable masks.
- `Print::get_filament_unprintable_flow` (`:4943–4976`) reads variant declarations and self IDs from the retained full checkpoint, not already shortened filament columns.
- `PrintObject::detect_extruder_geometric_unprintables` (`PrintObject.cpp:349–515`) and its print-wide union (`Print.cpp:3593–3604`) own exclusion-region, height and printable-area restrictions. Preserve actual print Z, per-region role IDs, fill/perimeter presence, raw-slice/fill polygon operations, instance transforms and plate-origin handling; no bounding-box-only substitute or new collision heuristic.
- Reuse existing `options/bed_exclusion::{get_bed_excluded_regions_by_extruder,model_part_intersects_region}` and core polygon operations for their already ported source behavior. The bed-exclusion parser and mesh-intersection implementation remain read-only. If a typed accessor or retained geometry datum is unavailable, stop for an exact visibility/data-threading proposal; do not serialize the whole configuration into a second adapter or substitute an empty constraint set.

This dependency makes the previously supplied-constraint boundary explicit. It is a source rewrite of those named producers, not authorization for a general collision subsystem or new hardware analysis. Any necessary owner/API expansion is a separate source-owned boundary, not an implicit extension of the solver.

## Scoring and determinism invariants

- `prepare_flush_matrices` partitions raw entries by physical extruder count. Cast raw doubles to `float` first; select Fast multipliers only for Fast, resize/truncate the multiplier vector to physical count with `1`, and reproduce C++ float-entry compound multiplication by double, rounding back to float after that multiplication. Do not round engine entries to integers. Footer preparation remains raw-double multiplication plus round, shaped by the selected multiplier vector length.
- Preserve actual layer IDs/order, custom sequence precedence, initial/prior filament and physical-nozzle state through `ToolOrderUtils`. Do not score a fixed sorted sequence or replace simulation with a generic switch-count formula. Forecast remains disabled.
- The pinned wrapper supplies no filament print-time table and leaves usage type at its source default. Print-time contribution is zero for either flag; actual load/unload/extruder-change simulation still runs with evaluation time enabled. Do not invent a usage-type or print-time producer.
- Preserve printable/capacity/support preference arithmetic and its source applicability. Higher preference wins before strictly lower cost. Preserve the source flush conversion, master penalty and conversion points.
- Preserve lowest-position-fastest enumeration, source sorting and fixed-width wrapping hashes independently of WASM `usize`. No arbitrary filament-count cutoff, random iteration or lexicographic tie-break is permitted.
- Preserve remembered relative gap `0.01`, absolute gap `10` near zero, candidate insertion order and source heap ties. Rust floating total ordering or a stock heap must not silently add signed-zero/identity ordering or change equal-key extraction behavior.
- Run source AMS postselection over primary plus remembered candidates. Synthetic CLI records have no `tray_name`: usable inventory is empty despite capacity four per head. Do not fabricate trays. Preserve missing-inventory penalties, first minimum, type/support rules, threshold `20` and integer matching semantics.
- Reuse the existing colour implementation through `calc_color_distance`; introduce its façade alias only with its first actual engine consumer. No duplicated parser or formula.
- Master preference remains outside the solver after AMS selection. Swap only on strictly greater non-master occupancy and valid source capacity/printability constraints.

## Replace the temporary recommendation shells

`project/effective_config/phases.rs` keeps normalization, materialization, applied-map capture and the existing CLI AMS preparation. Remove early dual-BBL automatic map computation: geometry must see the applied configuration until real ToolOrdering usage exists. Preserve the source-owned behavior of the existing independent single-head/non-BBL/TPU controls; do not broadly delete their routes.

`options/filament_map_recommend.rs` must no longer return all-master success for the included dual-BBL automatic path. It may retain independently sourced preparation/control helpers, not an optimizer fallback. The actual non-TPU FlushMode solver call and publication replace that shell in `recommend.rs::build_recommended`, after `collect_layers` and before `finish_layers`. Keep the existing by-layer/one-object gate and public slicing-error channel.

Validate the existing malformed-matrix contract before unsupported-context/grouping dispatch. Preserve `Flush volumes matrix do not match to the correct size!`, the CustomGCode/header failure contract and the flag-1 E-speed/API–CLI control.

## Test-first qualification and remaining evidence gaps

| Mechanism | Existing public evidence | What remains before claiming that mechanism qualified |
| --- | --- | --- |
| Missing producer/publication | Six distinct frozen producer identities: Native `2,1`, API `1,1`; volume/nozzle/self/retraction assertions | Make all fields and literal retracts GREEN through the producer and refresh, not fixed assignments |
| Physical matrix context and Fast selection | Default/Fast pair changes Native map `1,2,1` → `1,1,2`; Fast footer field independently GREEN | Engine must use the source float matrix rather than rounded footer data |
| Actual change time with flag 0 | Scalar load-time pair changes Native map `1,2,1` → `2,1,1`, both with flag 0 | Source sequence/change simulation; no `if !group_with_time { return 0 }` shortcut |
| Flag 1 compatibility | Original producer cases and E-speed/API–CLI control | Preserve support; the empty print-time table is not a flag-0-only restriction |
| Refresh and full map shape | Ten map REDs assert maps and materialized fields; 23 retraction identities remain registered | Post-publication E2E results, unused-seed/volume-only change witnesses where existing inputs do not discriminate |
| Traversal/dedup/ties, remembered selection, custom/initial/prior sequences, master constraints | Source contracts are identified; the ten cases alone do not independently isolate all of them | Audit frozen inputs for witnesses; author additional Native minimal pairs and public RED assertions before the corresponding behavior |
| Restriction producers | Named physical/flow/geometric source boundaries | Native reachable/unreachable, height/area and printable/volume conflict witnesses; no blanket empty sets |
| AMS matching | CLI capacity and empty usable inventory are source-established | Reachable missing-inventory/first-minimum behavior needs a public witness. Nonempty inventory/colour matching is not runtime-qualified by these inputs; disclose dormant branches and do not invent a live inventory producer |

Unchanged-map pairs, Native-only observations and early validation failures are not scoring RED. Keep new exploratory inputs separate from frozen formal inputs. Same-byte producer identities remain separate tests. If a required public discriminator cannot be reached in the included wrapper, report that limitation rather than claiming coverage or silently broadening the wrapper.

## Implementation and verification sequence

1. Establish the refresh contract, constraint dependency and source-to-destination boundaries before connecting the solver. Keep shared API expansion separate.
2. Establish missing mechanism witnesses through Native replay and public API/CLI target assertions. Record their failure inventory before implementation. Existing tests and all frozen input bytes remain unchanged.
3. Port a connected private engine and its real wrapper/publication consumer. Avoid unused scaffold commits, lint allowances and premature colour exports. Split source concerns into files before 400 LOC; preserve the source class/function boundaries above.
4. Run the fixed ten-case inventory in both frozen modes. Require all map/volume/nozzle/self/nullable-retraction and matrix assertions GREEN, qualified Native double replay and complete API/CLI byte equality. Keep independent generator-line-only whole-output comparisons as strict diagnostics, not implied parity.
5. Rerun all 23 retraction identities, both public controls and malformed-matrix controls. Preserve exact Native premises, feedrates, G1 formatting, wipe/toolchange/Z-hop/first-extrusion/custom-G-code/acceleration/photo-head boundaries. Retain the two existing nominal-matrix whole-output comparisons without suppression or normalization.
6. Verify actual-tip rustfmt, workspace/all-target Clippy without warnings and core/browser-WASM compilation. Each commit must independently pass Clippy. Do not transfer runtime credit across rewritten commits.
7. Request separately coordinated broader qualification only after focused gates pass. No automatic FULL, sweep change, general unequal-head claim, time-header fix or whole-program parity claim belongs to this implementation step.
