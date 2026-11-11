# H2C manual multi-nozzle nozzle-level minimum-flush parity

## Boundary and dependency

This document specifies the next source-cited rewrite slice only. It does not
implement Rust code, add tests, or change the public API.

The prerequisite boundary is narrowed manual multi-nozzle admission with typed
`NozzleVolumeType::Hybrid` rejection (`PrintConfig.hpp:618-623`, `nvtHybrid`).
This slice must preserve rejection of a manual multi-nozzle
`NozzleVolumeType::Hybrid` declaration while removing only the current
`used_filaments <= 1` admission restriction.

The existing narrowed boundary is intentionally incomplete. Its manual path
admits a multi-nozzle extruder only when at most one used logical filament is
assigned to that extruder, then does not execute the complete upstream flush
grouping algorithm. This slice replaces that temporary narrow admission with
the upstream manual flush path for multiple used filaments.

## Upstream source boundary

Source authority is the pinned OrcaSlicer checkout at revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. The source file is
`src/libslic3r/FilamentGroup.cpp`. The implementation must remain a rewrite of
these upstream functions and data flow, not an Ares-owned grouping design:

| Upstream location | Included source behavior |
| --- | --- |
| `FilamentGroup.cpp:901-922`, `calc_min_flush_group` | Collect sorted used filaments; use the physical nozzle-list cardinality as `k`; rebuild nozzle-level unprintable limits; choose enumeration or k-medoids from the upstream estimate; retain memoryed candidates. |
| `FilamentGroup.cpp:279-311`, `calc_group_by_enum` | Enumerate base-`k` assignments for used filaments, deduplicate equivalent assignments, evaluate preference and flush score, and retain the best assignment plus memoryed alternatives. |
| `FilamentGroup.cpp:69-90`, `fnv_hash_nozzle` | Hash nozzle volume type, right-extruder identity, and optional loaded-filament identity for equivalence classes. |
| `FilamentGroup.cpp:270-275` | Build the per-nozzle hash input from the physical nozzle list and nozzle status before assignment deduplication. |
| `FilamentGroup.cpp:359-363` | Update the optimum only on a strict preference/score improvement, retaining first-minimum behavior for equal candidates. |
| `FilamentGroup.cpp:427-459`, `select_best_group_for_ams` | Group each candidate by owning extruder, calculate AMS color-distance matching with type/support unlink constraints, and account for unavailable inventory. |
| `FilamentGroup.cpp:508-512` | Preserve the upstream minimum-cost AMS candidate choice and first-equal tie behavior. |
| `FilamentGroup.cpp:1253-1268`, `calc_filament_group_for_flush` | Run `calc_min_flush_group`, prepend its result to the retained candidates, and pass the complete candidate set to `select_best_group_for_ams`. |
| `FilamentGroup.cpp:1495-1505`, `calc_filament_group_for_manual_multi_nozzle` | Apply manual-map unprintable restrictions to the other extruder, remove group-capacity limits, and enter `calc_filament_group_for_flush`. |

The source-level contract is therefore nozzle-level: a six-nozzle extruder is
six assignment groups for enumeration and scoring, not one extruder bucket.
The manual map remains an input restriction; it is not the returned
nozzle-level result and must not be copied directly to the output map.

## Rust destination boundary

The destination remains private code under
`crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/`
and its existing private caller in
`crates/ares-core/src/project_slice/gcode_emit/tool_ordering/recommend.rs`.
The likely affected seams are:

- `context.rs`: retain the existing nozzle inventory, volume restrictions,
  unprintable reconstruction, and typed `NozzleVolumeType::Hybrid` rejection;
  remove only the rejection for more than one used filament on a multi-nozzle
  extruder once the complete source path is available.
- `enumeration.rs` and the existing sequence/score support: represent the
  physical nozzle cardinality, nozzle equivalence hashes, base-`k` assignment
  enumeration, strict preference/score selection, and retained candidates
  without collapsing nozzles to extruders.
- `filament_group/mod.rs` and `recommend.rs`: route manual multi-nozzle input
  through the private equivalent of
  `calc_filament_group_for_manual_multi_nozzle` and publish the resulting
  global physical-nozzle IDs through the existing ToolOrdering views.
- Existing private context/result types may be extended only to carry the
  upstream data required by these functions. No new public API, independent
  filament-map fallback, exporter path, geometry path, or motion path is in
  scope.

The existing X2D compatibility behavior and one-nozzle-per-extruder path
remain unchanged. The implementation must not solve this slice by editing
`options/project_variants/filament_maps.rs` or by adding a second grouping
algorithm outside the ToolOrdering seam.

Flush matrices remain indexed by physical extruder. In
`sequence.rs::score_nozzle`, an ordinary map value is already a physical head
and must select `matrices[head]`, as in
`ToolOrderUtils.cpp::reorder_filaments_for_minimum_flush_volume` (`1143-1235`).
Only a manual multi-nozzle map value is a nozzle-group ID: resolve its owning
extruder before selecting the matrix, matching
`FilamentGroup.cpp::full_evaluate` (`124-164`) and
`ToolOrderUtils.cpp::reorder_filaments_for_multi_nozzle_extruder` (`1324-1390`).
The ordinary two-head score must not index the retained nozzle-group vector,
which can contain only one entry. Keep matrix construction, group inventory,
scoring costs, and both paths' sequence algorithms unchanged.

## Included behavior

This slice includes the complete upstream manual flush-group path for a
manual H2C-style multi-nozzle configuration:

1. Preserve the manual extruder assignment as an unprintable constraint for
the opposite extruder, as in `1495-1505`.
2. Expand the selected H2C extruder to its six physical nozzle groups and use
those groups as the assignment domain.
3. Enumerate all used-filament-to-nozzle assignments when the upstream
estimated search is below its enumeration threshold, including the upstream
nozzle hash and filament-mask deduplication. Preserve the upstream k-medoids
fallback when enumeration is too large.
4. Preserve source-owned unprintable limits, flush-matrix scoring, preference
levels, strict score improvement, and memoryed candidate retention.
5. Select among the minimum-flush and retained candidates through the existing
AMS path, including owning-extruder grouping, color-distance matching,
type/support unlink limits, missing-inventory cost, threshold behavior, and
first-equal candidate selection.
6. Publish the chosen global physical-nozzle IDs through the existing manual
ToolOrdering result and preserve the existing downstream order/map consumers.
7. Remove the temporary admission rule that rejects two or more used
filaments on a multi-nozzle extruder. Keep the typed
`NozzleVolumeType::Hybrid` rejection and all unrelated unsupported-context
checks.

The slice must preserve the source's behavior for empty or unavailable AMS
inventory and for loaded-nozzle identity in equivalence hashing. It must not
assume that equivalent physical nozzles remain equivalent when volume type or
loaded-filament state differs.

## Native-first failure witness plan

The first runtime witness must be Native-first and must use the stock H2C
machine shape with `extruder_max_nozzle_count = [1,6]`. The fixture must use
at least two logical filaments on the six-nozzle extruder; the planned compact
fixture is a three-filament manual assignment shaped like `1,2,2`.

The witness sequence is:

1. Run the exact H2C-derived fixture through Native before changing Ares and
retain the complete Native result or exact rejection. Confirm which logical
filaments are used and record the complete filament/extruder/nozzle maps and
whole public output when Native accepts it.
2. Run the same bytes through the current narrowed implementation to retain
the expected admission failure caused specifically by the more-than-one
used-filament restriction. Do not treat this failure as proof of Native
behavior.
3. After implementation, run Ares through the same public API/CLI path and
compare the complete output and result fields to Native. The successful case
must show a nozzle-granular result; it must not manufacture a direct copy of
`1,2,2`.
4. Re-run the existing H2C single-used-filament Standard control, the X2D
three-filament control, and the typed `NozzleVolumeType::Hybrid` rejection
control. Native is run first for each new or changed witness, and the
comparison retains exact failure diagnostics when Native rejects.

The Native-first plan is a behavior gate, not an implementation shortcut.
Use the existing `NativeCase::compare` complete-output seam and its generator-only
helper: only generator identity and clock lines receive the existing treatment
(`GCode.cpp:4217`). Time estimates, footer fields, maps, and all remaining bytes
are compared strictly. No additional normalization, footer filtering, status-only
substitute, or weakened comparison is allowed.

## Deferred behavior

Defer the following adjacent upstream behavior:

- `calc_filament_group_for_match_multi_nozzle` and Auto-For-Match dispatch;
- unrelated TPU/time-optimization grouping paths;
- new AMS hardware modeling beyond the data already exposed by the existing
  context adapter;
- UI arrangement, public data-model changes, and new public APIs;
- GCodeProcessor, extrusion, geometry, motion, exporter, and configuration
  membership changes;
- broad replacement of the existing ToolOrdering scoring or sequence logic;
- fixture families beyond the H2C `1,2,2` witness and its required controls.

The typed `NozzleVolumeType::Hybrid` rejection, existing X2D behavior, and
current unsupported-context checks are retained rather than deferred.

## Spec exit criteria

Before implementation begins, this spec must define the boundary for one
upstream rewrite slice. The implementation plan must cite the pinned
`FilamentGroup.cpp` locations above, name every Rust destination file it will
change, and keep included and deferred behavior separate. The first code
change must be preceded by the Native-first H2C witness plan being executable
through the existing public harness.

## Per-file implementation plan

This plan is the source-cited boundary for the implementation. It ports only
the manual multi-nozzle flush path exercised by the H2C witness. It does not
port Auto-For-Match, TPU grouping, dynamic per-layer regrouping, new AMS
hardware models, or unrelated ToolOrdering paths.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/context.rs`

- Extend the private `Context` with the physical nozzle domain and the
  nozzle-level restrictions required by `FilamentGroup.cpp:1495-1505` and
  `FilamentGroup.cpp:901-922`: physical nozzle IDs, owning extruder IDs,
  per-nozzle volume types, manual opposite-extruder restrictions, and the
  source-owned enumeration threshold/budget inputs.
- Keep the existing matrix construction, printable/geometric restrictions,
  declaration volume restrictions, typed `NozzleVolumeType::Hybrid` rejection,
  AMS capacity parsing, and X2D single-nozzle behavior unchanged.
- Remove only the `used`-filament-count rejection for a multi-nozzle manual
  extruder. Build the manual restriction as a per-filament set of physical
  nozzles rather than copying the configured nozzle map into the result.
- Preserve the existing two-head destination and the current `Context`
  construction boundary; no public context or allocator is added.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/context/inventory.rs`

- Keep `build_nozzle_groups` as the source-cited physical expansion from
  `ToolOrdering.cpp:1503-1527` and expose only the private inventory data
  needed by enumeration.
- Do not change the existing stats validation or typed Hybrid handling.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/enumeration.rs`

- Replace the current two-extruder bit-mask solver with the bounded manual
  nozzle solver corresponding to `FilamentGroup.cpp:199-224`, `279-311`,
  `359-363`, and `901-922`.
- Use the physical nozzle count as `k`, estimate `k^n` with the pinned
  `ENUM_THRESHOLD = 10000` and early-exit bound, build FNV nozzle hashes from
  volume type, right-extruder identity, and loaded-filament identity, and
  deduplicate equivalent assignment candidates by sorted nozzle/filament
  masks.
- Apply manual opposite-extruder and volume restrictions at the physical
  nozzle level. Score every retained candidate through the existing flush
  matrix and layer-order support, then apply strict preference/score
  improvement and the source absolute flush-gap tolerance.
- Preserve the source memory heap semantics and retained candidates. Add the
  source k-medoids fallback boundary from `FilamentGroup.cpp:374-414` and
  `901-922`; for the bounded witness, the enumeration branch is selected, but
  the fallback remains explicit and is deferred from this slice unless its
  dependency can be represented without inventing a second grouping model.
- Return a private result containing both the coarse owning-extruder map and
  the selected global physical-nozzle map. The returned nozzle map must be the
  solver result, never the input `filament_nozzle_map` copied as output.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/sequence.rs`

- Keep the existing Hamiltonian layer sequence/flush scoring as the Rust
  destination for the single-nozzle ordering support used by the witness.
- Add only the nozzle-aware transition inputs needed by the manual solver:
  physical-nozzle ownership and the source loaded-nozzle state. Do not change
  the X2D score or custom-sequence behavior.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/mod.rs`

- Change the private `apply` result assembly so manual H2C uses the solver’s
  physical-nozzle IDs and derives its owning-extruder and volume maps from the
  selected nozzle records.
- Keep the existing `refresh_filament_map` writeback and the manual view
  override, preserving synchronized map/volume/nozzle identity for all
  downstream consumers.
- Keep the non-manual solver and its post-AMS master preference path
  unchanged.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/recommend.rs`

- Keep the existing private caller and static ToolOrdering seam. Pass the
  manual-map restriction and returned nozzle-level result through
  `filament_group::apply`; do not add a second grouping entry point.
- Preserve the existing X2D branch and all unsupported-context behavior except
  the narrowed multi-used manual admission restriction.

### Deferred upstream source boundaries

`FilamentGroup.cpp:427-459,508-512,1253-1268` AMS color-distance matching,
`ToolOrderUtils.cpp:676-800` flow/matching solvers,
`MultiNozzleUtils.cpp:467-666` change-time/nozzle-state simulation, and
`ToolOrdering.cpp:2093-2270` per-layer nozzle rematching/order are adjacent
upstream dependencies. They remain deferred unless the H2C witness proves
that the existing context and sequence seam cannot produce the strict public
Native output without them. No comparison normalization or output filtering
may substitute for these behaviors.

### Bounded implementation destinations and deferred dependencies

The implementation boundary below narrows the earlier complete-path description
to the source branches required by the static H2C witness:

- `enumeration.rs` retains the existing X2D solver and memory heap. Its private
  child `enumeration/manual.rs` ports the physical-nozzle base-`k` traversal,
  nozzle/filament-mask deduplication, preferences, strict optimum selection,
  heap retention and empty-inventory AMS selection. No injective-assignment
  shortcut is permitted: multiple filaments may share a physical nozzle.
- The manual child implements `estimate_dedup_enum_count` (`199-224`) and the
  `ENUM_EARLY_EXIT = 10000000` / `ENUM_THRESHOLD = 10000` selector. Inputs that
  select clustering remain unsupported. `FlushDistanceEvaluator` (`570-635`),
  `KMediods` (`636-900`) and `ClusteringBudget` (`FilamentGroup.hpp:155-163`)
  are explicitly deferred; no substitute clustering or clock/RNG API is added.
- `filament_group/sequence/nozzles.rs` ports the per-nozzle base reorder and
  cyclic extruder/nozzle stitching in `ToolOrderUtils.cpp:1030-1141,1324-1494`,
  reusing the private Hamiltonian scoring routine without inter-layer forecast
  or custom sequences, as passed by `FilamentGroup.cpp:124-196`. This routine
  belongs only to grouping evaluation. The static downstream call remains
  extruder-map based (`ToolOrdering.cpp:3111-3119`), including H2C. `reorder.rs`
  is unchanged; X2D retains its existing scoring and downstream ordering.
- `sequence.rs` ports the `full_evaluate` composition (`FilamentGroup.cpp:124-196`)
  for an empty initial nozzle status and disabled preload. In this boundary,
  `simulate_filament_change_time` (`MultiNozzleUtils.cpp:467-661`) reduces to
  the loaded-filament state per owning extruder: switching physical nozzles
  does not eliminate AMS-channel clearance or the associated load/unload time.
- `context.rs` reconstructs opposite-extruder restrictions before source
  intersection removal, and retains requested per-filament volume restrictions.
  Nonempty nozzle statistics and loaded-nozzle/AMS hardware state are not
  admitted by this new manual solver. General nozzle-state loading, preload,
  AMS color matching/flow and dynamic per-layer rematching remain deferred.
- `mod.rs` publishes the selected physical IDs, deriving used-filament
  extruder/volume values from their nozzle records. Unused solver slots retain
  the upstream primary-map master default or remembered-map zero default;
  input nozzle IDs are not used as a solution.

The bounded enumeration, full evaluation, static extruder order and G-code
consumers must together satisfy the unchanged complete-output witness. Physical
map equality alone is insufficient. `GCode.cpp:223-274` distinguishes static
BBL hotend placeholders (`-1`) from physical nozzle IDs, while the incoming
material-change configuration at `11068-11082` supplies both identities. These
consumers are not part of the grouping evaluator; their existing Ares
extruder-only scaffolding remains a temporary compatibility shell until the
corresponding source slice is ported. This is not a claim to port every branch
listed in the broader source boundary above.

## Static G-code identity consumers

The static grouping result must reach its G-code consumers without conflating
physical nozzle IDs, owning extruder IDs, and firmware hotend selectors.
Included source behavior and Rust destinations are:

- `GCode.cpp:223-274,11068-11082`: static BBL `current_hotend` and
  `next_hotend` are `-1`; non-BBL hotends remain owning-extruder indices.
  `tool_transition/change/dynamic/mod.rs` replaces its X2D-only selector
  special case and supplies the old/new physical nozzle identities alongside
  the existing logical filament IDs. Dynamic nozzle maps remain unsupported.
- `GCode.cpp:3738-3741,4398-4402`: initial hotend values follow the same
  static-BBL rule. `machine/first_filaments.rs` replaces its X2D-only
  compatibility shell without changing temperature indexing or physical-head
  first-filament lists.
- `GCode.cpp:4408-4419`: only printers with multiple nozzles on a head add
  the signed `H-1` field to the initial `;VT` marker. `layer_gcode/startup.rs`
  uses the configured nozzle inventory, not the selected map cardinality.
  Single-nozzle markers retain their source byte shape.
- `GCode.cpp:241-247,11237-11267` and `GCodeWriter.cpp:803-831`:
  `motion/tool_selection.rs` resolves static physical nozzle IDs from the
  grouping result represented by `filament_nozzle_map`, uses those IDs for
  writer commands, and stores the static hotend selector separately from
  the owning head. Its existing private accessor remains the identity seam;
  no public identity API or allocator is introduced.

The changes above do not alter `PlaceholderParser` evaluation or broaden input
admission. General dynamic nozzle migration, loaded hardware status, preload,
AMS matching, calibration-only writer paths, and per-layer rematching remain
deferred. Nozzle-indexed diameter/volume template arrays and parked-retraction
state propagation are adjacent G-code consumers; port them only if the strict
witness requires them, first naming their source producer and Rust destination.

The strict witness also requires the static physical-nozzle owner to survive
into `GCodeProcessor.cpp:2160-2343`. The included processor slice is
`UsageBlocks` owner resolution and `GCodeProcessor.cpp:2217-2223`,
`nozzle_group_result.get_nozzle_from_id(...)->extruder_id`, defaulting to head
0 when absent. `filament_group::apply` retains its existing physical groups
alongside the refreshed views; `recommend.rs` installs those groups in
`ToolOrdering`; `orchestration.rs` passes a borrowed slice through
`processor.rs` to `pre_heating.rs` / `pre_heating/injector.rs`. The processor
must not reconstruct an ownership table from counts or filament maps. Static
single-nozzle paths retain one group per head, matching
`MultiNozzleUtils.cpp:319-369`.

`tool_ordering/nozzle_group.rs` also reads the startup nozzle count through
`ProjectSettingsRaw::iter` and the existing strict `DynamicPrintConfig`
`IntsNullable` decoder (`Config.hpp:1115-1135`), rather than splitting raw
array tokens. The conditional processor extension is now triggered by
`impl-r8`: official and seeded each ran five tests with four passes and the
first strict residual at byte 64198, line 1301, Native `M632 S1 N R W` versus
Ares `M632 S1 W`. Include the `add_M104_lines` count input and `M632 N R`
emission (`GCodeProcessor.cpp:2333-2343`): `orchestration.rs` supplies the same
typed count reader through a private borrowed integer slice in `processor.rs`;
`pre_heating.rs` forwards it to `injector.rs`, which uses
`value_at_or_front(counts, target_extruder, 1) > 1` for the literal ` N R`.
Dynamic `H` addressing and other processor behavior remain deferred. No
generic dynamic configuration channel is added, and unrelated single-nozzle
behavior remains unchanged.

Existing unsupported paths stay rejected; the unchanged full-output comparator,
time estimates, footer, and raw API/CLI equality remain the verification
boundary.

## Non-wipe-tower pre-toolchange timelapse gate

The strict witness reaches a separate non-wipe-tower upstream rewrite
slice in `GCode.cpp::process_layer`, not a change to minimum-flush scoring.
Include the complete small non-wipe-tower insertion gate at `GCode.cpp:7513-7539`
and the inline-insertion synchronization at `GCode.cpp:7476-7478`.
`tool_transition.rs::emit` is the Rust destination: replace its partial
pre-toolchange predicate, using the existing `FarthestPointTimelapse` state from
`layers/farthest.rs`, the current logical filament's owning extruder, and the
existing `most_used_extruder`. No new public state or identity abstraction is
introduced. The existing Ares predicate is only a temporary compatibility shell
for this upstream gate, not an independent timelapse design.

Included behavior:

- Synchronize the layer's `has_insert_timelapse_gcode` with the retained
  `inserted_this_layer` before evaluating the gate.
- Retain all ordinary prerequisites: traditional mode, not already inserted,
  actual logical tool change, exactly two configured nozzle-diameter entries,
  and a currently selected writer filament. Physical nozzle IDs do not replace
  logical filament identity or owning-extruder identity here.
- Suppress this ordinary insertion in Case A: farthest-point timelapse enabled
  and the farthest target belongs to the photo head.
- In Case B, enabled with the farthest target on the other head, invert the
  current-head test: insert when the current owning extruder is not the
  most-used/photo extruder. With farthest-point timelapse disabled, retain the
  ordinary current-head-equals-photo-head test.
- Preserve the chosen insertion order: ordinary retract, pending object-change
  labels, existing timelapse append, mark inserted, then the existing
  `set_extruder` equivalent. Do not compensate by changing retraction lengths,
  shared retraction-state ownership, or object-label primitives.

Before implementation, debugger captures on the unchanged r9 source establish
Case-A operands on both sides at the first differing transition: layer0,
logical filament1 to2, current owning head1, most-used head1,
`enabled=true`, `farthest_is_photo_head=true`, and
`inserted_this_layer=false`. Native suppresses this early insertion; the partial
Ares predicate enters it. Native locals optimized out by the authority binary
remain explicitly unavailable; the measured false Case-A conjunct suffices to
establish Native suppression. The first incomplete Ares debugger attempt is
excluded; the completed capture supplies the same catalog argument as the
parity CLI adapter. These are root-cause diagnostics, not full-output parity.

Add two non-H2C no-tower dual-color Native witnesses with explicit
`farthest_point_timelapse=1` and `=0`, traditional mode, and both colors actually
used. The enabled witness must exercise Case-B inversion, complementing the H2C
Case-A witness; the disabled witness pins the ordinary branch. Retain exact
Native input/output and runtime branch premises before production edits. Run
both witnesses and the original H2C/Hybrid/X2D controls in official and seeded
modes. Require the unchanged generator-only ordered-byte comparator for both
public API and CLI, independent raw API/CLI equality, and complete timing/footer
comparison; do not substitute local sections or map equality for full parity.

Deferred behavior remains wipe-tower-specific insertion and `wipe_tower/type1`,
changes to the farthest-target producer or inline snapshot algorithm, generic
timelapse redesign, wrapping-detection changes, and other toolchange paths.
The only inline-path addition here is synchronization of its already-retained
insertion flag. Existing unsupported input paths and dynamic nozzle-map `H`
addressing remain unchanged. This small generic gate is included in full rather
than specialized to the H2C witness.

## Grouping-evaluation per-head nozzle carryover audit

`ToolOrderUtils.cpp:1463-1488` updates the last nozzle for every extruder with a
nonempty fragment, then commits the whole per-head vector after the layer. The
last extruder is a separate scalar. In the already-included
`filament_group/sequence/nozzles.rs::sequences` destination, retaining only the
final head's nozzle is an incomplete compatibility shell for this source state.
When a multi-nozzle head is visited before another head, its next cyclic tour
must start at its own last nozzle, not an older layer's nozzle.

The original H2C witness has zero load/unload times and cannot establish this
state's contribution to candidate score. The single carryover discriminator
keeps that witness's geometry and all other archive members byte-identical,
sets both heads' off-diagonal flush costs to1, and sets load/unload times to30s.
Native-first official and seeded exports both choose physical nozzle map
`0,2,1`; the Ares output before this change chooses `0,1,1`. This
field-only diagnostic is not the full-output mismatch: register the same input
as a strict Native/API/CLI E2E on the completed timelapse gate before changing
carryover, including actual use of all three filaments and retained raw streams.

Only after that E2E distinguishes the implementations, port the per-head nozzle
update in this existing evaluator. Preserve cyclic nozzle/head traversal,
nonmanual behavior, and existing score arithmetic. Downstream `reorder.rs`,
extruder/retraction registries, the grouping admission boundary and other source
algorithms are unchanged. Do not introduce another scoring model. If this one
candidate does not distinguish Native and Ares, record that result and stop;
further fixture expansion or carryover changes are outside this audit slice.
