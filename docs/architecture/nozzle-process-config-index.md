# Runtime process nozzle configuration columns

## Authority and ownership

Port OrcaSlicer v2.5.0 revision
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, under `src/libslic3r/`:

| Source | Responsibility |
| --- | --- |
| `Print.cpp:5113–5117,5155–5184` | Resolve a process column from the published nozzle group and default-region process identities |
| `PrintConfig.cpp:798–814,833–837` | Ordered variant/identity lookup and defined column-zero result on no match |
| `GCode.cpp:5083–5087,2185` | Current-layer process lookup and `NOZZLE_CONFIG` consumption |
| `MultiNozzleUtils.cpp:142–216,380–447` | Published static assignments and layer-specific nozzle information |
| `GCode.cpp:538–602,677–736,10838–10864` | Existing retract/Wipe consumer that exposes the incorrect process column |

Logical filament, physical extruder, nozzle group, filament configuration column,
and process configuration column are distinct identities. A process column is
not necessarily `filament_map[filament] - 1`. Physical hardware reads must keep
that mapping. `FILAMENT_CONFIG` and nullable retraction overrides stay unchanged.

This slice replaces the physical-only process selector in supported G-code
emission. It does not introduce a new grouping engine or configuration pipeline.

## Lookup contract

Use the finalized default region's ordered `print_extruder_variant` and
`print_extruder_id` arrays. These are the source's process-wide identities, not
per-object overrides and not the originally authored, unexpanded JSON vectors.
For the current logical filament and current published assignment:

1. If no grouping result was published, return the mapped physical extruder
   index. This is the no-result branch of `Print.cpp:5158–5162`.
2. If a result exists but has no nozzle for this filament/layer, return zero,
   as at `Print.cpp:5163–5170`. Do not treat this as the no-result branch.
3. From the assigned nozzle obtain its zero-based physical extruder and actual
   volume type. Read the physical extruder's type. Form the source variant name
   from that type and volume, and search using physical extruder ID plus one.
4. Select the first exact variant-and-ID match. If absent, select the first
   column with the same ID. If both searches fail, return zero. Preserve source
   handling of an empty variant list, an empty ID list, and a wildcard request.
5. Preserve `ConfigOption::get_at` behavior when reading the selected vector.
   Do not add column clamping, new defaults, sorting, deduplication, or a second
   normalization pass.

`find_variant_index` returns -1 when there is no match.
`get_config_index_base` deliberately returns `max(index, 0)`. This defined
behavior is not a legacy fallback or upstream undefined behavior.

The source entry requires aligned expanded variant/ID arrays. Existing import
and materialization own that invariant. The resolver must not silently repair
malformed raw arrays or change their public admission errors.

## Rust destinations

- Add a private `options/project_variants/nozzle.rs` for the source-equivalent
  ordered process-column lookup. Reuse existing variant naming/search primitives
  where their contracts match. It receives borrowed finalized identity arrays,
  the physical ID/type, and the assigned volume. It owns no config copies,
  scheduling, filesystem access, public API, or exporter state.
- `project_slice/gcode_emit/tool_ordering.rs` and its `nozzle_group` module adapt
  the existing published assignment into the source nozzle information. Reuse
  `NozzleGroup { extruder, group_id, volume, ... }` and finalized filament maps.
  Do not regenerate groups from colour count, default all-master maps, or the
  pre-recommendation checkpoint. Preserve the distinction between no published
  result and a published result without this filament.
- `motion/options/build/context.rs` prepares separate physical and process
  indices. `fields/assemble.rs` assigns the process index to
  `MotionOptions.nozzle_config_index`. `build/region.rs` uses it for source
  process vectors. Physical limits, offsets and other hardware consumers retain
  the physical index.
- Feed the selected column into the existing motion option assembly and layer,
  object-entry, skirt/brim and overhang consumers listed below. Avoid a parallel
  option builder. The physical-only process selector is replaced at these
  consumers rather than retained behind a feature flag.
- Keep `options::mapped_extruder_id` as the physical mapping function. Do not
  change its meaning globally. `project_variants/{filament,printer}.rs` continue
  to own their current materialization domains.

Resolve after the existing recommendation, used-entry publication and config
refresh. For a static assignment, borrow the plan and compute once per active
filament option installation. Do not allocate a cache or copy configuration on
individual G-code moves. Reinstall process options at the same existing
filament/region transitions that consume them.

## Consumer inventory

The inventory covers every `NOZZLE_CONFIG` use in `GCode.cpp`, local
`nozzle = get_nozzle_config_index(...)` vector reads, and direct resolver calls.
It includes forty distinct macro/local-nozzle option keys. Call sites sharing a
key must use the same process selector.

| Consumer and source | Process vectors / Rust destination |
| --- | --- |
| Role and first-layer speed, `GCode.cpp:9531–9564,9600–9627` | `inner_wall_speed`, `outer_wall_speed`, `bridge_speed`, `internal_bridge_speed`, `sparse_infill_speed`, `internal_solid_infill_speed`, `top_surface_speed`, `gap_infill_speed`, `support_speed`, `support_interface_speed`, `initial_layer_speed`, `initial_layer_infill_speed`. Existing motion kinematics and region speed assembly. |
| Small wall loops, `:8588–8592` | `small_perimeter_threshold`, `small_perimeter_speed`, and the selected outer-wall speed. Existing loop/path kinematics. |
| Small support loops, `:9020–9031` | `small_support_perimeter_threshold`, `small_support_perimeter_speed`. Deferred: Ares has no corresponding G-code consumer. Existing support/skirt base speed remains in scope. |
| Overhang processing, `:6997,9693–9765` | `enable_overhang_speed`, `slowdown_for_curled_perimeters`, all four quarter-overhang speed vectors (`overhang_{1_4,2_4,3_4,4_4}_speed`), bridge and inner/outer reference speeds. Existing overhang estimator inputs. |
| Extrusion acceleration, `:9414–9443` | `default_acceleration`, `initial_layer_acceleration`, `outer_wall_acceleration`, `inner_wall_acceleration`, `top_surface_acceleration`, `bridge_acceleration`, `sparse_infill_acceleration`, `internal_solid_infill_acceleration`. Existing motion region/context acceleration assembly. |
| Extrusion jerk, `:9449–9462` | `default_jerk`, `initial_layer_jerk`, `outer_wall_jerk`, `inner_wall_jerk`, `top_surface_jerk`, `infill_jerk`. Existing region jerk assembly. |
| Top infill extrusion flow, `:9481` | `top_solid_infill_flow_ratio`. Deferred: pre-generation projection precedes assignment publication. Do not apply this index to filament flow ratio. |
| Layer/object entry and reset, `:4596–4601,6860–6869,6898–6903` | Outer-wall/default/initial acceleration and jerk plus `default_junction_deviation`. Existing `layers/{boundary,object_entry}.rs` and motion initialization. |
| Travel acceleration/jerk, `:10523–10556` | `initial_layer_travel_acceleration`, `travel_acceleration`, `travel_jerk`, bridge/outer/default acceleration and outer/default jerk. Initial-layer travel acceleration is a direct resolver read in addition to the forty-key set. `initial_layer_travel_jerk` is deferred: no Ares consumer exists. Existing motion travel kinematics remain in place. |
| Skirt/brim requested speed, `:5966–5969,6055,6085,6284` | Selected `support_speed`. Existing `motion/skirt_brim.rs` must not re-index with the physical slot. |
| Percentage Wipe speed, `:574,685` | `travel_speed` selected through the process resolver is the base of `wipe_speed`. Keep this base separate from normal writer travel speed. Existing motion retraction option assembly. Role-based Wipe still consumes saved extrusion speed. |

### Travel and other physical-index reads are not interchangeable

`GCodeWriter.cpp:810,891,994–999,1179–1183,1211–1215,1674` uses
`m_cached_extruder_idx = get_extruder_index(config, filament_id)`.
`PrintConfig.cpp:140–146` defines this as the physical filament map. Therefore
ordinary writer `travel_speed`, `travel_speed_z`, and
`initial_layer_travel_speed` reads remain physical. The process-indexed travel
acceleration/jerk and percentage-Wipe base above do change selector.

`MotionOptions` must not use a single local `travel_speed` as both the physical
writer speed and process-indexed percentage-Wipe base. Separate the two reads
without changing their existing speed formulas or pointwise first-layer policy.

Preserve the physical domains used by cooling initialization, placeholder
hardware values, machine speed/acceleration limits, nozzle diameter, offsets,
temperatures and tool selection. Tower generation's own indexed input arrays
and acceleration selection are not `GCode::NOZZLE_CONFIG` consumers and stay
unchanged. No Type1/Type2 geometry or constructor-input changes are included.

## Static publication and deferred dynamic behavior

Included:

- Consume existing supported static manual and automatic assignments after
  publication, including single-head and static multi-head configurations.
- Use the assigned group's actual volume and physical extruder type. Distinct
  groups on the same head must not be collapsed to a head-local guessed volume.
- Preserve the source no-result, missing-filament, exact-match, same-ID-first
  and no-match-zero cases.
- Read default-region process identities without changing their producers.

Deferred:

- New layer-varying assignment generation, dynamic nozzle inventory/search,
  hybrid hardware switching, assignment scheduling and group invalidation.
- New support for modes already outside the grouping producer's admitted
  scope, including deferred multi-object sequential/dynamic paths.
- A general `GCode` per-layer generation-aware cache, runtime printer
  reselection, and changes to filament configuration column resolution.

A static plan may represent the same assignment at every emitted layer. It is
not evidence of dynamic parity. Do not label an unimplemented dynamic plan as
"no group" and apply a physical fallback to claim success. Preserve existing
admission/unsupported-scope behavior. A layer-changing assignment witness or
required missing producer is a stop-and-report condition for this slice.

For singleton process columns the selected value is unchanged. For an exact
match whose process column equals the existing physical column, behavior is
unchanged. Same-ID variant fallback and physically reordered but valid process
columns follow the source, even if that exposes another previous physical-only
selection error. Matching identity does not justify a blanket promise that all
multi-head output is unchanged.

## Existing-consumer boundary

The inventory has 40 Native keys, not 40 existing Ares G-code consumers. Port
all existing Ares process selectors in this slice. Ares currently defines
`small_support_perimeter_speed` and `small_support_perimeter_threshold` only
as typed/wire configuration: it has no corresponding G-code consumption.
Implementing that missing support algorithm is deferred to an independent
slice; this resolver port must not add it.

`top_solid_infill_flow_ratio` is also deferred to an independent slice.
`group_fills/params/projection.rs` pre-bakes its physical-column value before
`gcode_emit/layers/schedule.rs` publishes the static grouping result through
`build_recommended`. There is no earlier assignment producer or group-fill
replay at that boundary. Preserve the current projection and physical-column
behavior; do not move top-flow application into motion, advance assignment
publication, or invent a producer. This slice covers existing motion/G-code
consumers with an available producer: 37 of the 40 inventoried Native keys.
The review report must distinguish these from all three deferred keys.

The direct Native `initial_layer_travel_jerk` read lies outside the forty-key
macro/local-nozzle inventory and is also deferred. Ares defines the option but
`motion/state.rs::begin_path_travel` reuses `travel_jerk` on the first layer;
it has no consumer for the separate initial-layer option. Preserve that branch
and treat the missing first-layer travel-jerk algorithm as an independent slice.
Do not add its absent consumption while porting existing selectors.

## Public tests before implementation

Keep the original `print_process_identity_short_ids_use_generated_group_identity`
input, assertions and whole-file comparator unchanged. It remains the broad
public API regression. It must go from its real failure to complete Native
parity in both official and seeded modes. If an independent later mismatch is
exposed, report that first difference and stop rather than broadening this port.

Add a small public Native/API/CLI witness from the public cube10 model and stock
H2D project. Construct its archive in the test from public bytes, with explicit
two-column process identities, distinct finite outer-wall speeds, role-based
Wipe, filament retract length .4 and wipe distance 1. Preserve stock H2D's
supported variant lists. Its CLI normalization requires four authored process
requests (High Flow/Standard for each head) and short `[7]` IDs to preserve
two distinct finalized Standard columns. Native must independently finalize
those to `[7,7]` with speeds `[500,200]`. Narrowing hardware variant lists to
force two raw columns exposes a separate import/refresh mismatch and is not
a runtime-resolver witness. Keep a single object and
one used filament mapped to the right physical head. Avoid carrying the broad
witness's unnecessary original authored variant cardinality into this new case.

Qualify these variants in Native before asserting Ares results:

1. No matching process ID: finalized IDs `[7,7]` with two Standard variants.
   Physical ID2 selects column0. Distinct speeds discriminate column0 from1.
2. Matching ID/variant: IDs `[1,2]` with two Standard variants. Verify finalized
   exported identities/speeds because the CLI can reorder authored vectors.
   Both implementations must select the matching process column.
3. Matching ID but absent requested variant: qualify the defined same-ID
   branch with finalized `[2,2]` IDs and two Bowden Standard process variants
   against the Direct Drive physical assignment. Native normalizes both speed
   columns to the first value, so this is a path control, not an independently
   discriminating first-same-ID-column witness. It is not the no-ID case.
4. Singleton single-nozzle control and an equal-speed-column control. Neither
   may hide a primary failure by being the sole asserted case.

The new primary must retain a fresh successful Native reference and the actual
public API failure before production changes. Compare the complete ordered
first eligible retract/Wipe/unretract sequence with nonempty extrusion/path
premises, including every E/F/XY line. No coordinate/count/value selection is
allowed. Also compare complete Native/API/CLI G-code with only existing
producer-line normalization, and preserve API/CLI byte equality. Sequence
success is not a replacement for whole-output equality.

Supplement these with discriminating non-default process vectors covering each
inventory family: extrusion speed/thresholds, acceleration, jerk/junction,
overhang, support/skirt and travel acceleration. Top-flow is excluded for the
pre-generation producer-order reason above. A percentage-Wipe
case must distinguish its process travel base while ordinary writer travel
remains tied to the physical head. Use complete outputs, not helper return-value
unit tests, source-text assertions, or expected answers derived from Rust.

Keep all fixture bytes unchanged. Synthetic archive metadata must be fixed and
free of local provenance. Retain commands, environments, input/member hashes,
Native exit receipts, selected maps/identities and complete streams.

## Qualification and stop conditions

Before production edits, run the same base and changed test identities in both
modes on the exact base. Freeze the selected inventory covering process
identity/variants, multi-head mapping, motion speeds, support, skirt/brim,
overhang, travel, cooling and tower controls. Preserve each existing comparator.
Require zero PASS-to-FAIL transitions. Do not replace the actual base inventory
with historical suite counts.

After the separately approved implementation, require both-mode original and
new witnesses, all planned controls, `cargo fmt --all -- --check`, workspace
all-target Clippy with `-D warnings`, and browser-WASM compilation. Changed
files remain at most 400 lines. Core stays platform-neutral for WASM, Windows,
macOS and Linux, without new dependencies, I/O, unsafe code or public API.

This document authorizes no implementation by itself. Review the resolver
boundary, consumer inventory and static-group adapter before writing tests or
production code.
