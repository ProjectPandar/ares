# Region-owned perimeter and fill implementation plan

## Current merge scope and historical source notes

Single-tool canonical region consumers are integrated. Actual per-tool GCode
emission is not: expanded inherited-second-filament validation is RED, with a
scope separated by user approval into the next ToolOrdering milestone, with
the mainline baseline body defect reproduced. Final verification follows the Tree plan's
Normal + Tree merge gates. The records below are historical;
claims that already integrated consumers remain unported are superseded.

### Archived architecture analysis

# ARD0089 — Region-owned perimeter generation and fill handoff

Status: canonical region ownership implemented; modifier output controls accepted.

#### Source owner and Rust destination

Rewrite `OrcaSlicer/src/libslic3r/Layer.cpp:139-291`
(`is_perimeter_compatible`, `Layer::make_perimeters`) and
`LayerRegion.cpp:82-142::LayerRegion::make_perimeters` in
`ares-core/src/project_slice/perimeters/`. Carry the resulting region ownership
through `prepare_infill/`, fill grouping / `fill_entities/` and the existing G-code emission adapter,
following `PrintObject.cpp`'s region/layer processing and
`Fill/Fill.cpp::group_fills` / `Layer::make_fills`.
Include `extrusion_islands/`, `island_print_order.rs` and `seam_placement/`:
their upstream owners are `GCode.cpp::GCode::process_layer` (extruder/object/
island/region grouping and emission) and `GCode/SeamPlacer.cpp::SeamPlacer::init`
(region perimeters and physical-layer seam preparation).
The former one-record-per-layer shells are replaced by canonical per-region
records and shared physical-layer cache/scheduling. They are not an
independent Ares pipeline to preserve or grow.
Required configuration prerequisite: `PrintApply.cpp::generate_print_object_regions`,
`PrintObject.cpp::region_config_from_model_volume`, `Print.cpp::object_extruders`,
`PrintObject.cpp::object_extruders` and
`PrintRegion.cpp::collect_object_printing_extruders`. Carry modifier-derived
region roles through `project/effective_config/` usage and
`project_slice/extruders.rs`, reusing the source-owned bounds/region graph
semantics rather than merely removing early option rejections.
Shared destination: `project/region_graph.rs` and `region_graph/bounds.rs`.
`ResolvedProjectObject::all_regions` carries the candidate pass's deduplicated
source-created configurations; pre-region object shells retain an empty list.
The slicing adapter adds released occurrence IDs and actual bound indices to
neutral source-volume records, not fabricated IDs in early normalization.

#### Non-negotiable decisions

- GCode.cpp6155-6178/7635-7636 reads positive sparse density from the current
  nonempty emitted infill region. Update that consumed operand by canonical
  region identity before chaining/travel, including ironing; empty phases do
  not change active configuration. Raft uses the print default region installed
  at5341. An object-wide any-positive flag is not the upstream state transition.
- Modifier density must reach canonical parent-derived region options after
  all_regions usage gathering is implemented. PrintApply.cpp1029-1054 and
  PrintObject.cpp3664-3720 own this merge; admission alone is not output parity,
  and downstream differences remain failures of the full public comparison.
- GCode.cpp5796-5805's clipped-empty loop produces no emission-side changes:
  preserve the preceding wipe state and do not execute wipe-before-external or
  inward motion. Treat this source-defined empty result as an emission state
  transition, not an unsupported-feature guard or silent support omission.
- Infill chaining preserves source region ownership through collection
  flattening and thin-fill transfer. GCode.cpp6150-6178 chains separately inside
  each region, including ironing, using the continuously updated output cursor.
  The flat output shell carries explicit regional identities, never inferred
  from path coordinates or role. This finite rewrite does not certify regional
  motion config or mixed-region infill-first dispatch.
- Width headers consume each shared ModelObject all_regions once, in source
  order, with the first actual PrintObject's fallback/support operands and
  print-wide support presence. PrintApply.cpp1680-1772 shares region ownership
  across transforms; its global registration loop does not insert into its
  lookup set. Do not deduplicate equal configs across distinct objects or local
  region IDs across owners. Header consumption borrows existing configurations;
  this does not supply global IDs to downstream tool/config consumers.
- Perimeter preflight validates all printing regions and physical layer heights
  before derived Flow computation. The single-region rejection is temporary
  scaffolding to replace, not an upstream capability boundary. Public modifier
  output must exercise the real downstream stages; their defects cannot be
  concealed by restoring an unsupported guard.
- Keep one compensated object owner, with its original region identities,
  options and layer slices. Do not clone objects to simulate regions, replace
  region slices by aggregate islands, or apply region0 configuration globally.
- Normalize/collect usage from all source-created regions, including modifiers.
  A density override can activate inherited sparse/solid filament roles; model
  part candidates plus modifier's generic extruder selector are insufficient.
  Do not remove usage-affecting rejection while that prerequisite is missing.
- Keep configuration usage distinct from generated-path tool usage. The header
  in `GCode.cpp:2623-2632` serializes one-based IDs from
  `Print::get_slice_used_filaments(false)`, populated by
  `Print.cpp:2457-2514` from `ToolOrdering`. Neither the count of configured
  filaments nor their unfiltered configuration usage is that list. Preserve
  path-to-region-to-filament ownership before wiring the header and start/end
  placeholders; a count-to-list formatting patch alone is insufficient.
- Keep print-wide configuration usage distinct from per-object slicing
  parameters. `Print.cpp:451-489` includes raw positive layer-range extruders,
  even outside the object's occupied bounds. `PrintObject.cpp:3784-3811`,
  used by `slicing_parameters`, collects actual `all_regions()` and volume
  extruders, without that raw-range loop. Do not add unused range selectors
  to `project_slice/extruders.rs` by conflating these two source functions.
- Keep physical layer identity distinct from a region-layer storage slot.
  Region-local neighbors and aggregate physical-layer neighbors are different
  source inputs; expanding a vector does not make its index a layer number.
  G-code scheduling and active/cooling layer IDs use the physical layer plan
  (`GCode.cpp::collect_layers_to_print/process_layer`), never a filtered stream
  of present region records. Keep the physical plan borrowed from its owner.
  Retraction internal islands borrow every region's slices on that physical
  layer (`RetractWhenCrossingPerimeters.cpp9-55`); do not union disjoint region
  islands or clone geometry to flatten storage.
- Storage order is not execution order. Audit stateful generator consumers,
  including Classic materialization's shared `SlicerContext` fuzzy RNG and
  the Arachne path. Preserve source-selected object/layer/group/surface traversal;
  sorting generated records after RNG consumption cannot repair changed bytes.
  Region-major storage grants no change to native reference scheduling or RNG.
- Generate perimeter groups in the upstream region order, using the complete
  `is_perimeter_compatible` condition. Group surfaces by `extra_perimeters`
  and use the upstream safety offset before generator dispatch.
- Within a compatible group, the strictly greatest sparse infill density
  selects `layerm_config`; ties retain the first member. Native code calls
  `layerm_config->make_perimeters`, whose output pointers are
  `&this->perimeters` and `&this->thin_fills`. Consequently these outputs belong
  to that selected region. The earlier Layer.cpp comment saying the first
  compatible region owns them does not override the actual pointer path.
- Generator input for a merged group and original region input for surface
  typing must be separate views. Classic and Arachne use merged group surfaces;
  region surface classification must still see the original region slices.
- Split generated fill surfaces and no-overlap geometry back by intersection
  with each member's slices, preserving upstream surface metadata. Recover
  counterbore extra fills into the first compatible member as upstream does;
  this exception does not change the perimeter output owner.
- Surface typing, shell propagation and fill grouping use actual region options
  and source-selected neighbor scope. G-code emission visits all regions of a
  physical layer once; it must not duplicate whole-layer paths for every region.
- Public project bytes to complete ordered G-code remain the acceptance seam.
  Removing a rejection alone, reaching a later rejection, or passing a geometry
  example is not modifier parity. No new rejection or silent-skip substitute.

#### Included and deferred

Regional Lightning expansion follows PrintObject.cpp2593-2723: original lower
surfaces with own pattern determine gates; current regions stop at the first
empty Internal region and consume only own candidates. Preserve per-candidate
offset/intersection before solid union and sparse subtraction, including holes.
Destination candidate_expansion.rs and lightning_expansion.rs retains immutable
original horizontal surfaces, using canonical region/layer slot keyed temporary
expanded surfaces only for Generator consumption. This replaces the front-only
expanded map; no whole-object copies or mutation/restore are necessary to preserve
candidate identities. Public modifier/Tree and broader ownership remain required.

Lightning generator outlines follow
`Fill/Lightning/Generator.cpp:151-171::generateInitialInternalOverhangs` and
`:180-193::generateTrees`: iterate every physical layer's regions, retaining
Internal/InternalVoid contours and holes in region/surface order, without a
regional pattern/density gate. Destination is
`candidate_expansion.rs::build_lightning_generator`; canonical region-layer
addressing supplies physical geometry. Configuration ownership/activation is
already ARD-0091. Regional bridge expansion uses canonical region/layer slots:
each temporary replacement applies only to that region, while all unprocessed
regions retain their original surfaces, including after the first-empty break.

Deep sparse bridge operands follow PrintObject.cpp2819-2846. Lower traversal
uses physical depth and iterates all region surfaces with each region's density;
shared sparse/non-sparse morphological operations run after that aggregation.
Views borrow geometry and preserve source order, not copy an object or reuse
front-region density for later regions. Sparse anchoring/lightning and public
modifier output parity remain required adjacent consumers.

Bridge expansion follows PrintObject.cpp3154-3214 and3242-3265: the first ordered
candidate's own thick Flow controls shared spacing and depth, every region
contributes current-layer surfaces/boundaries/pattern, and each candidate
retains its own Flow/configuration. An empty front-region record cannot
discard a later-region candidate. All-region deep lower geometry and sparse
anchoring remain separate required consumers, not fallback substitutes.

Bridge surface rewrite follows PrintObject.cpp3315-3391: shared current cuts
and upper ensuring operands belong to physical layers, while upper Flow and
local perimeter Flow belong to their original regions. Iterate all regional
outputs through canonical region/layer addressing; do not infer physical
layer identity from a flat storage slot. This finite consumer rewrite does
not establish regional lightning/expansion or modifier output acceptance.

Emission must preserve `GCode.cpp:5453-5468`'s separate ordinary-infill and
ironing phases. Ironing cannot run inside the earlier infill phase when
infill-first walls remain pending. The single-region rewrite belongs to
`gcode_emit/motion.rs` and is a consumed prerequisite, not an alternative
regional pipeline; regional ownership/configuration switching remain deferred.

Included in this rewrite chain: modifier-derived configuration/usage roles,
region/layer addressing, compatible perimeter
groups, source-selected flows and dispatch, region-owned generated paths,
fill-boundary redistribution, downstream region-aware typing/shells/fills and
island/seam preparation and emission. Preserve already successful single-region
complete output and seeded fuzzy behavior.
The configuration graph follows `PrintApply.cpp`'s float3D full/range-clipped
triangle bounds, composed transforms, contour inflation and epsilon, source
volume order, reverse parent traversal and changed-config/fallback handling.
Share that implementation across normalization and slicing; no whole-object
copies, invented modifier parents or whole-mesh bounds for every Z range.

Deferred adjacent behavior: support-region traversal and actual Tree generation
remain the separate SupportMaterial / Tree source slices in ARD0084/0086.
Native cross-job cache persistence and parallel first-arrival reference policy
remain unresolved; this design grants no native scheduling/cache modification.
Normal support cannot merge before Tree parity. This plan is not acceptance
of the existing downstream `multi_region_layer_slices` rejection.

#### Perimeter handoff storage

Compatible-group preparation owns merged generator surfaces separately from
the original region slices. A physical-layer/group execution list orders
shared fuzzy RNG; region-layer slots retain typed predecessor alignment.
Singletons borrow original surfaces without an offset. Multi-member groups
own safety-offset surfaces bucketed by extra perimeter count. Both generators
consume that view; classification continues to see original region slices.
Intersect generated fill outputs back into the members before surface typing.
Existing configuration and downstream single-region gates remain until
complete modifier emission is implemented and verified.

Compatibility is not an equivalence relation: ConfigOptionFloat and
FloatOrPercent use strict absolute `1e-4` comparison; ConfigOptionPercent
and explicitly compared `.value` fields use exact equality. Serialized
wall widths instead compare native six-significant-digit tokens, including
percent tags and signed zero. Do not add a `done` check to upstream's later
member scan to force disjoint groups. With non-transitive comparisons a
previously processed member may participate again, even as generator owner.
The current one-record-per-region generator shell cannot yet represent
these repeated invocations and RNG consumption; separate invocation storage
from region output storage before removing the multi-region gate. Regional
fuzzy partitioning also remains a required generator-consumer rewrite.

The temporary physical-layer fill shell stores perimeter collections once,
with per-region ranges matching all region-major slots, including empty ones.
This is a borrowed view of LayerRegion::perimeters, not duplicated aggregate
geometry. Route every slot to its physical layer during move_thin_fills;
preserve first allocation and region order. Regional support bridge views
must borrow only that region's range. Thin-fill routing moves every record,
but regional fill collection ownership and downstream tool/island grouping
remain unported. This storage correction alone does not establish modifier
or Tree parity and does not authorize the Normal batch to merge.

Ranges are valid only before extrusion_islands moves perimeter collections
out of the physical-layer shell. Current support views borrow before that
handoff. Later regional consumers must carry ownership with moved entities,
not reuse stale ranges. Regional fill collections require their own selected
owner view rather than repeating an aggregate test for every region.

#### Regional Flow consumer ownership

Bridge candidate gathering follows `PrintObject.cpp::bridge_over_infill:
2493-2589`: build one borrowed view per physical layer, retaining region order,
original surface indices and a single first-region solid spacing. Aggregate
all lower-region fill boundaries and density-selected solids before morphology;
filter each current region against that shared unsupported area. The Flow
getter (`LayerRegion.cpp:21-28`, `PrintRegion.cpp:25-53`) is independent of
surface presence: resolve first-region solid Flow from its configuration when
its canonical input is absent, never borrow a later nonempty region's Flow.
Candidate addresses remain physical layer + region + original surface index.
No geometry/object copies or Classic scratch dependency. Later bridge
lightning/expansion/rewrite regional addressing remains a separate source port,
not a reason to redefine candidate layer identity. Keep the public modifier
and Tree output gates open until that complete consumer chain is ported.

Regional surface processing owns its Flow operands. `Flow.hpp:62,69`,
`PrintObject.cpp::detect_surfaces_type:1577`,
`LayerRegion.cpp::process_external_surfaces:497-518` read the current region's
flows, not PerimeterGenerator scratch. Rewrite those two consumers to use canonical
regional inputs; retain double scaling and integer truncation. No duplicated
prelude geometry or scaled-flow cache is needed. Remaining shell consumers
must be similarly decoupled before generator storage becomes an invocation
sequence. Bridge candidate gathering instead uses the physical layer's front
region Flow (`PrintObject.cpp:2512`); port that with its all-region operands
and physical-neighbor traversal, not as a per-region Flow substitution.
This prerequisite does not enable modifiers, fix regional neighbor
walks, or establish multi-region output parity.

#### Shell Flow consumer ownership

PrintObject.cpp2008-2446::discover_vertical_shells separates physical idx_layer
from region_id. Regional storage slots must not index physical plan layers or
walk into another region's cache. Source top/bottom cache is shared only for
multiple printing regions with interface_shells disabled; otherwise it is
region-local. Holes still cover every region's fill_expolygons. The shared
branch also consumes regional Flow and global lslices for simulated perimeter
offset/min-spacing and extra holes. Removing a one-region assertion without
porting these operands and neighbor scope does not establish regional behavior.

The cache destination now retains canonical regional presence records plus one
physical cache bank for multiple regions. Its two source states are shared
top/bottom/holes, or regional top/bottom with shared holes. Projection borrows
views from that bank and uses physical layer indices with a region-scoped row;
it does not duplicate the shared geometry per region. Shared-cache Flow remains
defined for empty regions, as native LayerRegion::flow does: SolidInfill first,
then External and Internal only for a positive simulated perimeter count, never
Overhang. The source spiral num_layers limit applies to the whole shell
operation: cache construction, projection visits and trimming activation;
inactive trimming preserves surfaces through downstream no-op assignment. This
implementation boundary does not imply public modifier acceptance while earlier
perimeter and later shell consumers remain incomplete.

Downstream discover_vertical_shells consumers preserve per-region surfaces and
Flow while indexing visibility neighbors by physical layer, as
PrintObject.cpp2314-2440 does. Canonical region/layer slot identity replaces
the temporary single-region cardinality assumption. A slot must never become
an index into global lslices. The spiral execution window bounds trimming;
staged assignment/disposal remain unchanged. Regional downstream ownership
alone does not establish complete public output while other consumers remain
unported.

The regional `discover_vertical_shells` Flow ownership
(`PrintObject.cpp:2132,2174-2175,2242/2269,2334-2403`) likewise belongs to
canonical region inputs in vertical_shells/projection/regularization/filtering,
not generator scratch slots. Remove generator-count/presence assertions and
their private structure-pinning subcases at these consumers, retaining canonical
identity and physical-layer invariants. This does not port the shared
all-region cache branch2024-2106 or regional neighbor traversal. Other stages'
generator alignment assumptions must still be removed before invocation
storage can change. No gate removal or multi-region parity claim follows.
Keep current-versus-stopped-neighbor spacing behavior coverage. The approved
supplementary test injects into canonical regional external Flow rather than
generator scratch, retaining all four behavioral assertions. This does not
replace the public complete-output seam or imply integer-injection bit parity.

#### Fill collection handoff storage

PrintObject.cpp3955-4161::discover_horizontal_shells traverses region first,
then physical layer. Horizontal promotion/propagation entry cardinality is
regions×layers; slot identity is not physical i. Region-local options and existing
source/neighbor canonical Flow remain required, as do same-region serial
propagation, staged/dirty commit and rollback. Replacing temporary one-region
validation does not approve public parity while earlier entry and adjacent
consumers remain incomplete.

Shell alignment follows canonical region outputs, not generator scratch.
`PrintObject.cpp::discover_vertical_shells:2334-2435` and
`discover_horizontal_shells:3955-4161` consume LayerRegion surfaces and physical
layer geometry. In vertical_shell_trimming/filtering/assignment and
horizontal_shell_promotion/propagation, retain canonical input, cache, plan,
scale and identity checks, but remove scratch count/presence-only dependencies.
`Layer.cpp:139-291` can invoke a compatible owner repeatedly; call records
cannot define final region slot cardinality. Remove only corresponding private
test subcases. Preserve transaction/geometry coverage and the four spacing
assertions. Physical-layer neighbor rewrites, invocation storage, candidate
front-region Flow and all-region output acceptance remain deferred.

`Fill/Fill.cpp:1274,1354-1356` appends generated collections to the selected
`SurfaceFill.region_id`, not every member of region_id_group. Preserve the
flat generation order and record each append's region and collection range;
one region may have multiple noncontiguous ranges. Regional bridge detection
borrows only those ranges. Simplification changes paths, not collection order.
The current single-region ironing shell records its actual input region;
native ironing parameter grouping and selected group owner
(`Fill.cpp:1507-1724`) remain deferred, not replaced by a loop over regions.
Thin gap fills remain separate and are not bridge paths in this view.
Ownership ranges are valid before extrusion_islands consumes collections;
later consumers must move ownership together with entities. This repairs the
regional bridge view but does not itself port all-region generation,
support assembly, ToolOrdering, or Tree generation.

#### All-region fill grouping

Lightning activation scans printing regions for positive-density Lightning
(PrintObject.cpp1010-1020), but Generator.cpp66-100 takes configuration from
shared_regions()->all_regions.front(). build_lightning_generator must not
select the activating region's width/density/angles as its parameter owner.
This source-specific global ownership differs from regional anchor filler
parameters. All-region outline gathering and regional expansion remain required
adjacent ports; no geometry aggregation or output acceptance is implied here.

Sparse anchor generation belongs to the physical Layer boundary
(PrintObject.cpp2736-2756; Fill/Fill.cpp1394-1512), not its front region.
candidate_expansion/layers.rs must call the all-region sparse anchoring
generator once per required lower layer. Regional option validation borrows
each present region's canonical surfaces and resolves its own nominal Flow;
absent or zero-density front regions do not suppress later groups. No extra
geometry copies or per-region generator instances. Lightning construction and
adaptive behavior remain separate required ports, not claimed by this seam.

Rewrite `Fill/Fill.cpp::group_fills:828-1053` in group_fills.rs, params.rs
and coalesce.rs. Visit physical-layer region slots in upstream region order;
skip aligned absent records, not other regions' present surfaces. Project all
surfaces with one shared SurfaceFillParams state and one LockedZag map builder.
Intern parameters globally before assigning output groups. The first surface
sets the region owner and representative metadata; later distinct region
members merge no-overlap geometry once through upstream union_ex. Preserve
parameter order, member order and surface order. Apply priority clipping and
narrow splitting once to the combined groups, not independently per region.
Native internal-void extension remains a separately audited adjacent branch
(`Fill.cpp:1076-1152`); this change does not invent synthetic void fills.
Compatible perimeter execution, multi-region ironing, later island/tool
ownership and all-region support assembly remain open. No new rejection or
permission to remove existing downstream guards without their consumers.

## Active-region retraction density continuation

Quality continuation: pre-quality Clippy60906 now exits0/2m16s with326
testwarnings309duplicates. The one added warning is emit_layer six arguments
against this repository's threshold5; it is a new required repair, not baseline.
Group borrowed geometry and its density reader as one tuple argument, without
adding an abstraction or lint suppression. No arithmetic/state/order change.
fmt49705 and diff exit0. Final layer SHA7a72e81feb5516b9cce39e5a2b62e069c3d7e77027445c239d26155c650d3cde;
object_entry SHA887557f619a48dfea2e06c3a6e8d6aa1c47c7a369b75904d6cc453db5427b42c;
raft_entry SHA1fa4901ad0e0f58531300a41eb0d2761ae4a95d408de165eed4a2105acd34446.
New final Clippy20115 `/tmp/ares-active-region-density-final-clippy.log` and
public3-case native66995 `/tmp/ares-active-region-density-final-native.log`
are live/queued. Keep prior full15591/controls90783/official65840/platform
31251/85103/76766 alive until terminal too; no restarting due to observation
timeout. They were launched before this signature-only quality repair and do
not replace final-snapshot gates. Seek independent review of the repaired
signature and actual final outcomes. No dependent WIP submission.
Independent six-axis signature re-review verifies the final layer hash and
tuple destructuring: identical borrowed geometry/getter, state timing and
region identity, no allocation/dynamic dispatch/suppression. The argument count
is restored to5; actual final Clippy/native remain required and pending.
Final-snapshot full88135 `/tmp/ares-active-region-density-final-workspace.log`
is now launched alongside those existing final handles; preserve it until
terminal. It is not a restart of the still-live pre-quality full15591, whose
results remain historical evidence for its compiled source only. Source remains
unchanged while collecting all current checks. Final fmt39126 is launched;
diff check exits0. Original KSR fixture and spacing four assertions remain intact.
Final fmt39126 exits0. Seeded controls90783 now exits0/21.686s7run7PASS293skip:
original density21.252s, bridge-flow19.894s, width21.683s and all four prior
controls pass complete ordered output. This does not include the second-tool
tracer or certify full option/printer domains. Official65840 exits100/2.739s,
3run2PASS1 LightningFAIL297skip, retained without command filtering. Other
current handles remain live; do not replace final full/native/Clippy evidence
with these historical or narrower checks. No dependent WIP submission.
Prior full15591 is now terminal100/268.347s7513run7512PASS1 original
Tree-widthFAIL9skip37slow. WASM31251/macOS85103/Windows76766 exit0 at7m20/
7m52/8m10 including queue, compile-only. Official65840's exact first difference
is1808/line83: expected extrusion, actual M73 P9 R7; keep the complete RED.
Final Clippy20115 is now actively checking; final native66995 and full88135
remain live/queued. Preserve those three handles, not completed older ones.
Final Clippy20115 now exits0/7m08 including queue,325testwarnings308duplicates;
the new6/5 diagnostic is absent, closing that mandatory quality repair without
suppression. Final native66995 exits0/6.337s3run3PASS297skip after successful
build: bridge-flow5.139s,density6.133s,width6.336s complete comparisons pass.
Final source/test hashes remain unchanged. Full88135 is the only live gate now,
compiling the final signature snapshot; do not restart it or substitute prior
full15591. Independent final-output six-axis re-review requested with these
terminal facts; full global/Tree/multi-tool coverage remains open.
Independent final-signature six-axis re-review verifies all six complete cases
(three control/modifier pairs) after only generated-by removal; density input
matches the original RED SHA exactly. Clippy325/308 baseline and absence of the
new argument diagnostic are verified. No new finite-slice mandatory finding;
final full88135 still pending, official Lightning remains RED and platform
results are compile-only. No dependent WIP/Normal/Tree/global completion approval.
Final full88135 has entered execution: the unchanged original spiral Z-ramp
passes10.425s and the original current-spacing/not-stopped-spacing case passes
31.383s. Independent read-only review confirms PrintObject.cpp2022's physical
bottom-shell window and the matching Rust trimming gate; spiral test has no
diff and spacing's four-assertion file SHA remains
c334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551.
The only failure observed so far is the original Tree header-width rejection.
There is not yet a terminal full summary; preserve88135 rather than declaring
the partial run a full pass. No dependent WIP/Normal/Tree submission.
Final full88135 is now terminal100/253.940s:7513run7512PASS1 original
Tree-widthFAIL9skip35slow. Spiral10.425s and spacing31.383s PASS on this
snapshot. Independent six-axis terminal re-review finds no new finite-slice
mandatory issue; active-region density and spiral regressions are closed.
Official Lightning, inherited second-tool output, Tree generation, repeated
owner invocation, other region configuration and all printer/option/runtime
coverage remain open. No dependent WIP/Normal/Tree/global submission approval.

Source GCode.cpp6155-6178/7635-7636; destinations motion/layer.rs and
layers/object_entry.rs, preserving raft's5341 default-region density.
Use existing native42128's unchanged complete-output RED and raw two-extra-
retraction evidence. Borrow the canonical PostRegion options; expose the density
read to layer emission without rebuilding/cloning configs or object-wide union.
Update only nonempty ordinary/ironing phases before chaining; preserve canonical
identity and region entry/exit, cursor and phase ordering. Move existing layer
emission into a proper module if motion.rs would exceed400LOC.
Run the original density comparison, both support modifiers and other seeded
controls, full regression, fmt/Clippy and independent six-axis actual review.
Other regional config/perimeter/tool/Tree/global gates are not closed by this.
Prior admission full27750 still runs its already compiled prior-snapshot binary;
do not restart or mistake its results for validation of a later source change.
Implemented canonical object-region density getter and explicit raft-default
getter, with update after empty infill return and before chaining. Existing
layer phases moved to motion/layer.rs; files motion275/layer130/object_entry128/
raft_entry98LOC, no macro splitting or config copies. fmt70403 and final check
83034 exit0; diff check0; index empty. Independent static six-axis review finds
no new logical mandatory issue, verifies canonical IDs and positive-to-zero
assignment, and defers actual causal closure until the original tracer passes.
Native51447 `/tmp/ares-active-region-density-native.log` now exits0/6.334s,
3run3PASS297skip after successful1m48 compilation: unchanged later-density
6.152s plus bridge-flow5.141s and width6.333s complete comparisons PASS.
Artifacts `/tmp/ares-active-region-density-native.yLAqbD` include all six
modifier/baseline cases (three pairs); generated-by-only comparator, no M73/retract filtering.
This establishes the specific density operand's contribution for this input,
not all-region config/tool/global output acceptance. Manifest reference producer
provenance remains unknown. Sent independent actual-output six-axis review.
Independent actual-output six-axis re-review verifies all six complete cases
after only generated-by-line removal. Density input SHA matches the old RED
exactly; both extra retract/wipe/hop sequences and the entire ordered-stream
difference disappear. This finite public defect is closed, without approval
of other config/tool/Tree/global gates or dependent WIP submission. New frozen
full/Clippy/platform gates still pending; old admission results are not substitutes.
New frozen gates are live: Clippy60906/full15591
`/tmp/ares-active-region-density-{clippy,workspace}.log`, seeded7 controls90783,
official3 controls65840, WASM31251/macOS85103/Windows76766 with the same log
prefix. Preserve these processes until terminal; no dependent WIP submission.
Frozen motion/layer SHA144e758f0cefa998f312a6f051e1d379fdaa172d18fb98e689456da39dadf224;
motion SHA ddd2b0eab404f972b00fe719284bdad307d2db1dcd2311b6897b0594dade3809.
All prior admission gates are terminal. Full27750 exits100/257.384s,
7513run7512PASS1 original Tree-widthFAIL9skip36slow; original spiral10.711s,
spacing32.148s PASS. WASM60924/macOS99667/Windows19036 exit0 at4m32/5m05/
5m37 including queue, compile-only. Clippy13361 and inherited95457 terminal
results are below. They describe only the earlier compiled admission snapshot.

## Modifier density admission continuation

Fresh RED45465 `/tmp/ares-density-modifier-red.log` exits100/2.698s:
1run0PASS1 density rejection299skip; original public input and assertion unchanged.
Source PrintApply.cpp1029-1054 and PrintObject.cpp3664-3720; destination
effective_config/candidates.rs. Verified region_graph's modifier parent merge
and usage.rs's all_regions sparse/solid-role gathering are already present.
Remove only the stale density rejection and its obsolete rejection-table row,
then run the same native complete-output test. If downstream output differs,
retain RED and trace actual source consumers rather than reintroduce a guard.
Run fmt/Clippy, regression gates and independent six-axis review for the frozen
slice. No dependent WIP submission or Normal merge before complete Tree parity.
Native42128 `/tmp/ares-density-modifier-native.log` now exits100/3.038s,
1run0PASS1DIVERGENT299skip after successful compilation. Admission reaches the
complete output: baseline PASS, modifier first9291/line393 M73 placement,
183400 reference/183661 actual normalized bytes. Full artifacts are retained in
`/tmp/ares-density-modifier-native.tspCK0/case-eXBDot`; input and comparison
assertions unchanged. Raw diff also has two extra retract/wipe/hop sequences
around native lines4500 and4738, and estimated16m18s versus16m19s. Do not
filter M73 or call this geometry-only parity. Manifest producer provenance
remains unknown despite the explicit seed0/CPU0 reference command.
Independent static six-axis review verifies parent-derived density and both
all_regions usage consumers, with no new mandatory admission issue, but requires
the original inherited-second-filament tracer as well as actual full gates.
That tracer95457 `/tmp/ares-density-inherited-filament.log` is now running/queued.
Frozen gates: Clippy13361 `/tmp/ares-density-admission-clippy.log`, full27750
`/tmp/ares-density-admission-workspace.log`, WASM60924/macOS99667/Windows19036
`/tmp/ares-density-admission-{wasm,macos,windows}.log`; preserve live processes.
Source/test files180/342LOC, ARD371LOC; fmt completed80303 exit0 and diff0.
Clippy13361 is now terminal exit0/2m19s325testwarnings308duplicates; it must
not be relaunched merely to collect its completed result. Frozen candidates SHA
60413292dc8e5241633d8a632558ed63e4e7432c7d0834e290fdb8b0b7f68ffc;
unchanged public tracer SHA22640a5eff72bd4ca1cab2c3a4d3761bf03b5bcd0b86afcbac2b27d1fca66ec9.
Next source-cited consumer trace: GCode.cpp6136/6163 applies the emitted
region's config, and7635-7636 tests that active sparse density for retract
suppression. Rust retains infill region identities but builds has_sparse_infill
from one object MotionOptions and does not apply region config in emit_layer.
This is a concrete operand ownership gap; its contribution to the two extra
retracts must be demonstrated by the unchanged complete-output test, not assumed
to explain every M73 difference. Keep current gate snapshot frozen first.
Independent actual-output six-axis review confirms the additional retract/wipe/
hop is real and the active-region operand gap is source-owned. It requires
checking density region entry/exit and perimeter/ordinary/ironing apply timing;
an object-wide any-positive-density shortcut is not equivalent. Full, inherited
filament and platform gates remain pending, so no submission approval.
Inherited95457 now exits100/6.922s1run0PASS1FAIL299skip: both baseline and
modifier emit complete but divergent output, first186/line7 native filament:1,2
versus actual filament:2. This original public assertion is preserved; admission
and usage gathering do not prove multi-tool output parity. WASM60924 and
macOS99667 are terminal exit0 (compile-only; macOS5m05 including queue).
Windows19036 remains compiling; full27750 now executes its compiled binary.

## Clipped-empty loop state continuation

Actual native91398 `/tmp/ares-clipped-loop-native.log` exits0/4.653s,
2PASS298skip: unchanged bridge-flow4.029s and width4.652s complete output.
No assertion/input/comparator changes or command filtering. The original width
case now proves the clipped-empty-state defect's causal contribution, not just
similar source code. Frozen loop_paths.rs SHA c6589505ae324ab85042d4110e4f00bf70497120d8dcf7ac25cf386699a89b9c,
307LOC. Final fmt/diff checks exit0 on the unchanged frozen source.
Full67275 `/tmp/ares-clipped-loop-workspace.log` exits100/257.051s:
7513run7512PASS1 original Tree-width failure9skip34slow. Original spiral
10.544s and spacing31.937s PASS; spacing SHA c334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551
and all four assertions remain unchanged. Clippy11082 exits0/6m04s,
325testwarnings308duplicates; no new loop/motion warnings.
Seeded27416 `/tmp/ares-clipped-loop-controls.log` exits100/22.814s:
7run6PASS1 later-density configuration rejection293skip. Both modifier
complete outputs now pass alongside the four prior controls. Official77846
`/tmp/ares-clipped-loop-official.log` exits100/0.996s:3run2PASS1 Lightning
FAIL297skip, first1727/line80 expected M73 P9 R7 versus actual extrusion;
this unchanged-official result remains RED, not normalized away.
WASM83726/macOS17417/Windows74270 exit0 at5m06/5m29/5m45 including queue;
compile-only, not platform runtime parity. All seven gate handles are terminal.
Sent terminal independent six-axis review; full regional config/tool/phase,
density and Tree/Normal/global coverage remain open. No dependent WIP commit.
Final independent six-axis re-review verifies frozen hashes and all terminal
logs, finds no newly introduced mandatory defect, and closes only this width
input's missing-wipe/full-output defect. Original spiral regression is closed;
it is not treated as a known failure. Normal/Tree, density, regional config,
Tier1 runtime and global gates stay open; dependent WIP submission is not approved.
Independent actual review verifies all four baseline/modifier G-code pairs
using only the existing generated-by normalization; the new width input SHA
matches the old RED input. This establishes the specific clipped-empty wipe
defect's fix. No new static mandatory issue; terminal gates are recorded above.
Eligible scarf construction does not mutate emission state and each emission
branch retains one wipe-before call. Extreme scarf/clipping combinations are
not fully certified; cached reference provenance stays unknown.

All previous frozen jobs are now terminal: full11573 exits100/266.339s,
7513run7512PASS1 original Tree-width failure9skip36slow; spiral10.469s,
spacing32.263s PASS. Official39535 exits100/1.030s3run2PASS1 Lightning
1808/line83 M73 FAIL297skip. Tier1 compile58688/76394/42971 exits0,
8m26/8m55/9m13 including queue; not runtime proof. Final Clippy2217 previously
passed with the two new warnings gone; fmt/diff checks pass. Sent terminal
six-axis review for chaining, not approval of full parity/dependent submission.
Implement the separate loop state rewrite now against the existing real width
RED; prior gate logs describe only the prior frozen source.

Final-quality frozen controls63143 now exits100/21.552s7run5PASS2FAIL293skip;
the original bridge-flow modifier and four single-region/multi-object controls
retain complete output. Width remains RED with its same first21490/line923
and missing wipe; density remains configuration rejection. The complete
workspace11573 and official/platform handles are still live. Independent review
accepts the next empty-loop source boundary, but requires nonempty ordinary
and eligible scarf hop exactly once and no changes to prior seam selection;
it does not yet confirm the width root cause or submission.

Previous turn made concrete progress: region-local chaining produced two
independently checked bridge-flow full-output passes; nested warnings were
fixed and final Clippy passed. Current collection starts by resuming the same
live frozen gates, not restarting them. Pre-quality-fix full32173 now exits100:
258.473s7513run7512PASS1 original Tree-width failure9skip35slow.

Source GCode.cpp5796-5805::extrude_loop; destination motion/loop_paths.rs::emit.
Keep source frozen until prior runtime/reference gates finish. The unchanged
public width-modifier comparison supplies RED and full raw missing-wipe evidence.
Preserve prior wipe state when ordinary clipping consumes every path and place
wipe-before-external after that source-defined empty return; retain the existing
scarf emission branch's hop. No speed adjustments or input/assertion changes.
Re-run width and bridge-flow comparisons to prove/refute the candidate cause,
then full regression, fmt/Clippy, Tier1 checks and six-axis independent review.
Do not call full regional/Tree output accepted or submit dependent WIP.

## Regional infill chaining continuation

Independent actual-output review verifies the bridge-flow artifact's whole
G-code equality after only the existing generated-by normalization; producer
provenance remains unknown in runner_cache metadata. It makes the two added
nesting warnings mandatory repairs. Flatten the perimeter case to push/continue,
retaining identical ordinary/ironing ownership and phases without lint allows.
fmt/diff32017 exits0. New frozen gates: Clippy2217
`/tmp/ares-region-chain-final-clippy.log` now exits0/2m43s325testwarnings,
308duplicates; both newly introduced nesting warnings are absent. full11573
`/tmp/ares-region-chain-reviewed-workspace.log`, seeded63143, official39535,
and Tier1 compile58688/76394/42971 remain running/queued, not accepted yet.
Old full32173 is running the pre-quality-fix equivalent logic, not proof of
the new frozen source. Preserve its handle until terminal as well.

Frozen-source controls71129 exits100/10.740s7run5PASS2FAIL293skip;
bridge-flow repeats full ordered-output PASS9.696s, plus Lightning, both
ironing and unequal-height skirt controls pass. Width and later-density remain
RED. Official99081 exits100/1.553s3run2PASS1FAIL297skip: ironing controls
pass, Lightning first1808/line83 M73 remains divergent. No command filtering.
Reference command remains the explicit seeded wrapper/seed0/CPU0 and separate
official wrapper; re-read actual executable SHA8f0cf69d…cc03d at
`bin/.orca-slicer-wrapped` (the outer C-wrapper SHA is3d05d4cc…bfaa).
Artifacts' runner_cache manifest does not itself certify reference provenance;
do not upgrade its unknown producer field or generalize this output pass to
all-printer/options coverage.

Clippy15921 exits0/3m02s327testwarnings310duplicates (two new nesting warnings
in the touched emission loop; warnings are not a clean-lint claim). fmt/diff
checks exit0. WASM1226,macOS55683,Windows43034 checks all exit0 after the
fixture-pattern correction; compile-only, not runtime gates. Corrected complete
workspace32173 `/tmp/ares-region-chain-final-workspace.log` is still compiling;
keep that same live handle until terminal, not the superseded failed build.

Native64135 exits100/4.605s1PASS1FAIL298skip after successful production
compilation. Bridge-flow modifier full output passes3.968s, manifest/artifacts
`/tmp/ares-region-chain-native.xEM3MV/case-w5KTrL`; its without-modifier
control also passes. Width modifier remains RED4.602s, first21490/line923
G1 F1997 vsF1977, reference308681/actual308588 bytes, case-JmvVoK.
Raw full diff additionally shows one missing native wipe path on that layer;
therefore the speed/M73 differences must not be hidden or treated as solely
processor rounding. Full acceptance comparator is unchanged.
Next read-only trace identifies a concrete candidate state defect:
GCode.cpp5796-5805::extrude_loop returns before all emission state changes when
clip_end leaves no paths. loop_paths.rs can reduce path_count to0 yet still
replace state.wipe_path with an empty vector and call the inward helper. That
could erase the previous emitted loop's valid wipe without any visible extrusion.
This is a source-confirmed semantic gap, not yet a proven cause of case-JmvVoK;
test the unchanged public width input before/after a separate clipped-empty-loop
rewrite, with no synthetic private seam or speed/timing normalization.

Initial workspace39095 `/tmp/ares-region-chain-workspace.log` exits101:
four E0308 test-only inventory matches still expected the old infill enum
instead of the new (region,entity) tuple. Adapt those patterns to the same
entity, leaving every assertion intact; this failed build is not runtime
evidence. Re-run complete workspace on the corrected frozen test source.
Independent static six-axis review finds no new clear mandatory repair for the
finite chaining slice, but requires actual regression gates and retains all
regional config/phase/tool and Tree/Normal gaps. No dependent WIP submission.

Fresh public RED16710 `/tmp/ares-region-chain-red.log` exits100; preserve
both full modifier byte assertions. Source GCode.cpp4971-5068/6150-6178;
destinations fill_entities ownership ranges, extrusion_islands, island_print_order
and motion emission. Carry existing collection region IDs and record-owned thin
fill IDs through flattening, then independently chain each region in source
order, including ironing and continuous cursor updates. No ownership inference,
config clones, output filtering or new guards. Keep existing flat storage only
as a temporary shell; mixed-region phase/config dispatch remains a known gap.
Adapt existing private fixtures to supply actual region ownership, without new
private assertions. Re-run original public comparisons and single-region
controls, full workspace, fmt/Clippy, Tier1 compilation and independent six-axis
review. No dependent WIP commit before Tree/Normal gate closes.

## Regional width-header continuation

Borrowed regional retraction gather is now wired through EntryGeometry and
LayerGeometry. Four empty raft/skirt/support constructors needed Vec::new();
initial runs81994/78039/9780 stopped with E0308 before execution and are not
acceptance evidence. Corrected native63183 exits100/4.595s2FAIL298skip,
`/tmp/ares-regional-retraction-final-native.log`, artifacts
`/tmp/ares-regional-retraction-final-native.caXxLx`: bridge-flow case-ekQSqM
275898 reference/275769 actual bytes, first5031/line205; width case-bEmNP3
308681/308051 bytes, first2870/line119. First differences are M73 positions;
complete output comparisons remain RED, unchanged.

Read-only diagnostic diff (not acceptance normalization) also finds real body
order differences: bridge-flow emits sparse before internal solid where native
does the reverse; width modifier emits gap infill before another region's solid
fill where native emits it afterward. GCode.cpp4971-5068 stores island entities
by region, 5453-5468 runs perimeter/infill/infill-first-wall/ironing phases,
and extrude_infill6150-6178 chains each region independently. Rust
extrusion_islands::assign_layer erases that regional grouping, and
gcode_emit/motion::emit_infills chains the whole island. This is the next
source-owned rewrite boundary, not a processor-only M73 adjustment. Preserve
region identity through fill/perimeter ownership before implementing emission;
do not invent entity ownership from coordinates, role or nearest path.

Corrected seeded17978 exits100/19.296s7run4PASS3FAIL293skip in
`/tmp/ares-regional-header-retraction-seeded.log`: Lightning, both ironing
controls and unequal-height multi-object skirt retain complete output; both
modifiers remain divergent and later-higher-density still rejects
sparse_infill_density. Official9521 exits100/2.255s3run2PASS1FAIL297skip in
`/tmp/ares-regional-header-retraction-official.log`: ironing/timelapse pass;
Lightning remains divergent at byte10027/line437 (M73 position). Producer and
comparison unchanged; this run's exact first difference supersedes earlier
producer timing observations, not the unresolved official parity gate.

Corrected Clippy9338 exits0/6m25s325testwarnings308duplicates. Tier1 compile
7597(WASM core+adapter),15236(macOS core),28552(Windows core) exit0 after the
constructor fixes; compile-only, not runtime acceptance. Full workspace47272
exits100/259.163s7513run7512PASS1 original Tree header-width failure,
9skip35slow. Original spiral10.571s and original spacing32.541s pass; no new
failure in this full inventory. fmt/diff checks exit0. No dependent WIP commit.
Independent static six-axis review finds no new mandatory gather/header repair,
but withholds full regional parity and Tree/Normal submission approval; it does
not certify the unchanged travel kernel or unimplemented native AABB cache.
Terminal independent six-axis re-review confirms the full inventory and original
spiral/spacing passes: the reported spiral regression is closed, not overall
parity. Next repairs retain region config and collection hierarchy during
region-local chaining, with continuous current-position updates. Repeated owner
invocation, fuzzy grouping, density activation and Tree remain open. Reviewer
does not approve dependent WIP, Normal/Tree submission or global completion.

Header-native73002 exits100/4.613s2FAIL298skip. Both width headers now match;
bridge-flow first difference6437/line261 M73 position, width first795/line27
M73 P0 R23 vsR24. Full comparisons unchanged, no modifier GREEN.
Multi-object public29874 exits0/1.466s1PASS299skip: two distinct objects with
equal region configs retain complete ordered output.
Next consumed defect: island_print_order::internal_surfaces currently selects
records[layer_index], i.e. only front region. Native
RetractWhenCrossingPerimeters.cpp9-55 gathers each region's slices. Gather
borrowed Surface references from every canonical region on the physical layer;
wire the entry view and unchanged containment kernel to that reference view.
Preserve original travel/rectangle assertions; only adjust fixture references.
No polygon union, cloning, new guard or output normalization. This may repair
extra retractions but does not pre-claim closure of speed/M73 or other differences.

Fresh RED51952 `/tmp/ares-region-header-red.log` exits100/4.609s2FAIL298skip;
both real modifier inputs first differ at line18, missing second-region header.
Source PrintApply.cpp1680-1772 groups PrintObjects by shared ModelObject region
owner; its region_set stays empty, so no cross-object config dedup occurs.
Correct the prior analysis's assumed global dedup: resolved all_regions can
provide the consumed ordered configurations, once per shared source object.
Port GCode.cpp2676-2693 into header.rs, using first actual print object's
fallback/support operands, Print.cpp2049-2056's print-wide support enablement,
and role-selected nozzle including inner-wall initial width. Reuse Flow's
selected_nozzle and existing width arithmetic; propagate errors at emission.
Do not fabricate global IDs or cache/clone full configs. Keep body comparisons,
original fixture/spacing assertions and all Tree/Normal gates. Existing full
output public tests are the acceptance tracer; no private seam test is added.
After the first header repair, trace remaining full-output differences rather
than declaring a complete modifier GREEN from a header match alone.

## Public perimeter entry continuation

Independent terminal six-axis review confirms this finite scope has no newly
mandatory static repair or observed single-region regression, exact inventory
change and unchanged behavior assertions. It withholds all-region output and
submission approval. Remaining repair list: global region header/full stream,
repeated owner invocation storage, fuzzy group consumption, density input and
actual regional tool/config consumers; Tree/Normal and full option/printer gates
remain open. Index empty; no dependent WIP submission.

Final frozen gates: workspace54257 exits100/257.474s7513run7512PASS,
1 original Tree header-width failure,9skip35slow. Original spiral10.676s PASS,
unchanged spacing31.562s PASS. Inventory7522 = previous7523 minus the one
standalone obsolete guard test; five embedded branches changed no test count.
Clippy81611 exits0/4m46s325testwarnings308duplicates; fmt/diff checks0.
Tier1 compile39179(WASM core+adapter),92907(macOS core),20635(Windows core)
all exit0; these are not runtime checks.

Seeded52942 exits100/8.702s6run3PASS3FAIL294skip in
`/tmp/ares-perimeter-entry-seeded.log`: Lightning, ironing and timelapse controls
PASS; both public modifiers remain header/output DIVERGENT, later-higher-density
fixture remains sparse_infill_density configuration rejection. Official63404
exits100/1.006s3run2PASS1FAIL297skip in
`/tmp/ares-perimeter-entry-official.log`: ironing/timelapse PASS, Lightning's
original M73 timing difference remains; neither producer nor comparison was
modified. Complete-output parity, global printer/options scope and dependent
Normal/Tree merge remain unachieved. No commit/push of this dependent WIP.

Physical-schedule actual native63683 exits100/4.718s, 2 failures/298 skipped
in `/tmp/ares-physical-schedule-native.log`; artifact root
`/tmp/ares-physical-schedule-native.TxF3Rj`. Both now generate complete G-code.
Bridge-flow:275898 reference/278361 actual bytes; width:308681/309764.
Only final parity assertions fail, no production index panic. First mismatch
is missing second-region header; full diff also contains speed, travel/retraction
and M73 timing differences. None may be filtered from acceptance comparison.

Next output owner: GCode.cpp2676-2693 iterates Print::m_print_regions, with
first-object Flow/support operands. PrintApply.cpp1751-1772 constructs the global
registry from all_regions and assigns global IDs. Rust region_graph/volume_regions
currently own object-local IDs: deduplicating those IDs across objects would
incorrectly discard unrelated regions. Port global configuration ownership
before replacing header's one-model-part-per-object loop; preserve region order,
shared configurations and first-object operands. Boundary is not a header-only
formatting fix; remaining consumed regional motion/config paths require output
validation. Keep this continuation separate from the frozen entry/schedule gate.

Frozen source SHA256: preflight2fe0d4996de03b284d51295d806c538c1d29a7f0fb8ab5e0650ea064c04d64ff;
traversal8958d580c594f10b769eb2f6c232b5d315f711efa7ebc93f100f20538cdf71d6;
layersbfa51b725468d3df7707d955850f0c0ae6947e5d3c25be052a10ddd9e44e7d10;
schedule2fc1d0671abd63e9bbc34ba4f6650508ac7092b549241fac47f78c355d64c8af;
cooling_resulta57e84476d23d57b82fd0995f8c0aa6898b6ff77b1e734fa3dda51d73c6a4911.
Independent six-axis static/actual review accepts only removal of obsolete guard
assertions and physical identity repair; complete regression gates remain pending.

Remove six obsolete assertions of the deleted multi_region_layer_slices guard:
one standalone cache-rejection test and five embedded lifecycle branches.
They asserted a temporary prohibition, not geometry output. Keep every other
capability/geometry precedence branch, transaction test, original spiral and
four spacing assertions. Public actual modifier byte comparisons remain RED
and retain all original assertions. Inventory decreases by exactly one test;
no new rejection expectation substitutes for output acceptance.

Actual entry-native run exits100: 2 failures in4.727s. The line-width modifier
now generates complete output (309764 bytes vs308681 reference), first mismatch
missing second-region width header. Bridge-flow input now reaches emission but
panics in layers/group.rs: index97, physical layer count75. No early rejection
remains in either tracer. Fix the consumed GCode.cpp1718-1784 physical-layer
schedule and process_layer active/cooling identity in layers/schedule.rs,
layers.rs/cooling_result.rs; expose a borrowed physical layer view on traversal.
Do not call either test GREEN or weaken full byte comparison.

Source: Layer.cpp139-291/LayerRegion.cpp82-142; destination
perimeters/preflight.rs. Fresh public RED before changes:
`/tmp/ares-perimeter-entry-red.log`, exit100, 2 failures/298 skipped in4.205s,
both UnsupportedProjectFeature(multi_region_layer_slices); baseline without
modifier succeeds. Existing per-region Flow preparation, compatible groups,
owner generation and fill redistribution are already present but public output
is not accepted. Replace temporary one-region entry with all-region raw option
and physical layer alignment validation, keeping raw-before-derived ordering.
Then run the unchanged public modifier/bridge-flow tests and fix the actual
next source-owned consumer. Do not count guard removal alone as parity.
Repeated compatible invocation semantics remain unverified; defer acceptance,
not execution. No new guards, skipped support, fixture edits or normal/tree merge.

## Spiral-vase review confirmation

The new vertical-shell-cache regression is fixed, not a baseline failure.
PrintObject.cpp::discover_vertical_shells limits its complete processing window
to num_layers; trimming now also requires current.layer_index < num_layers.
This prevents window-excluded layers from turning Internal minus empty holes
into full solid infill. The original spiral Z-ramp test and all four original
spacing assertions remain unchanged.

Fresh confirmation in `/tmp/ares-spiral-review-confirm.log` exits 0:
2 tests pass (spiral 4.540s, spacing 13.824s), 7521 skipped across 21 binaries.
The preceding complete workspace run remains 7513 passed, only the original
Tree header-width failure, 9 skipped; fmt and diff checks pass again.
Independent six-axis review closes this specific regression, not overall
regional parity or the Normal/Tree submission gate. No dependent WIP commit.

## Regional horizontal-shell continuation

Source boundary: PrintObject.cpp3955-4161::discover_horizontal_shells, including
region-major/layer-minor traversal, extra_solid_infills physical i pattern,
EnsureAll skip, source-local region configuration, serial same-region neighbors,
neighbor External Flow and source Solid Flow. Rust destinations:
horizontal_shell_{promotion,propagation}/stage.rs; existing propagation/window/
geometry implementation already consumes canonical source/neighbor inputs.
Replace the two temporary single-region validators with canonical regional count
and slot identity plus physical plan/layer/region identities; do not just remove
assertions or change the correct region-scoped neighbor algorithm. Promotion and
propagation keep staged/dirty commit, scratch isolation and failure disposal.
Include only this remaining entry alignment; defer public perimeter entry,
compatible execution, bridge stage Flow ownership and regional full-output
closure, plus full Tree/Normal completion. No guard deletion, fallback, new
helpers or private tests at the unconfirmed injection seam. Existing public
modifier E2E remains the agreed tracer; rerun before edits and retain its genuine
early rejection, not regional GREEN. Verify horizontal behavior plus unchanged
spiral/spacing, full workspace, fmt/Clippy and independent six-axis review.

## Regional vertical-shell downstream consumers

Horizontal terminal regression evidence: full2621 exits100/284.967s,
7514run7513PASS1originalTreeheaderFAIL9skip41slow. Focused3763 exits0/126.086s,
72PASS13slow7451skip. Both use --workspace,21 binaries and inventory7523;
source two-stage freeze hashes unchanged. No assertion or fixture modification.
Actual repaired native38705 exits100/4.225s2FAIL298skip, fresh
`/tmp/ares-horizontal-shell-native.SAKp34`; both still reject early at
multi_region_layer_slices, not proof of later regional propagation.
WASM17621/macOS75709 terminal0, Windows99852 remains pending; compilation
only. Final independent six-axis evidence review remains required.

Windows99852 terminal0; all three current Tier1 compile checks pass, not runtime.
Independent terminal review has no new mandatory or regression finding for
the two-validator finite scope and confirms inventory correspondence. Existing
spiral/spacing and transactional coverage retained. Real differing regional
configuration/serial propagation and complete public output remain unverified;
the entire staging model is not accepted as equivalent on all inputs. No
dependent WIP/Normal/Tree submission or global completion approval.

Horizontal quality83082 terminal0/5m25s325testwarnings308duplicate, no new
stage diagnostics; fmt/diff61695 terminal0. Workspace2621 original spiral
PASS10.534s, unchanged four-assertion spacing PASS42.096s. Focused3763 includes
72 tests over the same workspace graph; spiral21.125s/spacing50.675s PASS,
terminal summary pending. Repaired public native38705 in
`/tmp/ares-horizontal-shell-native.log` and Tier1 compile17621(WASM),
75709(macOS),99852(Windows) are pending. No new branch or output acceptance.

Next-consumer source audit resolves the front Flow suspicion without code edit:
PrintObject.cpp2512 explicitly reads layer.regions().front()->flow(frSolidInfill)
for bridge gather/filter spacing. bridge_over_infill/stage.rs's front canonical
row and empty-front Flow resolution match this ownership; do not average it.
Generator.cpp70 also owns first shared-region configuration. This differs from
SupportParameters.hpp's explicitly averaged bridge_flow. Remaining prepare_infill
production front references are the source spiral first-region bottom-shell cap,
bridge gather spacing and Lightning generator configuration, not a blanket
regions[0] defect. Compatible perimeter invocation/public preflight remains the
next real regional entry boundary after this round's gates.

Horizontal round fresh public RED10094 terminal100/4.209s2FAIL298skip in
`/tmp/ares-horizontal-shell-red.log`, before code changes, remains inherited
multi_region_layer_slices rejection. Two validators now use canonical regional
count and identities; geometry, Flow and tests unchanged. Source files182/223LOC.
Freeze promotion/stage.rs SHA256
dce6083f7f9bae59fae57197aaba8a202db6ffbf5f0118089defe86d18d704b9;
propagation/stage.rs
62a7aad51a8e0303d400736197d03a0f7c07330cdc5b57cd23acd768df1df18a.
Fmt/diff61695 terminal0; workspace2621 in
`/tmp/ares-horizontal-shell-workspace.log`, focused3763 in
`/tmp/ares-horizontal-shell-focused.log`, Clippy83082 in
`/tmp/ares-horizontal-shell-clippy.log` pending. Focused uses --workspace graph.
Independent static six-axis review has no new validator mandatory finding;
retains true later-region/configuration/serial-behavior and public-output gates.
It does not approve the entire pre-existing promotion→propagation staging model
as equivalent on all inputs. No WIP/Normal/Tree/global submission approval.

Next necessary portion of the agreed public modifier-output tracer rewrites
PrintObject.cpp2314-2440::discover_vertical_shells in
prepare_infill/vertical_shell_{trimming,regularization,filtering,assignment}/stage.rs.
Native layerm surfaces and Flow are region-local, while tiny-shell visibility
uses physical idx_layer±1 over global m_layers lslices, not regional storage
slots. Replace inherited one-region/flattened-layer alignment with canonical
region/layer slot identity and physical plan cardinality. Change filtering's
actual neighbor operands to input.current.layer_index; retain the spiral
execution window, geometry kernels, staged commit and error disposal order.
No shared geometry copies, new helpers, guard deletion or fallback. Existing
public complete-output modifier tests remain RED at the earlier perimeter
entry, so this portion cannot claim its regional GREEN alone. Supplemental
private shell test seam remains unconfirmed; do not invent such tests. Existing
shell behavior and original spiral/spacing tests are regression gates.
Deferred: horizontal shell propagation, compatible perimeter execution/public
entry closure, full multi-region output proof and Tree/Normal completion.
Run activated public RED before code edits, then focused/full fmt/Clippy and
independent six-axis review. Documentation remains with dependent code WIP.

Fresh public RED68149 terminal100/4.182s2run2FAIL298skip in
`/tmp/ares-shell-downstream-red.log`: inherited multi_region_layer_slices entry
still rejects modifier and bridge-flow modifier; no new branch proof is implied.
Four stage implementation is now connected with canonical count/identity and
physical filtering neighbors. No tests, fixture assertions or geometry kernels
changed. Fmt/diff96129 terminal0. Workspace73464 in
`/tmp/ares-shell-downstream-workspace.log`, focused53057 in
`/tmp/ares-shell-downstream-focused.log`, Clippy29810 in
`/tmp/ares-shell-downstream-clippy.log` are pending. Focused uses workspace
feature graph to reuse the same built binaries rather than separate core graph.
All four stage source files remain below400LOC (102/98/125/168).
Independent static six-axis review finds no new deterministic mandatory issue,
confirms actual physical neighbors and retained spiral window/Noop/disposal;
requires terminal regression evidence and eventual genuine regional output.
No current WIP/Normal/Tree submission approval.

Remaining next-consumer audit: horizontal_shell_promotion/stage.rs137-169 and
horizontal_shell_propagation/stage.rs176-210 still require single-region count
and physical==slot. propagation/propagate.rs already uses source planned layer
window and same-region canonical neighbor slots, so do not replace that actual
regional algorithm with flattened neighbors. Bridge stage still has front-region
Flow operands which require their own upstream ownership review. No edits to
these next consumers while this round's frozen regression runs are pending.

## Physical vertical-shell cache and neighbor implementation

Downstream round terminal evidence: workspace73464 exits100/312.682s,
7514run7513PASS1originalTreeheaderFAIL9skip53slow; original spiral PASS10.782s,
unchanged four-assertion spacing PASS66.724s. Focused53057 exits0/159.966s,
195PASS38slow7328skip; spiral23.963s and spacing61.473s PASS.
Clippy29810 exits0/5m27s325testwarnings308duplicate; no stage diagnostics.
Fmt/diff96129 exit0. Concurrent test-run durations are observation, not a
performance claim. Actual repaired public modifier9050 and Tier1 compile
checks47657/86765/57163 remain pending; no new coverage or parity approval.
Final six-axis reviewer will inspect these terminal gates, not the old snapshot.

Evidence scope clarification requested by reviewer: this focused command is
`cargo nextest run --workspace --no-fail-fast -E 'test(vertical_shell) |
test(active_project_spiral_vase_ramps_z_across_body_extrusions)'`, not the prior
`-p ares-core` graph. Log53057 starts195 tests across21 binaries, skips7328;
full73464 starts7514 across the same21 binaries, skips9. Both inventory totals
are7523. Prior core-only focused18006 used6 binaries, total7096; the427-test
difference is workspace scope, not removed tests. Four stage hashes are unchanged
from pre-run freeze to terminal check. Current workspace core test binary
ares_core-b38f5b5ecb6b4218 SHA256
e0c8d4a8a09a24956321281bf90e6e4a51cb91af12d4f1a85d13335d59f71cfd.
Do not treat previous core-only run as this round's source proof.
Native9050 terminal100/4.176s2run2FAIL298skip, fresh
`/tmp/ares-shell-downstream-native.HRo1RI`: both retain inherited
multi_region_layer_slices rejection. All three current Tier1 compile checks
47657(WASM)/86765(macOS)/57163(Windows) terminal0; compile-only, not runtime
or public multi-region acceptance. No comparison changes.

Fresh workspace nextest inventory confirms7523/21: core six binaries7096,
CLI408, vgcode16, WASM3. Independent final review accepts this scope explanation
and closes the inventory evidence question; no new mandatory or regression is
found for the four-stage repair. Canonical alignment/physical neighbors and
original spiral/spacing behavior may be recorded only within this finite
verified scope. Real regional outputs, runtime platform parity and Normal/Tree
completion remain open; dependent WIP submission is not approved.

Existing public complete-output support/modifier and modifier bridge-flow tests
remain the agreed tracer. Fresh initialized native run25199 in
`/tmp/ares-vertical-shell-public-red.log` exits100:4.226s2run2FAIL298skip,
before any shell code edit. Both fail at the inherited multi-region perimeter
entry rejection; this RED does not exercise shell behavior. Supplemental regional
shell injection preference remains unanswered, so no new private test is written.
Continue the necessary implementation portion of the public regional-output
slice without claiming its GREEN before the entire path is connected.

Port PrintObject.cpp2008-2151 cache and physical index ownership into vertical_shells
and vertical_shell_projection. Keep canonical regional presence records; own
multi-region physical geometry once as shared top/bottom/hole entries or a
regional-top/bottom bank with shared physical holes. Projection borrows these
views, visits physical neighbors and never clones a whole object/cache matrix.
Use canonical regional Flow/configuration even for empty regions in the shared
branch, including their source simulated wall offset. Retain the source spiral
num_layers limit. Existing single-region cache/projection tests and unchanged
four-assertion Flow test provide regression coverage, not regional proof.
Downstream shell stages, compatible execution, public preflight closure and
Tree remain required. No guard removal, fallback or comparator change here.

Implementation now registers physical_cache.rs (under400LOC) and a source-state
cache bank. Regional presence slots remain aligned; shared geometry is owned
once, regional top/bottom avoids duplicated global holes. Projection borrows
views, scopes regional rows and passes actual physical layer index to gather.
Spiral num_layers limits construction/visits. Shared branch resolves source
Flow even for empty regions rather than omitting their configured wall offset.
Original spacing test file remains SHA
c334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551;
no new private test, no assertion or fixture weakening. Existing projection
unit-input builder only supplies absent physical bank/full cache length.

Initial Clippy42082 exits0/1m18s but finds three new nesting diagnostics in
physical holes and stage's nested fill selection;328test311duplicate. Repair
uses borrowed region→fill→contour/holes iterator, and guarded pattern matching
before the local cache branch. No suppression or behavior change. Quality
Clippy98668 exits0/2m35s,325test308duplicate in
`/tmp/ares-vertical-shell-cache-quality-clippy.log`; the three new diagnostics
are gone. Initial focused57958 exits0/25.647s64run64PASS7032skip in
`/tmp/ares-vertical-shell-cache-focused.log`; it predates the quality repair
and is not final-source evidence.
Native ClipperUtils.hpp default is Miter/3; cpp578-580 offset2 is exactly
expolygons_offset then offset_paths, matching the new shared holes sequence.

Independent six-axis review requires role-specific Flow evaluation: native
PrintObject.cpp resolves Solid first, then External and Internal only when
perimeters>0, never Overhang. The full perimeter resolver incorrectly evaluated
bridge configuration for this cache stage. Minimally expose its existing role,
borrowed context and role resolver inside project_slice and use exactly that
native ordering; add no fallback or duplicate Flow implementation. Runs62845
and18342 predate this mandatory repair and cannot prove the repaired source.
Public multi-region acceptance and Tree/Normal submission gates remain open.

Pre-Flow-repair workspace62845 exits100/278.233s7514run7512PASS2FAIL9skip:
unchanged Tree header rejection and a new existing public spiral-vase regression.
Projection stops at native num_layers, but trimming still processes every layer:
empty out-of-window holes incorrectly turn all Internal into solid. Native
PrintObject.cpp's entire discover_vertical_shells body is bounded by num_layers,
not just cache gathering. Propagate the existing cache's physical layer limit to
the trimming active predicate; its empty trim already produces downstream no-op
regularization/filter/assignment. Preserve the spiral test and every assertion.
Focused18342 exits0 (64 tests); this cannot overrule the public regression.

Original spiral test after window repair21619 passes10.348s; final-source
focused18006 also passes the same unchanged test10.429s and original four-assertion
spacing test31.963s. Whole195-test focused18006 exits0/102.486s195PASS13slow6901skip.
Clippy11799 exits0/7m08s325testwarnings308duplicate; no new diagnostics in
physical_cache or trimming. Fmt16222 and diff-check exit0. Full-workspace91797
remains pending. The spiral failure is a new regression from this
cache slice, never an accepted known failure. Reviewer confirms static empty
trim→empty regularization/filter→Noop assignment and role-specific Flow fixes;
actual final gates are still required. No commit until regression verification.

Frozen repair SHA256: perimeters/flow.rs
44dee55d722db4b917ca1361dde346844f9fd8de80df6c42db5b6dbbade3219d;
physical_cache.rs e304b21b669bca0333744bc0a1db8e3949f291a36280912da8ae233b5549eb4f;
vertical_shell_trimming/stage.rs
fbd58002c9e9f65d509705567c821a48e009050997eb28491f5239676485190f.
Activated seeded77393 and official50776 use separate fresh artifact directories
and unchanged producer/comparison policies. Tier1 compile checks67029(WASM),
46699(macOS),71430(Windows) remain compile-only, not native/browser execution.

Activated seeded77393 terminal100/18.953s6run3PASS3FAIL294skip,
fresh `/tmp/ares-vertical-shell-cap-seeded.uv53HV`: complete ordered Lightning,
ironing and traditional timelapse pass; two support modifier cases fail inherited
multi_region_layer_slices and compatible density fails sparse_infill_density.
Official50776 terminal100/2.606s3run2PASS1FAIL297skip,
fresh `/tmp/ares-vertical-shell-cap-official.QHEvQq`: ironing/timelapse pass;
Lightning differs at byte1808,line83,expected G1 versus actual M73. Comparison
unchanged, no progress filtering and no new cause attribution. Full91797
terminal100/265.135s7514run7513PASS1FAIL9skip36slow: only the unchanged
Tree header-width support_type/support_style rejection fails; original spiral
test PASS10.681s, unchanged spacing PASS in the same run. The newly introduced
spiral regression is fixed, not reclassified as baseline. WASM67029,
macOS46699 and Windows71430 all terminal0; compile-only. Independent final
six-axis review closes both mandatory fixes with no new deterministic finding:
original spiral regression restored and four spacing assertions preserved.
Shared/regional output proof, public parity and Normal/Tree completion remain
open; reviewer does not approve current WIP submission or global completion.

## Lightning regional expansion and immutable originals

Rewrite PrintObject.cpp2593-2723's two-phase LIGHTNING SECTION in
candidate_expansion.rs::populate_lightning_expanded_surfaces and
lightning_expansion.rs. Phase1 reads ORIGINAL all-region lower surfaces with
each region's own pattern (not object has_lightning alone). Phase2 visits current
regions in source order, breaks at first empty Internal region, filters candidates
by their owning region, offsets/intersects each candidate separately, unions
solid, subtracts sparse and preserves passthrough metadata/order. Expanded
storage is keyed by canonical region/layer slot and consumed by generator for
all regions. Keep horizontal originals borrowed/unmodified instead of temporary
whole-layer mutation plus backup/restore; candidate source indices stay valid.
TDD seam remains agreed region injection to actual generated polylines, with
public complete ordered-G-code gates retained. First RED moves identical lower
Lightning geometry to a later region with empty lower front and fixed current
configuration/geometry/candidate; generator lines must match and be nonempty.
Then cover own-pattern gating, per-region candidate ownership, first-empty break
and hole-safe expansion. Verify focused/full, fmt/Clippy, activated native controls
and Tier1 compilation, then independent six-axis review. Regional/Tree public
acceptance remains required; no guards/fallback/reference build-policy changes.

First RED4460 `/tmp/ares-lightning-regional-expansion-red.log` terminal100:
15.017s1run1FAIL7092skip (build2m39s), actual ordered polylines differ when only
lower original Lightning geometry moves from front to later region. No fixture,
compile or slot failure. Phase1 only now reads all canonical lower slots and own
patterns; phase2/storage/helper deliberately unchanged until next behavioral RED.
GREEN is running in `/tmp/ares-lightning-regional-expansion-phase1-green.log`.
Fmt36719 terminal0. No new broad parity/submission claim.

Phase1 GREEN58491 terminal0:14.741s1run1PASS7092skip. Next tracer keeps
front-region sparse donut and later-region solid bridge partition fixed, compares
two valid nonempty later-owned candidate extents inside that solid region.
Front-region Generator lines must be invariant; old all-candidate flattening
incorrectly expands front sparse near the foreign bridge. This consumes actual
polylines, not expanded-map structure. Current-later region lacks Internal and
must hit the upstream break after front region. RED pending in
`/tmp/ares-lightning-regional-foreign-red.log` before phase2 changes.

Foreign-candidate RED26920 terminal100:14.462s1FAIL7093skip, actual generated
lines change as later-owned candidate grows into front sparse donut; no compile
or fixture failure. Minimal ownership repair filters the current front region's
candidate list only. Full regional map/loop and per-candidate ExPolygon geometry
still pending a later-current-region tracer; do not call phase2 complete yet.

Ownership GREEN10132 terminal0:14.439s2run2PASS7092skip (build2m42s),
including the lower all-region gate regression. The next behavioral tracer keeps
both current regions sparse, moves front geometry outside the later-region output
mask, and varies only the later-owned bridge candidate extent. Actual later-region
Generator lines must change and stay nonempty. Existing original surfaces are
untouched; test geometry partitions sparse donut and central solid in the later
region, with the candidate inside that solid. Initial run66470 exits101 from a
test-only Point field access error; use x()/y() and retain that compile log, not a
behavioral RED. Corrected run78940 uses
`/tmp/ares-lightning-regional-own-red-fixed.log`; await terminal before porting
the current-region map/consumer and per-candidate ExPolygon intersection.

Corrected own-region RED78940 terminal100:14.940s1run0PASS1FAIL7094skip.
Both generated outputs are nonempty but equal; the required extent-sensitive
later-region behavior is absent. Port now loops canonical current region/layer
slots, stops at the first missing/empty Internal region, filters own candidates,
and lets Generator consume each matching slot. `expand_layer` is replaced by
`expand_region` (no alias); borrowed per-candidate polygon slices are offset and
intersected separately into ExPolygons, preserving holes before solid union.
Original horizontal geometry and candidate identities are not mutated. Focused
GREEN31392 in `/tmp/ares-lightning-regional-own-green.log` includes all candidate
expansion tests plus the unchanged four-assertion spacing regression.

Independent six-axis spacing review confirms the per-region
`prelude.object.records[*].ext_perimeter_flow.spacing` injection, not detached
Classic `prelude.records`, and all four original behavior assertions. Immutable
spacing file SHA256 c334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551;
prior full-workspace PASS31.901s is in
`/tmp/ares-lightning-outlines-final-workspace.log`. This is fixture behavior
coverage, not all-region/scale or complete public parity. No new mandatory
finding; Tree/Normal/public modifier and submission gates stay open.

GREEN31392 terminal0:14.582s10run10PASS7085skip. Later-owned extent test
PASS14.571s; unchanged spacing regression PASS13.656s. Independent six-axis
review finds no deterministic algorithm deviation but requires first-empty
break coverage. Added `empty_front_internal_stops_later_region_bridge_expansion`
using the same actual-polylines fixture: front current record has no Internal,
later current has sparse donut plus solid candidate, candidate narrow/wide
must produce equal nonempty output. This supplements the preserved upstream
break contract; it is not claimed as a new historical RED. Test child now SHA
fa5845e35c2f9ade42de3d43a2b83a0bb9db52466dfbe50e00e9549aad2ce32d.
Final focused84064 `/tmp/ares-lightning-regional-final-focused.log` pending;
production hashes unchanged. Full77346, Clippy71277, seeded55497 and official67275
are running/queued with fresh separate reference artifact roots. Tier1 compile
checks70541/80422/47666 are queued; compilation is not platform runtime acceptance.

Observed terminal results for frozen production: workspace77346 exit100,
271.157s7514run7513PASS1originalKSRTreeFAIL9skip37slow. Own-later extent
PASS33.584s, first-empty break PASS33.989s, unchanged spacing PASS32.895s.
Failure remains `support_type/support_style` in the unchanged header-width test;
no fixture/assert relaxation. Clippy71277 exit0/3m33s325testwarnings308duplicate,
only existing seven-argument warning in the candidate file; no new nesting
warning. Fmt/diff2006 exit0. WASM70541/mac80422/win47666 exit0,
7m20/7m51/8m09 including build-lock waits, compilation-only.

Actual initialized seeded reference55497 exit100:18.090s6run3PASS3FAIL294skip,
fresh root `/tmp/ares-lightning-regional-seeded.IrD2cO`, seed0 CPU0 and all
six selectors activated. Complete Lightning, ironing, timelapse controls PASS;
support modifier and bridge-flow modifier fail `multi_region_layer_slices`,
compatible later-higher-density fails `sparse_infill_density`. Actual official
reference67275 exit100:2.939s3run2PASS1LightningFAIL297skip, fresh root
`/tmp/ares-lightning-regional-official.4il5lm`. Lightning difference remains
byte1808 line83, expected G1 and actual M73; do not mask M73 or infer a root
cause from this run. Approved wrappers/native binaries/build policy unchanged;
manifest unknown producer fields are retained, not relabeled. No public
regional/Tree/global acceptance or dependency-WIP submission is authorized.
Final focused84064 still awaiting terminal and includes all four new tracers.

Final focused84064 exits0:15.051s11run11PASS7085skip, first-empty PASS14.858s,
spacing PASS13.968s. Six-axis reviewer verifies finite semantics and the same
open public/Tree/submit gates. Final Clippy67470 exits0/1m51s but introduces
one test-only `enum_variant_names` warning (326test308duplicate): all variants
share Sparse postfix. Rename the enum to CurrentSparseRegions with Front/Both/
Later variants; geometry, candidate inputs and assertions are unchanged. Retain
the warning log; quality reruns75523/67404 are pending in
`/tmp/ares-lightning-regional-quality-{clippy,focused}.log`. All production hashes
and spacing SHA remain unchanged. No broad parity or submission claim.

Quality focused67404 terminal0:15.010s11run11PASS7085skip; first-empty
PASS14.998s and own-later extent PASS14.927s. Renamed test SHA256
298c937732237c848c73a8db6769a585de9d88780f2ba5e76f94575044719ef5,172LOC.
Quality Clippy75523 terminal0:3m55s325testwarnings308duplicate; the new enum
postfix warning is gone, existing seven-argument warning remains. Production
hashes unchanged, no fallback/suppression, no new Tree/public acceptance.

Next physical-neighbor prerequisite identified from authoritative source:
vertical_shell_projection/stage.rs passes a flattened regional slot as the
physical index to gather::project_record and asserts one region. Upstream
PrintObject.cpp2008-2446 iterates region_id separately from idx_layer. For
interface_shells=true, cache top/bottom is region-local but holes are gathered
over all regions; for false with multiple printing regions, the shared-cache
branch2024-2106 merges all top/bottom with per-region Flow, simulated perimeter
offset/min spacing and global lslices. Do not merely relax alignment asserts
or remove public preflight guards: physical traversal/cache ownership must be
ported before the public modifier path can be accepted. The request to allow
a supplemental actual-shell regional injection test is pending; existing public
3MF complete-output seam stays the default agreed acceptance boundary.

Final independent six-axis quality re-review confirms the renamed SHA,
11/11 focused PASS and closed enum warning. No new mandatory defect in this
finite regional expansion. It explicitly does not approve dependency WIP
submission, Normal/Tree, public modifier or global completion. Next shell-cache
work is not covered by this approval. No code/fixture/comparator guard changes
have been made for that next slice; source-boundary prerequisites are recorded
in spec and ARD before implementation.

## Lightning all-region outline collection

Source Generator.cpp151-171::generateInitialInternalOverhangs and
180-193::generateTrees; destination candidate_expansion.rs::build_lightning_generator.
Use the existing agreed regional-injection/generated-polyline seam, retaining
public complete ordered-G-code acceptance. First RED: unchanged geometry moves
to a later region, global first configuration fixed; actual polylines must be
identical and nonempty. Then collect every physical layer's regions through
canonical record addressing, with no regional pattern/density gate. Preserve
front expanded replacement only for front region until the separate regional
PrintObject.cpp2593-2723 expansion port. Verify focused/full nextest, fmt/Clippy,
activated public controls and independent six-axis review. Regional dependency
closure/Tree gate still prevents broad WIP submission. Existing ARD-0091
configuration repair is already committed5c7ebd22, not part of a new delta.

RED55862 `/tmp/ares-lightning-all-region-red.log` terminal100:14.626s
1run0PASS1FAIL7091skip. Real generated-tree equality fails: later-region lines
are empty while the front-region baseline is nonempty. No compile/slot panic.
Port now iterates every canonical region/layer record, appending Internal/Void
contours/holes without intermediate per-region polygon vectors, preserving
source order. Only front region consumes the existing front-expanded map.
Focused GREEN31254 `/tmp/ares-lightning-all-region-green.log` is live; no parity
or submission acceptance yet. Rustfmt41660 is also pending terminal observation.

GREEN31254 instead exits101 at compile time (tuple arity and slice inference),
not a behavioral result. Correct tuple rest pattern and explicit borrowed slice
type; rerun in `/tmp/ares-lightning-all-region-green-fixed.log`. Rustfmt41660
exits0; post-fix formatting80251 also exits0. Retain the failed compile log.

Frozen production SHA256
d40ac50b7d3d481917c7af48015184ebe3ee65baab5b6516ae6ca9002c60356b
and test SHA256
e6fc319eebe1d97bd170a3cdddeee20acbf74507bae94d59893e608ebb7c5c05;
313/97LOC. Focused corrected20836 is still compiling. Final validation launched
with same dedicated WIP target, incremental0, no source edits thereafter:
workspace24505, clippy66654, WASM88294, macOS27684, Windows75882,
seeded3613 and official67526. Logs are `/tmp/ares-lightning-outlines-` plus
workspace/clippy/wasm/macos/windows/seeded/official `.log`.
Native controls use empty fresh roots `/tmp/ares-lightning-outlines-seeded.Q8Cupq`
and `/tmp/ares-lightning-outlines-official.ZXF585`, unchanged wrappers/builds;
seeded launch explicitly seed0/CPU0. Relevant flags are enabled; public modifier
failures remain acceptance counters, not skips. All terminal observations and
independent review remain pending; no submission approval.

Corrected focused20836 terminal0:14.495s3run3PASS7089skip; width14.483,
zero7.168, moved-geometry14.327. Compilation2m42s. Primary seeded selector
does not match the compatible-region function (its name is not modifier);
run that exact activated test separately in
`/tmp/ares-lightning-outlines-compatible-regions.log`, preserving the original
running command rather than describing four selected tests as five.

Static independent review confirms source-order/all-region operands and finite
scope, but requires checking new nesting. Clippy66654 terminal0/2m05s emits
a new excessive_nesting warning at candidate_expansion.rs286; it is a mandatory
quality fix despite command success. Keep this frozen run's remaining processes
to terminal before editing, then filter surface kinds in the iterator and
revalidate a new frozen source. WASM88294/macOS27684/Windows75882 terminal0;
fmt32590 terminal0. These are the pre-quality-fix source only, not final evidence.
Multiple nonempty regions, holes/Void and front-expanded plus later geometry
remain additional coverage; full regional expansion/public parity is unaccepted.

All pre-quality-fix processes are terminal. Workspace24505 exits100:261.679s
7510run7509PASS1originalTreeFAIL9skip36slow. Original KSR header test fails the
unchanged support_type/support_style guard; fixture/assertion unchanged.
Spacing four-assertion test passes31.678s, KSR initialized complete output
passes40.534s. Seeded3613 exits100:19.104s6run4PASS2FAIL294skip; one PASS is the
unactivated inherited-filament test (41ms early return), not actual reference
coverage. Actual relevant controls are3PASS2modifierFAIL; both failures are
existing multi_region_layer_slices. Compatible77790 exits100:18.601s1FAIL299skip,
existing sparse_infill_density. Official67526 exits100:2.832s2PASS1LightningFAIL
297skip, first M73 difference byte1808/line83. These counters remain failures.
Remove new nesting with the equivalent surface-kind iterator filter; update
the generator doc to specify all-region outlines and front-only expanded map.
Refreeze and rerun all verification; do not carry old-source checks forward.

Quality-fixed production SHA256
4c6b9254bf0dfd5a7d777d90ff8fdc80570c43cc35ba35464bf595d9d00c5baa,
unchanged test e6fc319eebe1d97bd170a3cdddeee20acbf74507bae94d59893e608ebb7c5c05.
Formatting8750 terminal0; check54265 pending. Final dedicated-target runs:
workspace66256/focused66358/clippy9882/wasm16450/macOS45559/Windows23872/
seeded30979/official41698, logs `/tmp/ares-lightning-outlines-final-` plus
workspace/focused/clippy/wasm/macos/windows/seeded/official `.log`.
Fresh native roots `/tmp/ares-lightning-outlines-final-seeded.kRPTNG` and
`/tmp/ares-lightning-outlines-final-official.wn1EPg`; seed0/CPU0 explicit.
Final seeded selector now names compatible-region exactly and excludes the
unactivated inherited-filament early-return test. Final six-axis review waits
for these actual terminal results. All remain pending; no submission.

Final fmt54265 exits0 and diff check0. Final clippy9882 exits0/2m06s,
326 core test warnings309duplicates; new nesting warning removed, unchanged
7-argument signature warning retained. Final WASM16450/macOS45559/Windows23872
exit0:32.52s/48.46s/16.50s, compile only. Freeze remains unchanged; spacing
test SHA c334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551,
original KSR header fixture/assertions unchanged and git index empty.

Final focused66358 terminal0:14.377s3run3PASS7089skip, zero7.168s,
geometry-move14.224s, width14.365s (build4m45s including locks). Remaining
final full66256/seeded30979/official41698 are confirmed live. Keep polling
these handles; no restart or early all-green/submission claim.

Final seeded30979 terminal100:18.144s6run3PASS3FAIL294skip, all selected
reference flags active. Lightning/ironing/timelapse complete outputs pass;
modifier bridge-flow and modifier support fail multi_region_layer_slices,
compatible-region fails sparse_infill_density. Final official41698 terminal100:
2.634s3run2PASS1LightningFAIL297skip; M73 first difference byte1808/line83.
No comparator relaxation, reference/source/wrapper changes or ignored failure.
Final workspace66256 still live; source/test hashes unchanged. Final review
will retain these incomplete gates, not substitute finite geometry acceptance.

Final workspace66256 terminal100:266.606s7510run7509PASS1originalTreeFAIL
9skip36slow. New moved-geometry33.107s, width33.904s, zero16.804s all pass;
spacing31.901s and KSR initialized complete output41.278s pass. Tree header
still rejects support_type/support_style; no test fixture/assertion changed.
Every final verification process is terminal, no interruption/restart used.
Final frozen hashes unchanged and index empty; request independent six-axis
复验 of this finite consumer only, retaining regional expansion, actual public
modifier failures, Tree and global completion as open gates. No WIP submission.

Independent final six-axis复验 verifies frozen hashes/terminal logs, confirms
source-order/all-region geometry collection and closure of the new nesting
finding, and accepts only the finite outlines consumer and proven behaviors.
No new static mandatory finding; no dependency-WIP commit approval. Required
remaining evidence includes simultaneous nonempty regions, holes/Void and
front-expanded plus later geometry, followed by actual regional expansion and
public modifier parity. Official M73 residual is not an established root cause;
it remains RED. Normal/Tree, platform runtime and full printer/options goal
remain incomplete; keep goal active and continue the source-owned ports.

## Lightning generator configuration ownership

Source PrintObject.cpp1010-1020 and Fill/Lightning/Generator.cpp66-100;
destination candidate_expansion.rs::build_lightning_generator. Inspection
finds current first-Lightning parameter selection differs from native global
first-region ownership. First add generated-polyline invariance for an empty
later Lightning region's changed sparse width while front configuration and
all geometry remain fixed. Use real KSR stage input with a simple three-layer
internal square and no bridge candidates to isolate generator configuration,
not private field/structure pins. RED before port; preserve public fixtures,
comparators and spacing four assertions. Full nextest/public/fmt/Clippy and
independent six-axis review required. All-region outlines/expansion follow;
no normal/tree or dependency-WIP submission approval from this finite slice.

Independent source review confirms native activation/config-owner split.
Current single-range graph preserves all_regions order into PostRegion;
multiple range/shared-object ordering is not proven and remains required.
RED89452 /tmp/ares-lightning-owner-red.log terminal100/14.442s: changing
empty later Lightning width changes generated polylines. Port owner to
regions[0] without changing the existing pattern-only activation yet.
Focused GREEN40288 pending /tmp/ares-lightning-owner-green.log. Positive-density
activation is the next vertical TDD cycle, not claimed from this width test.

Owner GREEN40288 terminal0/14.744s, actual ordered lines invariant and
nonempty. Only after this GREEN, add zero-density activation test with the
same generated-line seam; positive case assertions unchanged. Later Lightning
density0 must not construct a generator while front CrossHatch remains15.
RED pending in /tmp/ares-lightning-activation-red.log. The bridge candidate's
has_lightning_infill flag remains true, matching the native pattern scan;
prepare_lightning_infill_data must independently apply positive density.

Activation RED47114 terminal100/7.487s: zero-density later Lightning region
still constructs Some generator. Add native positive-density condition only
to generator activation; bridge pattern-only flag unchanged. Final frozen
candidate_expansion.rs310LOC SHAf3a077d4272eba85bb50d7f1fa323039084e0e6cdac351b9f7dd9f19caa4ef83;
lightning.rs78LOC SHA8f3e42375711c9fb931b7272fa98cb7a0c41e25d332e5f5e3fa9c018167aebf0.
Fmt/diff0. Full workspace/public/Clippy/Tier1 target compile checks now pending
in /tmp/ares-lightning-owner-{workspace,clippy,public,wasm,macos,windows}.log,
three public flags enabled and original comparator unchanged. Actual terminal
results and independent six-axis re-review required; no commit/push or broad
Lightning, modifier, Normal/Tree or global acceptance.

Final workspace96573 terminal100/261.332s,7509run7508PASS/one original
Tree-widthFAIL/9skip36slow. Width invariance PASS33.339s, zero-density
activation PASS17.486s, initialized KSR complete ordered comparison41.179s.
Clippy56539 terminal0/5m30s,lib311/test326warnings309duplicates, no added
warning. Public92323 terminal100/1.454s5run3PASS2oldguardFAIL295skip, all three
flags enabled; cached producer unknown, not fresh official evidence.
WASM6471/macOS18801/Windows25324 terminal0/7m26s/7m41s/7m56s including locks,
compile-only. Fmt/diff0, frozen hashes unchanged and no final interruption.
Independent terminal six-axis review required. HEAD45cfb1d4/index empty;
Normal/Tree and regional dependency WIP remain unsubmitted.

Read-only HEAD audit finds the owner/activation production hunk itself does
not require inherited regional addressing. Current supplemental test placement
does require WIP fixture visibility, so it cannot be directly submitted. A
candidate standalone seam is the existing project-level bridge transaction
test module: consume transaction::prepare's lightning_generators and use the
existing ancestor fixture without widening visibility. Independent reviewer
requires actual clean-HEAD REDs, minimal production diff, valid canonical slot
invariants, full/public validation and final review before declaring an
independent consumed submission closure. This is not submission approval.

Independent final six-axis review verifies main-WIP terminal logs/hashes,
finds no new mandatory finding or regression evidence, and accepts only the
two generator configuration contracts. Whole regional geometry/expansion,
multi-range mapping, public modifier output, Normal/Tree and global acceptance
remain required. No main-WIP submission approval. Clean candidate now exists
at /tmp/ares-lightning-owner-clean.BmhCOL, detached45cfb1d4, dedicated target
/tmp/ares-lightning-owner-clean-target. Its independent owner/activation source
hunk, existing transaction test seam and three ARD-0091 docs are isolated from
regional WIP. Clean RED41395 and owner-only99103 are terminal; final workspace
7473PASS/9skip, Clippy/fmt and Tier1 compilation pass. Corrected seed0/CPU0
controls pass3/3; official Lightning remains RED while two other controls pass.
Baseline and candidate actual CLI outputs match on the exact retained official
failure input, and match direct initialized native output. Official same-input
repeat itself differs. All detailed evidence lives in the candidate's
2026-10-02-lightning-generator-configuration-owner plan; final independent
submission review accepted the exact six-file closure; committed/pushed as
5c7ebd2212c6db215360610095b0efa2794d96c7 (ARD-0091). No widened visibility
or regional/Normal/Tree WIP entered that commit; no broad parity completion
claim. This dirty worktree deliberately remains detached45cfb1d4, preserving
all inherited files; it has not been destructively synchronized to the new
remote tip. Future isolation must start from5c7ebd22 and not recommit this
already-consumed owner/activation delta.

## Physical-layer sparse anchoring call boundary

Source PrintObject.cpp2736-2756 and Fill/Fill.cpp1394-1512; Rust destination
candidate_expansion/layers.rs::prepare_infill_lines. First test unchanged
candidate expansion when identical lower-layer geometry/config moves from
front to later region, preserving physical layer identity. Run RED before
removing front-presence/density gating. Validate each present regional input
using own nominal Flow, generate grouped anchors once per physical lower layer.
Existing complete-output public modifier gates remain enabled and unchanged;
the supplemental seam cannot substitute for their acceptance. Run focused/full
nextest, fmt/Clippy and independent six-axis review; keep WIP unsubmitted until
dependency closure and Tree/Normal gates pass.

Initial test build15105 exits101 because snapshot declared f32 rather than
the actual f64 angle; no behavioral RED claim. Corrected RED3728 in
/tmp/ares-bridge-anchors-red-verified.log exits100/14.474s: identical lower
geometry/config moved to region1 changes angle3.138879799 instead of baseline
1.610272749 and changes ordered polygons. Independent review confirms source
requires physical-layer generation, and zero-density validation must remain
skipped only for its own region. Implement canonical per-region validation,
then one generation call outside the loop. Use exact Polygon equality for
the test snapshot, preserving f64 angle and ordered coordinates.
Frozen layers.rs309LOC SHA9708e9f12f366b4c03127b28344d7616db537343f4080e14a092a0a724ecbebd;
anchoring.rs56LOC SHAac82a3f05c3b4d56a17e509a3fa3edafea909ddd6f12facdf418a7e613a46f0d.
Independent static re-review finds no deterministic mandatory logic issue;
nesting quality awaits actual Clippy, different regional configs and mixed
zero/nonzero density await behavioral coverage. Final runs pending:
/tmp/ares-bridge-anchors-{green,workspace,clippy,public,wasm,macos,windows}.log
with sessions50345/82613/12489/8143/17257/55727/71578. All three public flags
enabled, unchanged cached reference scope. No WIP submission or global claim.

Focused GREEN50345 terminal0/14.454s,3/3PASS: exact expansion invariance
14.317s, prior regional angle7.206s, density15/10014.442s. Subsequent Clippy
actually detects new excessive nesting at layers.rs78. Refreeze by flattening
the aligned-slot match into a borrowed tuple, then the same zero-density
continue and own-Flow validation. No allow or assertion change. Cancel only
validated process groups belonging to this slice. Workspace82613 terminal100
65.231s5939/7507run5874PASS65FAIL(64SIGINT plus original Tree),1568notrun;
Clippy12489/public8143/WASM17257/macOS55727/Windows71578 terminal130. Those
intentional interrupted runs are not final acceptance or regression proof.
Final layers.rs307LOC SHAb68750491aa4975ef967a92483244387dcc7c907d5ddf75006ae4b6a7eade660;
anchoring test hash unchanged. Restart for this actual quality fix, not an
observation timeout. Final logs /tmp/ares-bridge-anchors-{workspace,clippy,
public,wasm,macos,windows}-final.log; sessions85717/59642/68967/64551/33017/37615
remain pending. Earlier GREEN proves only the pre-refreeze production version;
final full run must reverify behavior.

Final workspace85717 terminal100/255.578s,7507run7506PASS/one original
Tree-widthFAIL/9skip35slow. Exact regional anchoring invariance PASS32.556s;
own density15/100PASS33.697s; original four spacing assertions PASS31.419s;
initialized KSR ordered complete-output comparison PASS41.973s. Clippy59642
terminal0/5m20s,lib311/test326warnings309duplicates, baseline restored and
added nesting closed. Public68967 terminal100/1.504s,5run3PASS2oldguardFAIL
295skip, three flags enabled; cached producer unknown, not fresh official
output proof. WASM64551/macOS33017/Windows37615 terminal0/7m13s/7m28s/7m43s
including build locks, compile-only. No final timeout or interruption. Fmt and
diff0, frozen source/test hashes unchanged, header fixture/test untouched,
HEAD45cfb1d4/index empty. Independent terminal six-axis review still required;
no dependency-WIP submission or complete multi-region/Normal/Tree/global claim.

Final independent six-axis terminal re-review verifies frozen hashes, logs and
plan, finds no new mandatory issue or regression evidence, and accepts only
this call-boundary repair and demonstrated ordered expansion invariance.
Different regional configurations, mixed zero/nonzero density, pattern-specific
consumption and activated public modifier output remain required. No standalone
submission closure or Normal/Tree/Tier1-runtime/all-printer-option/global
approval. Next bridge consumer boundary is PrintObject.cpp2593-2723's regional
lightning expansion and generator construction; preserve original surfaces
and actual source region ownership, not an object-level pattern shortcut.

## Physical-layer deep sparse operands

Source PrintObject.cpp2819-2846; destinations deep_sparse_area and expansion
object construction. Supplemental behavior test moves lower0..14 KSR
canonical inputs/geometry to region1 but keeps layer15 candidate/current
region0. Expansion must not lose that candidate because lower front records
are empty. RED first, then borrowed physical regional views and source-order
classification with own density. Existing mathematical assertions remain
unchanged; only constructors adapt. Run workspace/public/Clippy/fmt/platform
checks and independent six-axis review. Lower sparse anchors and lightning
remain required, not falsely claimed by this geometry injection; unchanged
complete-output gates and Normal hold until Tree remain in force.

Initial lower-region geometry RED18155 exits100/7.402s. Independent review
requests own density evidence; strengthen before production by contrasting
region1 density15/100, with InternalVoid retagged Internal in both injected
cases (same sparse classification at15, excluded at100). Original standalone
InternalVoid assertions remain untouched. Rerun RED on the stronger behavior;
this does not claim cross-region sparse/non-sparse overlap or anchoring parity.

Strengthened RED98554 exits100/7.215s at lower region15 candidate absence.
Production deep_sparse_area SHA62a2533055ec2cdb160757b731d464c7370145812c1d722cf39b0219335db9c6;
expansion2c4d2b811ae550b25c04d7f0c50cdbc768e7350f6423f26a0481f2abcb5c379f;
testf9e2a5c25ece518f45615c8e2cde4297516b133c45ca5c7d5de69ed5fd594d5f.
First workspace56514/Clippy69568 exit101 because the separate real-KSR deep
fixture still used old fields. Migrate its constructor and input snapshot,
preserving every original assertion; no compatibility fields or skipped test.
Static review verifies singleton snapshot byte order unchanged, flags added
loop nesting. Second Clippy actually confirms that warning; replace hole loop
with ordered for_each, no allow. Final real-KSR testfile SHA
c874dc7b690f53963fedb6f866d52b29b10445afa0c22b699f7b71c57f975e3b.
Second workspace90655 intentionally SIGINTs for quality refreeze, terminal100:
127.761s6150run6085PASS65FAIL(64SIGINT plus original Tree),1356notrun. Second
Clippy73830 terminal130. Neither is final acceptance/timeout/regression proof.
Final workspace11654/Clippy71335 pending in /tmp/ares-bridge-deep-{workspace,
clippy}-final.log. Production hashes unchanged, public48561 terminal100/1.476s
5run3PASS2oldguardFAIL295skip, three flags enabled/cached producer unknown.
WASM75721/macOS55807/Windows77160 terminal0/2m19s/2m34s/2m49s including locks,
compile-only. Only cfg(test) adapters changed since those production checks;
no rerun necessary, but all final tests and quality gates remain pending.

Final workspace11654 terminal100/255.043s,7506run/7505PASS/one original
Tree-width FAIL/9skip/34slow. Strong density15/100 behavior PASS33.466s;
original real-KSR deep repeatability/input preservation PASS14.600s;
spacing four assertions PASS31.871s; initialized KSR ordered complete output
comparison PASS40.522s. Final Clippy71335 terminal0/5m26s,311lib and326test
warnings309duplicates, baseline restored and added snapshot warning closed.
Fmt/diff0, all frozen hashes unchanged, original header fixture/test untouched.
No final-run interruption or timeout. Public/platform checks above retain only
their proven production scope. HEAD45cfb1d4/index empty, no commit/push or
reference/guard/comparator changes. Independent final six-axis review required;
cross-region sparse/non-sparse overlap, sparse anchoring and lightning remain
unverified required consumers, not modifier/Normal/Tree/global completion.
Next source boundary: PrintObject.cpp2736-2756 and Fill/Fill.cpp1394's
Layer::generate_sparse_infill_polylines_for_anchoring; Rust destinations
candidate_expansion/layers.rs::prepare_infill_lines and sparse_anchoring/.

Final independent six-axis terminal review verifies frozen hashes and logs;
no new mandatory finding or regression evidence. Accept only the physical
lower-region operand repair and demonstrated own-density behavior. Original
spacing assertions and real-KSR deep invariants remain intact. Cross-region
sparse/non-sparse subtraction still lacks direct behavioral proof; sparse
anchoring, lightning and activated public multi-region output remain required.
No dependency-WIP submission approval, Normal/Tree parity, Tier1 runtime,
all-printer/option or global completion. Keep this batch uncommitted.

## Regional bridge expansion operands

Source PrintObject.cpp3154-3214 and3242-3265; destination candidate_expansion/layers.rs.
Supplemental behavior seam: keep real KSR lower geometry, move physical
layer15 input/surfaces/candidates to a second region with explicit internal
bridge angle90, leaving the front input absent. Expansion must retain those
candidates with own region identity and angle. Run RED before changing
production; then use canonical region/layer addressing for first and each
candidate and borrow all physical current-region operands. Public modifier
complete-output gates stay unchanged and enabled. Run workspace nextest,
fmt/Clippy and cross-target checks; independent six-axis review required.
Deferred lower/deep/sparse/lightning consumers still prevent modifier parity;
no Normal submission until Tree parity, no private source-structure pins.

RED /tmp/ares-bridge-expansion-red.log20190 exits100/7.438s: old first-input
lookup panics at inputs[15] when the real candidate belongs to region1.
Production layers.rs SHA9635b23ff50967e0fa621ea0b853b749e26bbb26412b084c8cac0cc8375570ab;
test4f138f1304b484af2338e923ebee35585bd17ac656cec17c561257e03d4e63ed.
Independent static six-axis re-review finds no added mandatory issue; current
regions retain source order and their own borrowed operands. Coverage only
proves later-region consumption/configuration once GREEN is terminal; several
nonempty current regions and distinct per-candidate Flow remain behaviorally
unverified. These are required future output checks, not claimed successes.
Previous surface rewrite and spacing-test frozen hashes remain unchanged.
All final workspace/public/Clippy/platform runs are pending; no submission.

Final /tmp/ares-bridge-expansion-{workspace,clippy,public,wasm,macos,windows}.log:
workspace76733 exits100/257.840s,7505run/7504PASS/one original Tree-width
FAIL/9skip/36slow. New later-region expansion behavior PASS17.532s; earlier
regional surface rewrite PASS17.279s; original four spacing assertions
PASS32.031s; initialized KSR ordered complete comparison PASS40.553s.
Clippy45278 exits0/4m48s,core lib311/test326warnings309duplicates, same
baseline and no new slice warnings. Public58088 exits100/1.459s,5run/3PASS/
2oldguardsFAIL/295skip, all three flags activated. Cached-reference producer
unknown; no fresh native or raw-byte claim. WASM96587 exits0/6m44s,
macOS68816 exits0/6m59s,Windows8190 exits0/7m14s including locks, compile-only.
Fmt/diff0; source/test/previous frozen SHAs unchanged. HEAD45cfb1d4/index empty.
HEAD has RegionLayerIndex but lacks region_layer_record_index; this consumer
still depends on the inherited canonical regional WIP, not an isolated
submission closure. No Normal/Tree batch committed; no public multi-region or
global acceptance. Independent terminal six-axis re-review remains required.

Final independent six-axis terminal re-review verifies hashes/logs and finds
no newly mandatory defect or regression evidence. Accept only the finite
operand repair and proven later-region consumption/configuration. Several
nonempty contexts/distinct candidate Flows, deep/lower/sparse/lightning,
activated public multi-region output and Tree remain required. No independent
submission closure or Tier1 runtime/all-printer-option/global completion.

## Physical-layer bridge surface rewrite

Source PrintObject.cpp3315-3391; destination transaction/surface_rewrite.rs.
First add a supplemental two-region rewrite behavior test: an upper candidate
in the later region with different solid spacing must ensure both lower
regions, while creating a bridge only in its original region. Run RED before
the consumer port, then move shared operands outside the regional loop and
resolve each upper candidate's canonical slot. Keep all existing behavior
assertions, public fixtures and complete-output comparators. Run focused and
workspace nextest, fmt/Clippy, enabled public output gates, cross-target
checks and independent six-axis review. Regional lightning/expansion remains
deferred; no Normal submission before Tree parity.

Initial test-only fixture visibility builds79420/88004 exit101; these are
not behavioral RED evidence. Reuse the existing KSR bridge fixture with
test-only project_slice visibility; no additional production test entry point.
Final behavioral RED /tmp/ares-bridge-rewrite-behavior-red-final.log37644
exits100/7.373s: ensuring area7840784000000 from the wrong front-region0.2
spacing, rather than the own-region1.0 spacing. Activated public RED9282
exits100/1.633s at the old multi_region_layer_slices guard; control PASS and
cached producer unknown, not cross-region branch proof.
Independent static review requests shared-cut non-owner geometry and bridge
angle assertions; add both before freezing. Production SHA
6826b3c061e37ba16760cb3463f1995c45d0064f5a033a63f1db50692ecb61f1;
behavior test9c604b9f68287819c9e057d2f5e0b9108c74a18357b235b1c2e9caf9dc355e09.
Static six-axis re-review finds no new mandatory defect. The existing spacing
behavior file staysc334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551
with all four assertions. Full validation is still pending terminal results;
no commit/push or complete-output acceptance claim.

Final /tmp/ares-bridge-rewrite-{workspace,clippy,public,wasm,macos,windows}.log:
workspace52871 exits100/259.904s,7504run/7503PASS/one original Tree-width
FAIL/9skip/36slow. New regional rewrite behavior PASS16.720s; preserved
current-versus-stopped spacing PASS32.512s; initialized KSR complete ordered
comparison PASS42.679s. Clippy62876 exits0/5m04s,311lib and326test warnings
(309duplicates), unchanged baseline and no new warnings in this slice.
Public20601 exits100/1.486s,5run/3PASS/2oldguardFAIL/295skip: ironing and
traditional timelapse plus landed support PASS; density sparse_infill_density
and bridge-flow multi_region_layer_slices FAIL. Cached reference producer
unknown; no fresh native or raw-byte claim. WASM10685 exits0/6m58s,
macOS5820 exits0/7m13s,Windows29526 exits0/7m27s including build locks;
these remain compile-only checks. Fmt/diff checks0, frozen SHAs unchanged,
index empty and HEAD45cfb1d4; no reference build policy/comparator/guard/header
fixture changes. Independent terminal six-axis re-review required before
finite acceptance; not public modifier, Normal/Tree or global completion.

Final independent six-axis re-review confirms all frozen SHAs and terminal
logs, no new mandatory finding or regression evidence. Finite surface rewrite
and preserved behavior accepted only; no dependency-WIP submission approval,
public multi-region/Normal/Tree/Tier1 runtime/all-printer-option/global claim.
Continue source-owned regional lightning/expansion and unchanged public output
gates; keep Normal batch unsubmitted until Tree parity completes.

## Physical-layer bridge candidate gathering

Public enabled RED /tmp/ares-bridge-regions-public-red.log exits100/1.573s
at original multi_region_layer_slices, with control comparison PASS. Source
PrintObject.cpp2493-2589, LayerRegion.cpp21-28, PrintRegion.cpp25-53; destination
bridge_over_infill stage/candidates. First write source scope in spec/ARD,
then replace single-region CandidateLayer operands with physical-layer borrowed
regions. Resolve empty front region's Flow from its own configuration. Preserve
all old geometry assertions while migrating constructor packaging, no new
private pins or alternate public fixture/comparator. Run workspace nextest,
fmt/Clippy, public enabled controls/tracers and cross-target checks, freeze
and independent six-axis review. Later bridge consumers remain to port; no
new guard/silent skip, no reference scheduling/build change, no modifier parity
claim from this earlier guard RED, no Normal submission before Tree parity.

First focused /tmp/ares-bridge-regions-focused.log exits100/27.138s,
242run/241PASS/one object_scope injected-shape failure/6843skip. Its injection
created3regions but only1region canonical record slots. Preserve all four
assertions and supply3*n empty canonical slots, restoring original vector
lengths before original records. Do not delete this behavioral coverage.
First Clippy62047 terminal0/2m31s adds excessive_nesting327/310; flatten the
current-region/surface loop with original region-major order and surface
indices. No algorithm/assertion change. Workspace67328/79701,Clippy89751,
public20939 and cross-target32509/90120/17178 were explicitly stopped130
to validate corrected frozen source, not timeouts or acceptance evidence.
Frozen stage7c601a924619ec39f6a3cb298f68e2f302b65f9a2bed17e0f6b4373f22d81bd7;
candidates45ad9576a25bdcdccdfc08105e36cfc5073b97415e2036e1ab761c5313c185ec;
object_scope d687da044fb0ceb484016f3c3a22940e8780648ef5131edc9e5df9ada0582ae7.
17geometry test files change only imports/physical-region wrapping; all
original assertions remain. The separate spacing behavior file remains
c334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551.
Final runs use /tmp/ares-bridge-regions-{workspace,clippy,public,wasm,macos,
windows}-verified.log; independent six-axis static and terminal re-review
required. Until terminal results, no acceptance/commit/push claim.

Frozen static re-review verifies all three SHAs, original region/surface order
and surface indices before filtering. object_scope resize/truncate preserves
all four assertions; no additional deterministic mandatory defect.
Final workspace47972 terminal100/256.823s:7503run/7502PASS/one original
Tree-width FAIL/9skip/36slow. object_scope behavior PASS15.096s, preserved
current-versus-stopped spacing PASS31.618s, initialized KSR complete ordered
comparison PASS40.892s. No failed-exec or interrupt in this final run.
Final Clippy25891 exits0/5m24s,326core-test warnings/309duplicates; added nesting
warning removed. Final public70480 exits100/1.474s,5run/3PASS/2 old guard
FAIL/295skip (CLI inventory300): ironing/timelapse/landed-support PASS;
density sparse_infill_density and bridge-flow multi_region_layer_slices FAIL.
Cached reference producer remains unknown, not a fresh official/replay result.
WASM90198 exits0/7m17s,macOS14067 exits0/7m32s,Windows92283 exits0/7m47s,
including locks. These checks remain compile-only evidence. Fmt check0;
hashes/spacing assertions/header fixture unchanged. Final independent review requested,
index empty, HEAD45cfb1d4, no commit/push or gate removal. Different front Flow
with a nonempty later region, cross-region lower solids and downstream
physical-layer consumers still need actual public output acceptance.

Independent six-axis terminal review verifies frozen source/test hashes and
all final logs. No new mandatory static finding or regression evidence;
quality repair closed, behavioral assertions preserved. Only finite candidate
source port and existing behavior preservation are accepted, not WIP submission
or public cross-region/Normal/Tree/Tier1-runtime/all-printer-option/global
completion. Continue actual regional lightning/expansion/rewrite consumers
and unchanged public complete-output gates; keep Normal held until Tree parity.

## Canonical shell alignment continuation

Source: PrintObject.cpp discover_vertical_shells2334-2435 and
discover_horizontal_shells3955-4161, Layer.cpp139-291 repeated compatible
owners. Destination: five shell stage validators. Independent six-axis design
review confirms scratch elements are only count/presence placeholders, not
operands. Remove those checks and only matching mixed-test subcases; preserve
outer object alignment, canonical/cache/plan/scale/identity and all geometry/
transaction behavior, including the four task22o20 spacing assertions.
Do not change guards, fixtures, comparator, native build or scheduling.
The first public invocation lacked ARES_PARITY_SUPPORT_MODIFIER_REFERENCE=1
and returned early (0.008s); explicitly exclude it from acceptance evidence.
Enabled public tracer /tmp/ares-shell-alignment-public-active-red.log exits100
before production edits; earlier multi-region guard is not branch coverage.
Run full workspace nextest, fmt/Clippy and independent frozen diff review.
No Normal submission before Tree; invocation/front-region Flow/physical
neighbor/shared-cache port remains required after this prerequisite.

Frozen static six-axis re-review finds no new mandatory defect: five validators
remove only scratch inner-record checks; six mixed-test files retain all test
functions and unrelated assertions. No operand/transaction/error behavior
changes. Changed production files are99/121/167/181/222LOC. Hashes:
trimming46f855b12a5eaa3089c026b7f444caa07be2ecebc1b38e1c4a7eb731ec1014bb;
filteringaca87f67e0cd33844c412f2f985c2f7b4160a80d0c886c8728d5454676d2c4e0;
assignmented7a47a5c5550e1c0b80b3842bc864b0e020d924b692a84dab74d34dce6f4409;
promotion28cb6e1ba9df919774ca413de95384c0551d97decb28f27f9edaf026b15da086;
propagatione8fccc21ca8316ecd16b76ec9bedd30a58c77e07f60e6710d5f27c728256e85d.
Spacing test remains c334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551.
Clippy /tmp/ares-shell-alignment-clippy.log exits0/5m03s,326core-test
warnings/309duplicates, no new warnings in edited stages. Fmt check0.
Workspace /tmp/ares-shell-alignment-workspace.log terminal100/254.326s,7503run:
7502PASS/one original Tree-width FAIL/9skip/34slow. Preserved spacing behavior
PASS31.626s; initialized KSR complete ordered comparison PASS40.861s.
Public enabled RED artifacts case-R1Oq6r control PASS under generated-by-only
ordered comparison, case-XX27xs original multi_region_layer_slices failure.
Both manifests retain runner_cache/producer unknown; no fresh native attribution
or raw byte equality claim. Final enabled public log
/tmp/ares-shell-alignment-public-final.log exits100/1.488s:5run/3PASS/2FAIL/
295skip (actual CLI inventory300). Ironing/timelapse and landed support control
PASS; density modifier sparse_infill_density and bridge-flow modifier
multi_region_layer_slices retain original guard failures. No new policy or
fixture change. Cross-target logs /tmp/ares-shell-alignment-{wasm,macos,windows}.log
exit0/5m34s,5m49s,6m04s including build-lock waits; these are compile checks,
not platform runtime parity. Fmt/diff check0, frozen hashes unchanged, original
KSR header fixture/assertions untouched. Independent terminal re-review
requested; index remains empty and no commit/push or gate removal.

Final independent six-axis review verifies frozen scope and terminal evidence,
no new mandatory repair or regression evidence. It approves only the limited
scratch decoupling, not submission of this WIP or modifier/Tree/Normal/global
acceptance. Keep cached producer unknown, compile-only platform evidence and
the original complete-output failure gates. Next actual source port must
remove bridge candidate's scratch dependency with physical front-region Flow
(including empty front regions) and all-region lower geometry, not substitute
each candidate's own spacing or silently remove the guard.

## Shell Flow consumer continuation

Port the four production shell operands cited in the spec and ARD. Remove
generator prelude count/presence coupling, not canonical identity checks;
delete obsolete prelude-only subcases in mixed private tests. Retain unrelated
canonical/cache/plan assertions and public output assertions, without new pins.
Retain the original public modifier fixtures and complete comparator. Verify
full nextest, public controls/REDs, fmt/Clippy, Tier1 compile boundaries and
independent six-axis review. Native shared/all-region shell cache and physical
neighbor rewrite remain required adjacent slices; this step cannot submit
the held Normal/Tree batch or prove modifier complete output by itself.

Public RED /tmp/ares-shell-flow-public-red.log terminal100/1.603s:
2run/0PASS/2 same original guards FAIL/298skip. Independent review agrees with
the four exact regional Flow operands but finds whole-test deletion too broad:
the mixed tests also cover still-retained canonical/cache/plan constraints.
Restore those cases unchanged and remove only five obsolete prelude count/
presence subcases; rename projection's mixed test to canonical input count.
No public fixture/assertion/guard/comparator or native-policy change. Initial
validation sessions66708/35967/73134 deliberately interrupted terminal130 for
this coverage-scope repair, not accepted as results. Freeze the corrected
snapshot for full/public/Clippy final runs; Normal/Tree/global gates remain open.

Coverage repair independently re-reviewed: the five obsolete prelude subcases
alone are removed, unrelated assertions remain. Four stage SHA256s:
vertical_shells21df937ec297115939a8a729a9372266f0f8ed25ded5ec0bcb38560be3f68b36;
projection2c32cd1291a6d8b2948fddfd9e2feb56169102e00a027a7a2cb112a00fcd4230;
regularizationb97d9f487deb3978d2f8c686449a94591e750df29a00d5d135b2acdea628a1d6;
filtering37fad43cb73e6f3b6ea4984917edf74431130b9580b712cba28d1966e5384bd5.
Stages61/102/97/134LOC; tests173/191/136LOC. Fmt/diff terminal0. Final Clippy
/tmp/ares-shell-flow-clippy-final.log/session69080 terminal0/5m23s (includes
build-lock waiting);326core-test warnings/309duplicates unchanged, none in the
four stage files. Workspace/public/platform terminal results still pending.

First corrected-scope full run session79523 terminal100/256.772s:
7503run/7501PASS/2FAIL/9skip/36slow. Tree-width failure unchanged; the other
failure is the current-versus-stopped-neighbor spacing test, whose injection
still mutates the now-unused Classic prelude. User review requires retaining
that behavior test, not deleting it. Restore its four assertions unchanged;
inject distinct current/neighbor spacing into canonical regional external
Flow, converting scaled test distances to float millimeters with the selected
coordinate scale. This supplementary seam is explicitly allowed by the user;
public complete-output acceptance remains mandatory. An attempted deletion
was reverted, and validation sessions49488/30273 interrupted terminal130 for
the restoration, not accepted as results. Focused behavioral GREEN pending.

Restored focused test /tmp/ares-shell-flow-spacing-green.log/session85217
terminal0/14.171s:1run/1PASS/7084skip. All four original assertions remain
unchanged. Helper now injects per-region external Flow, not obsolete scratch;
the float-mm injection is not claimed bit-identical to the old integer scratch
injection. Independent review confirms the user-authorized behavior coverage
is preserved. Test SHA256
c334e92099b11ee1ea9031f71329f7df7cb86a9df5bc346e4c8e568b3267c551,
217LOC. Final restored workspace/Clippy/public terminal validation remains
pending; no submission, modifier/Tree or global completion claim.

Restored snapshot Clippy session87743 terminal0/6m47s (includes build-lock
waiting),326core-test warnings/309duplicates unchanged, none in four stages
or restored test. Public session86721 terminal100/5.585s:5run/3PASS/2 original
modifier guards FAIL/295skip; references remain retained runner-cache/native
producer unknown. Earlier WASM21310/macOS4603/Windows77585 checks terminal0
on identical production SHAs; only host-test injection changed since them.
These remain compile evidence, not platform runtime parity. The restored
behavior test also PASS32.320s inside the final workspace run; initialized
KSR ordered bytes PASS41.960s. Full workspace terminal still pending.

Final restored workspace /tmp/ares-shell-flow-workspace-restored.log/session88758
terminal100:build4m06s/run256.001s,7503run/7502PASS/one unchanged original
Tree-width FAIL/9skip/36slow. Restored four-assertion spacing test PASS; no new
remaining regression evidence. Final set-e fmt/diff checks terminal0; four
production SHAs and restored test SHA unchanged. Index empty, HEAD45cfb1d4;
no new commit/push. Independent six-axis terminal re-review requested; true
modifier/Tree output, held Normal merge and global completion remain open.

Final independent six-axis review confirms restored four behavioral assertions
and exact five-subcase deletion scope, source-owned scalar Flow migration,
terminal results and no newly identified mandatory issue. Its approval is
limited to this repair; it explicitly withholds WIP submission and Normal,
Tree or global completion because actual multi-region public output remains
unverified. Continue remaining consumers/invocations/shared caches and the
unchanged real complete-output gates; do not remove the restored coverage.

## Regional Flow consumer prerequisite

Before separating generator invocation records, port the two production
Flow consumers cited in the spec/ARD to canonical regional inputs. Keep the
existing public fixture/complete comparator and gates unchanged; do not add
private helper tests as replacement acceptance. Current public RED
/tmp/ares-region-flow-public-red.log exits100/1.476s:2run/0PASS/2FAIL/298skip.
Density modifier fails sparse_infill_density; bridge-flow modifier fails
multi_region_layer_slices. These are existing public acceptance failures,
not evidence that the new regional consumer path has run. Remaining shell
consumers and independent invocation handoff are the next source slices.
Direct inspection of PrintObject.cpp2512 shows candidate gathering uses the
physical layer's front-region solid Flow, not each candidate region's Flow.
Retract that substitution before validation; its real physical-layer rewrite
remains deferred. Initial workspace/Clippy sessions27609/25170 were interrupted
with terminal130 for this source correction, not accepted as final evidence.

Frozen two-consumer snapshot: types.rs SHA256
10a528e08de2f4f0c97f41d8a19b90ab1f9c7be9fe0390b41a4745c21d3e5356;
surface_type_detection/stage.rs
3b42de9621a592607954b35d46539d31afac8cc010302f1385cbe6103252a81f;
external_surfaces/stage.rs
5007e7269cb9c18aba5c34585e592bfe437def131c35b309dc45e2c8f28784f3.
Fmt/diff checks terminal0; touched Rust265/274/89LOC. Clippy-final.log
terminal0/2m46s,326core-test warnings/309duplicates (same prior inventory,
none in these three files). Public-final.log terminal100/2.132s:
5run/3PASS/2 original modifier guard FAIL/295skip, actual300-test inventory.
The two ironing/timelapse and landed-support controls preserve complete
ordered output against retained runner references; their cached native
producer provenance remains unknown, not newly official or initialized.
WASM core+adapter, macOS and Windows core checks terminal0, logs
/tmp/ares-region-flow-{wasm,macos,windows}.log. Cargo elapsed times2m12s,
2m17s,2m22s include shared build-lock waiting; these are compile checks only.

Workspace-final.log/session59104 terminal101 before tests: rust-lld reports
undefined hidden anon LLVM symbols in untouched object_source/edge_grid
incremental objects. Preserve the failed log; no behavioral failure or
successful full run can be inferred. Rebuild the same source snapshot with
CARGO_INCREMENTAL=0 in workspace-rebuilt.log/session88934. No source, fixture,
native binary, cache policy or comparator change; no artifact deletion.
The suspected incremental-cache cause remains an inference until rebuild
verification. Independent review corrected its initial candidate ownership
approval after direct PrintObject.cpp2512 inspection and approves only the
remaining two consumers' static source/ownership/scaling direction. Full
test terminal evidence and independent final re-review remain pending;
submission and modifier/Tree/Normal/global completion remain withheld.

Nonincremental rebuild terminal100 (session88934), source unchanged:
/tmp/ares-region-flow-workspace-rebuilt.log compiles successfully3m11s and
runs7503 tests265.552s:7502PASS/one unchanged original Tree-width FAIL/9skip/
36slow. Initialized KSR complete ordered bytes PASS40.308s. This isolates
the earlier link failure to the discarded incremental build path; it does
not prove a specific cache-corruption root cause or require compiler/source
changes. Public nonincremental re-run session83333 terminal100:
/tmp/ares-region-flow-public-rebuilt.log build2m36s/run4.423s,
5run/3PASS/2 same original modifier guards FAIL/295skip. Both controls and
active landed-support output remain preserved. No modifier-region consumer
or averaged-support Ares GREEN, no fixture/assertion/guard/comparator/native
policy change. Final six-axis terminal re-review requested. Index remains
empty, HEAD45cfb1d4; no new commit/push or completion claim.

Final independent six-axis terminal review accepts only these two consumers'
limited source/ownership/scaling repair; no newly identified mandatory issue
or new regression evidence. It does not approve this WIP for submission or
Normal/Tree/global completion. Required next slices remain remaining shell
Flow consumers, all-regional Flow boundary validation before invocation
storage changes, repeated generator invocations, compatible fuzzy partition,
physical-neighbor and candidate all-region operands, regional emission and
the unchanged public modifier/Tree complete-output acceptance gates.

## Compatible perimeter group production handoff

Port Layer.cpp139-291 and LayerRegion.cpp82-142 into perimeters/groups,
context, generator views and the fill redistribution handoff. Preserve
original region slices for classification; consume merged surfaces only in
Classic/Arachne generation. Keep RNG-sensitive Classic materialization in
physical-layer/group order while restoring output to region-layer slots.
Included: exact compatibility fields, strict maximum-density owner, singleton
borrow, ordered extra-perimeter buckets with safety offset, fill intersections
and first-member counterbore recovery. Deferred: modifier option gate removal,
region-aware islands/seams/tool ordering/emission, multi-region fuzzy clipping,
all-region support acceptance and Tree generation. This is unsubmitted WIP,
not an independent modifier-completion slice.

Public RED /tmp/ares-perimeter-groups-public-red.log exits100/1.440s;
baseline case-Az4ut4 PASS, density modifier case-51XwMp ARES_ERROR at the
existing effective-config sparse_infill_density gate. It does not exercise
multi-region generators. Bridge-flow modifier remains a separate downstream
single-region RED tracer; no fixture/assertion or guard change is authorized
by these REDs. Initial new geometry tests fail to compile (missing producer,
plus corrected test setup); this is compilation RED, not runtime/native
behavioral RED. Native-backed full-output acceptance remains mandatory.

New numeric behavior RED /tmp/ares-compatible-perimeter-numeric-red.log
exits100/0.028s:5run/2PASS/3FAIL. Failures are six-digit width token equality,
signed-zero width identity, and scarf start-height approximate equality.
After source-cited numeric comparison repair, GREEN
/tmp/ares-compatible-perimeter-numeric-green.log exits0/0.030s:5run/5PASS,
7080skip. Float/FloatOrPercent use strict absolute1e-4; Percent's own overload
and gap_infill_speed.value remain exact. No approximate equivalence-class
shortcut or extra done check was added.

Independent six-axis review confirms union recovery and numeric comparison
classification; it identifies repeated-owner invocation storage and compatible
fuzzy configuration partitioning as real prerequisite defects, not permission
to submit. One record_groups slot cannot preserve two native generator calls
with the same owner when approximate compatibility is non-transitive. Separate
invocation records from final region outputs before enabling modifiers. Delete
the old constant compatible_region_ids field and its source-structure-only
assertions/tests as part of that consumer rewrite; actual membership belongs
only to the group. Keep real object/region/physical-layer identity checks.

Pre-numeric-fix public controls
/tmp/ares-compatible-perimeter-public-controls.log exits100/1.457s:
5run/3PASS/2FAIL/295skip, actual300-test CLI inventory. Landed-support and two
infill-first ironing controls PASS under the existing generator-only ordered
comparator; bridge-flow modifier remains multi_region_layer_slices RED and
density modifier remains sparse_infill_density RED. Neither cross-region
generator behavior nor averaged bottom-interface output has an Ares GREEN.

Fresh official and existing initialized binaries directly sliced the same
retained bridge-flow modifier input case-2nRt2Q, SHA256
abaf8c136956fda374f32589bc64a85d9651a33cf4394725f63f4d691cd06c65.
Both commands exit0 and produce24 positive support-interface extrusions at
height0.6 in /tmp/ares-region-bridge-official.rijLE7/plate_1.gcode and
/tmp/ares-region-bridge-initialized.o5x0hY/plate_1.gcode. This independently
confirms native activation of the average bridge-flow witness, not Ares parity.
Official ELF64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60;
initialized ELF8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d.
Actual normalized complete cmp exits1; first differences include M73 placement.
Normalized official7a979f7e87bf886c1e1aedb9e62e0db6673691062a7e750109690a445f6467a0;
initialized312614866f240fc284b38813653f615b24a6a9ee44079ee6911c57d87237d4c3.
No command removal except generated-by, producer relabel, native source/cache
or scheduling policy change. Harness cached references remain producer unknown.

Source is frozen for workspace --no-fail-fast, final public rerun and Clippy.
Fmt/diff check0. Final terminal evidence and independent review remain pending;
Normal-before-Tree and full modifier/platform/global completion gates stay open.

First frozen workspace /tmp/ares-compatible-perimeter-workspace.log exits100,
267.441s:7503run/7502PASS/one unchanged original Tree-width FAIL/9skip/38slow.
Initialized KSR full ordered comparison PASS41.434s. First numeric-fix public
rerun /tmp/ares-compatible-perimeter-public-final.log exits100/1.712s:
5run/3PASS/2 original modifier gate FAIL/295skip. Clippy first run exits0,
328core-test warnings/311duplicates, including two new groups nesting warnings.
Extract the per-physical-layer prepare_layer boundary while retaining one
shared output Vec (no new per-layer allocation), original scan/owner/offset
ordering and no done-filter shortcut. This is the sole final quality repair.
groups.rs110LOC, frozen SHA
94c0587bcc320872da721e99c7da6644329607cb30afbd519f7687c74d95fc5d.
Post-format set-e fmt/diff checks both terminal0; earlier overlapping formatter
and check output is not accepted as the final format check. Re-run workspace
--no-fail-fast, Clippy and the same unmodified public cases on this snapshot;
final terminal records and independent re-review remain pending. No commit.

Final quality snapshot workspace
/tmp/ares-compatible-perimeter-workspace-final.log exits100/263.686s:
7503run/7502PASS/one unchanged original Tree-width FAIL/9skip/36slow.
All5 new group tests PASS. Initialized KSR ordered output PASS41.111s.
Clippy /tmp/ares-compatible-perimeter-clippy-final.log exits0/3m18s;
326core-test warnings/309duplicates, no groups warning, removing the2 newly
introduced nesting warnings. Public
/tmp/ares-compatible-perimeter-public-verified.log exits100/4.717s:
5run/3PASS/2 original modifier gate FAIL/295skip. Both infill-first ironing
controls and landed-support control PASS; modifier branches still do not
reach the production group or averaged-support consumer. No fixture or
complete-output assertion change and no guard removal.

Cross checks terminal0: core+adapter wasm32-unknown-unknown20.03s
(/tmp/ares-compatible-perimeter-wasm.log), core aarch64-apple-darwin19.62s
(/tmp/ares-compatible-perimeter-macos.log), core x86_64-pc-windows-gnu19.50s
(/tmp/ares-compatible-perimeter-windows.log). These are compile checks, not
browser/macOS/Windows runtime parity. Rust touched files remain below400LOC;
region_slices395/context-test384 are the largest relevant files. Index remains
empty. Frozen groups/compatibility SHAs remain94c0587b/58b69bb7 (full hashes
above/in recorded tool evidence). Independent terminal re-review requested;
modifier/Tree/Normal/platform-runtime/global completion and submission remain
withheld. This round is source-owned WIP progress, not finished parity.

Final independent six-axis review confirms the terminal results and reports
no new static mandatory or new regression evidence. It withholds submission
and completion: repeated invocation storage, compatible-member fuzzy regions,
old constant membership/source-structure checks, regional ironing/island/tool/
config emission, actual modifier GREEN, Tree and Normal gates remain required.
Cross checks are compile-only; cached producer provenance remains unknown.
All current runs are terminal; no source edits after final quality freeze,
no commit/push and no goal-complete declaration.

## Single-region emission phase prerequisite

Port `GCode.cpp:5453-5468,6131-6178` into `gcode_emit/motion.rs` before
extending regional emission: separate ordinary infill and ironing; emit
ironing only after infill-first walls. First write a public cube 3MF tracer
with `is_infill_first=1` and ironing enabled, verify native activation and
complete ordered output RED, then change the actual emission consumer.
Preserve non-ironing controls and first-layer wall ordering. Regional
ownership/configuration, compatible groups and Tree remain open; do not
remove the modifier guard or claim Normal acceptance.

Preliminary public RED `/tmp/ares-emission-phases-red.log` exits100;
GREEN `/tmp/ares-emission-phases-green.log` exits0/0.879s. Same retained
input case-3zPgik was sliced directly by official and initialized binaries
in `/tmp/ares-emission-official.TwAu0l` and
`/tmp/ares-emission-initialized.LHJBx2`; both direct commands exit0. Both
native streams and GREEN Ares normalize to
9c277b0aceb8b8aa28f91663e276b618760da94ef8094befefd68b188e88eb59,
with actual complete cmp0 after removing only generated-by lines. This does
not relabel the harness cached reference's unknown producer provenance.
Frozen preliminary workspace `/tmp/ares-emission-phases-workspace.log`
exits100/259.214s:7497run/7496PASS/one original Tree-width FAIL/9skip.
Clippy `/tmp/ares-emission-phases-clippy.log` exits0, inherited326 core-test
warnings/309duplicates. Independent review requires preserving the original
infill existence for timelapse (including ironing-only islands and the
infill-first branch) and saving comparisons before witness assertions.
Those repairs and clean-HEAD independence verification remain pending;
preliminary GREEN does not authorize submission or regional acceptance.

The independent emission consumer slice completed the review/fix loop and
was submitted separately as45cfb1d4 (pushed origin/main). Its source-cited
ARD0090/spec/plan and exactly7-file scope exclude all inherited Normal/Tree
WIP; clean candidate workspace7471/7471PASS and explicit public3/3PASS.
See its own plan for exact source SHAs, terminal logs, platform compile-only
limits and independently identified same-input official/initialized outputs.
Active worktree HEAD advances to45cfb1d4 without overwriting unsubmitted
files or staging the Normal batch. All regional ownership and Tree gates
remain open.

Post-submission active replay `/tmp/ares-emission-active-public-final.log`
exits0/3PASS but incorrectly reuses the clean candidate's283-test binary
(280skipped); it is not active WIP validation. Cargo metadata confirms the
active manifest path, while its additional region/support test modules are
absent from that reused binary. Explicitly clean only ares-core/ares-cli Rust
package build artifacts before switching worktree validation; no source or
reference artifacts are deleted. Removed30,537files/48.9GiB of rebuildable
package cache. Fresh `/tmp/ares-emission-active-public-rebuilt.log` exits0,
build1m15s from the actual active path;300tests inventory,3run3PASS/0.980s
and297skipped, retaining all additional region/support modules. Do not count
the stale replay or infer active workspace GREEN from clean HEAD. This is
only the emission consumer replay, not regional/Normal/Tree acceptance.

## Starting evidence

Base prerequisite commit: `b79e14d11eb829f972eb4a3c6119c3f8ef795c12`.
Normal/Tree work remains uncommitted in the active worktree.
Existing public modifier RED:
`/tmp/ares-multi-region-compensation-modifier.log`, exit100, baseline PASS,
modifier ARES_ERROR at perimeter `multi_region_layer_slices`.
This is retained historical evidence, not a fresh run of this plan.
No fixture/assertion change is authorized.

## Source checks completed

- `Layer.cpp:139-291` and `LayerRegion.cpp:82-142`: compatibility, grouping,
  maximum-density configuration selection, actual output pointers, cumulative
  lower/upper slices, same-region upper input and Classic/Arachne dispatch.
- `Fill/Fill.cpp:1216-1275`: fills use `surface_fill.region_id` for configuration
  and destination. Continue reading the complete grouping/emission branch
  before implementing that handoff.
- Existing `perimeters/{types,context,preflight,layer_region}.rs` and
  `prepare_infill` consumers contain one-region invariants. In particular,
  horizontal propagation indexes physical `plan.layers` by record index,
  extra-bridge uses adjacent vector slots, and bridge-over-infill indexes its
  records by physical layer. A flattened vector alone is not a safe port.

## Implementation sequence — one observable slice at a time

1. Addressing: retain one object owner and dense region/layer metadata,
   distinguish storage slot from physical layer, and bound same-region walks.
   Proposed internal storage is region-major dense slots with empty-layer
   slots retained; this is storage only, not a new slicing algorithm. Verify
   every caller that indexes layers, neighboring records or emission order.
   Do not add unused addressing abstractions as a standalone feature.
   Storage order is independent of execution order: audit Classic materialize,
   Arachne and all shared SlicerContext consumers before changing record loops.
   Determine source-selected traversal before consuming fuzzy RNG; final
   output sorting is not a correction for changed random consumption.
2. Generator inputs: port source compatibility/group preparation and resolve
   each region's flows. Keep merged generator surfaces distinct from original
   region slices. Select max-density owner with stable tie behavior. Do not
   remove the public preflight rejection while consumers remain unsafe.
3. Outputs: generate only once per compatible group, materialize perimeter and
   thin-fill entities into the selected owner, redistribute fill geometry to
   all members, including source counterbore recovery. Cite exact geometry
   functions and surface metadata operations before implementing them.
4. Production continuation: port every affected typing, shell, bridge,
   infill-combination, grouping and G-code consumer's region/layer traversal,
   citing its PrintObject.cpp or Fill.cpp function. Retain physical-layer
   aggregates only where the source uses them. Then remove the obsolete
   perimeter rejection and rerun the unchanged public modifier tracer.
   Include `fill_entities/generate.rs`, `extrusion_islands/`,
   `island_print_order.rs` and `seam_placement/alignment/preparation.rs`.
   Cite `GCode.cpp::GCode::process_layer` for island/region grouping and
   `GCode/SeamPlacer.cpp::SeamPlacer::init` for seam preparation; preserve region
   perimeter counts, physical aggregate slices and collection source order.
5. Validate full output against the same native input; retain all failures and
   classify native provenance/cache uncertainty separately. Run nextest
   workspace, fmt/clippy, Windows/macOS/WASM compile checks and real browser
   byte-output comparison. Seek six-axis independent immutable-patch review.
6. Fix and repeat review until approved. Isolate only independently preserving
   code+documentation slices for commit+push. The complete Normal support batch
   remains gated on actual Tree auto/manual/style output parity.

## Verification gates

Each implementation cycle must record RED before GREEN, exact source function,
unchanged public seam/fixture, commands, terminal outcomes and artifact hashes.
Single-region preservation cannot prove a newly added modifier branch; include
a real multi-region browser/native case once the production path exists.
No selection among native variants, expected-output backfill, silent support
skip or relaxed assertions. There is no region-owned perimeter GREEN,
multi-region runtime acceptance or submission; the addressing preservation
checks below do not substitute for the public modifier gate.

## Independent design review and repair

The six-axis reviewer confirmed actual max-density output ownership and the
merged/original surface distinction, but required three additions before code:
island/seam/fill-generation consumer coverage, stateful RNG execution-order
discipline, and compatible-group public output coverage. These requirements
are now included above and in ARD0089/spec. Review did not approve runtime or
submission. Reverification is required after these design repairs.

Before each corresponding implementation, add one public complete-output
tracer at a time: compatible walls with higher density in a later member,
equal-density stable first member, then empty-region layer slots. Retain the
existing incompatible-width modifier and single-region controls. Expectations
come from complete native output plus an active native branch witness, not
private group IDs; fuzzy coverage must exercise actual random consumption.

Six-axis design recheck closed the three omissions and permits incremental TDD
implementation only. It grants no runtime, commit, Normal/Tree or global
completion approval. New code must still obtain exact-source consumer audit
and its observable RED→GREEN before removing a rejection.

Current Arachne `config.rs` / `traverse.rs` still reject active fuzzy skin.
This existing gap is not a seeded parity success. The RNG audit above does
not itself implement it; if a new public tracer exercises that branch, first
port the corresponding `PerimeterGenerator.cpp` / `FuzzySkin.cpp` behavior
and obtain complete output parity rather than treating rejection as coverage.
Seam preparation's precise callees are
`SeamPlacer.cpp::gather_seam_candidates` and
`calculate_overhangs_and_layer_embedding`, invoked by `SeamPlacer::init`.

## Addressing implementation slice

Source: `LayerRegion.cpp:82-142` region-selected options/flows and physical
layer neighbor inputs; `PrintObject.cpp::discover_horizontal_shells` at
3955-4045 walks physical neighbor layers inside the same region;
`PrintObject.cpp::detect_surfaces_type` at1741 onward collects external bridge
surfaces and rewrites layer i+1 of that same region. Destinations:
`perimeters/{preflight,context,types}.rs`, horizontal-shell propagation and
surface-type extra-bridge. Store dense region-major flow/context records with
physical layer metadata; consume region/layer addressing in those neighbor
walks. Keep the existing public perimeter rejection until all consumers are
ported, including grouping, stateful generation and emission. No compatibility
group generation, modifier usage collection or complete output is claimed by
this addressing work alone.

The first compatible-group public RED exposed an earlier density guard in
`project/effective_config/candidates.rs`. The modifier graph already applies
parent plus volume RegionOptions, but early usage collection still scans model
part candidates rather than all modifier region roles. Do not remove that
usage-affecting rejection without porting usage ownership as well. Add
`PrintApply.cpp::generate_print_object_regions` /
`region_config_from_model_volume` and Print's region extruder collection to
the owning source audit before enabling density in the public path.

## Addressing WIP evidence — not submit eligibility

### Shared region graph implementation WIP

Implement `project/region_graph.rs` and `region_graph/bounds.rs`, with neutral
source-volume records, actual float3D full/range triangle bounds, and one
source-volume-first / range-second configuration registration order. Keep
reverse parent traversal, extended parent boxes and source fallback behavior.
`project_slice/volume_bounds.rs` consumes the same full-bound calculation;
`volume_regions.rs` maps shared records to actual released occurrence IDs and
bound indices. No whole-object clone or early fake occurrence ID is introduced.
The existing slicing adapter's single-candidate restriction remains an open
adjacent multi-range slicing limitation, not a newly accepted parity outcome.

Candidate construction populates `ResolvedProjectObject::all_regions` and
both effective-config usage and slicing extruder collection consume it.
Object-shell passes keep it empty, preserving pre-region normalization order.
Candidate occupancy uses the resolved shrinkage-compensated representative
transform, matching the shared bounds and
`PrintApply.cpp::print_objects_from_model_object` (137-148), passed into
`generate_print_object_regions` at1730. Test constructors carry their supplied
region configurations into both candidate and all-region fields; expectations
and public fixtures are not changed to manufacture successful output.

The all-region role path is not the actual generated ToolOrdering path.
Density and other usage-affecting modifier rejections are not removed while
the complete perimeter/fill/emission path remains incomplete. No header patch
or Tree/support guard substitution is part of this WIP. The earlier judgment
that `project_slice/extruders.rs` lacked raw layer-range selectors relative to
`Print.cpp:451-489` is withdrawn: its actual owner is
`PrintObject.cpp::object_extruders`, as documented in the source-boundary
correction below. Native cache reuse and staged
print_apply diagnostic mesh carriers are not replaced or accepted here.

Shared graph verification terminal outcomes:

- First snapshot `/tmp/ares-shared-region-graph-first.log`: exit0,
  160PASS,18.989s; before final tuple cleanup and shrinkage occupancy
  correction, not final activation acceptance.
- Final workspace/no-fail-fast `/tmp/ares-shared-region-graph-workspace.log`:
  exit100,277.820s,7486run/7485PASS/1FAIL/9skip. Sole failure remains the
  untouched Tree support-header test at `support_type/support_style`.
  Initialized KSR complete ordered-byte comparison PASS41.812s. Env-gated
  tracers do not establish native acceptance in this workspace run.
- Official native two-tracer run `/tmp/ares-shared-region-graph-public.log`:
  exit100,4.569s,2FAIL. Under
  `/tmp/ares-shared-region-graph-public-output`, Ender3 baseline case-gzwXXk
  complete output PASS; modifier case-uCeBYY still hits density rejection.
  Its input SHA256 remains
  `a06ce5221a38679f5fd5a3fdde7531fe1df7d6d0db5a1070be41a072aec1453b`.
  J1 baseline case-W4yJMI retains the header-first divergence; modifier
  case-ALnPi0 retains density ARES_ERROR. Native witnesses and both complete
  comparisons still run. Inputs remain
  `78eca1c5df43561a9bb9da3526040b1a15d12a488d00c5ddf86d5bfbda00629f`
  and `30a532e027f991b2674066d9ff853201b329425106e9f09c141a4bb140cdd0d3`.
- Final workspace/all-targets clippy exit0,5m24s:
  `/tmp/ares-shared-region-graph-clippy-final.log`. Initial snapshot's new
  argument-count warning was corrected with modifier-source tuple
  destructuring; final log has no region_graph diagnostic. Existing warnings
  remain. Fmt/diff checks exit0 and index remains empty.

Six-axis static review found no definite new graph algorithm error but
requires reachable real range-crossing, adjacent/nonintersecting modifier,
chain and no-op fallback evidence plus complete inherited-filament output.
Internal graph preservation examples are supplemental only. Raw selectors,
multi-candidate slicing, ToolOrdering/header and modifier/Tree acceptance
remain open; no new platform/browser validation or commit/push is claimed.

### Next usage tracer and source-owned implementation

Public project input / complete ordered G-code remains the agreed test seam.
Add `compatible_regions/inherited_filament.rs`: actual Snapmaker J1 dual
0.4mm nozzles, manual filament map1,2, two PLA profiles, support/prime tower
off, parent sparse density0 with sparse filament2, modifier changing only
density to50%. Require native positive XY sparse extrusion while T1 is
selected; an unused configured selector or a custom macro is not that witness.
Retain both baseline and modifier complete-output comparisons, even if the
baseline exposes an earlier printer gap. Do not select another successful
printer or weaken assertions to conceal it.

The owning prerequisite is `PrintApply.cpp::update_volume_bboxes` (893-956),
`transformed_its_bbox2d` / `transformed_its_bboxes_in_z_ranges` (548-660),
`generate_print_object_regions` (958-1070), and
`PrintObject.cpp::region_config_from_model_volume` (3664-3716). Bounds are
float3D, referenced triangle vertices, composed transform with XY translation
removed, contour-only nonnegative XY inflation and Z epsilon. Multiple layer
ranges require actual edge/range clipped bounds, not whole-mesh bounds for
every range. Region creation visits source volumes in order, reverse parents,
extended parent boxes and changed-config/fallback semantics. Share that
computation between effective-config usage and slicing rather than assuming
each modifier intersects every parent. Early configuration phases before
candidate creation must retain their existing phase ordering.
`Print.cpp::object_extruders` (451-489) and
`PrintRegion.cpp::collect_object_printing_extruders` (71-109) collect all
created region roles plus raw generic volume/range selectors for print-wide
configuration usage. Per-object slicing parameters instead use
`PrintObject.cpp::object_extruders` (3784-3811), which collects created region
roles and volume selectors but has no raw-range loop. Actual source
bounds, inheritance and normalization precede removal of the density guard.
Painting and native cache/scheduling changes are adjacent deferred behavior,
not permission to skip an included modifier or density activation branch.

### Inherited-filament tracer execution evidence

All attempts use the explicit untouched official appdir above and
`ARES_ORCA_RUNS=1`; there is no reference selection or expected-byte backfill.
Initial attempt `/tmp/ares-region-inherited-filament-red.log` failed input
verification: the legacy `sparse_infill_filament` name is exported by Orca as
the canonical `sparse_infill_filament_id`. Use and verify that canonical key.
The next `/tmp/ares-region-inherited-filament-verified.log` reached native
slice exit251: native `00000.log` reported support-vector count2 versus
colour-vector count1 after identical profiles/colours were exported. Give the
two real PLA slots distinct colours, as existing multi-filament fixtures do;
do not patch or suppress native validation. Both failed attempts remain
retained and are not runtime acceptance.

The colour-corrected run `/tmp/ares-region-inherited-filament-colours.log`
was a valid diagnostic RED (exit100,1.535s). It retained both native outputs
and showed baseline header divergence followed by modifier density rejection.
The final input-strengthened run, with both complete comparisons asserted
together, is `/tmp/ares-region-inherited-filament-final.log`, exit100,1.509s:

- baseline `/tmp/ares-region-inherited-filament-final-output/case-isLop6`:
  input11279bytes SHA256
  `78eca1c5df43561a9bb9da3526040b1a15d12a488d00c5ddf86d5bfbda00629f`;
  native182143bytes SHA256
  `e1512aac1b317dd3d8642259375fbb401b82566b2dd110f04994b53701f78539`.
  DIVERGENT at comparator byte186, header expected `; filament: 1,2`,
  Ares `; filament: 2`. This is the first difference, not proof that all
  remaining baseline differences share its cause.
- modifier `case-AhI26Y` under that same artifact root: input12110bytes SHA256
  `30a532e027f991b2674066d9ff853201b329425106e9f09c141a4bb140cdd0d3`;
  native270591bytes SHA256
  `623b6b683b622ce5e9ca20d82e9dd359d64b5ee65d4732f2bac9ced4270a4ead`.
  ARES_ERROR at the existing early density guard. All input and native active
  sparse-extrusion assertions completed before the paired full-output failure.
- Export and both native slice journals are retained in
  `orca-runner-03ss51/command-{uOO0iP,4sp2UL,Jvqogh}`; native result files show
  success. The cache manifests still mark producer identity unknown; journals
  are supplementary provenance, not a repaired manifest or parity approval.

`GCode.cpp:2623-2632` writes one-based actual slice filament IDs, not their
count. `Print.cpp:2457-2514` obtains this list from per-layer `ToolOrdering` in
sequential mode or `tool_ordering.all_extruders()` in by-layer mode. Existing
Ares header uses configured usage length. Do not fix that symptom by dumping
all configured IDs: retain generated entity region/role/filament ownership and
the source-owned ToolOrdering path. Configuration usage and slice tool usage
are separate upstream concepts and separate prerequisites.

Six-axis independent recheck confirmed the final inputs and RED: independent
native parsing counted baseline0 and modifier975 positive XY sparse moves
under T1; only density is overridden, so inherited activation is witnessed.
No test repair was required. This is not baseline or modifier GREEN and grants
no submission or Normal/Tree completion approval. `cargo clippy --workspace
--all-targets` exit0,6.44s in
`/tmp/ares-region-inherited-filament-clippy.log`; existing warnings remain.
Fmt and diff checks exit0. No production/header/guard edit or commit/push was
made in this tracer cycle. Complete workspace regression from the preceding
addressing cycle remains historical evidence, not a new all-platform run.
Offline harness preservation: `cargo nextest run -p ares-cli --test
orca_parity` exit0,293PASS,3.138s in
`/tmp/ares-region-inherited-filament-harness.log`. The new live tracer is
environment-gated in this offline run; its reported PASS is not native output
acceptance and does not supersede its explicit provisioned RED above.

Public compatible-group tracer is in
`runner/compatible_regions.rs`; it exports Ender3/overhang with support off,
base sparse density0 and a later modifier changing only density to50%.
Input checks validate object/volume count, source order and kinds, actual
modifier transform and metadata containing only name/matrix/density (no wall
overrides). Native positive sparse extrusion must appear only after the
modifier is added. Complete ordered output comparison remains mandatory.

Both provisioned executions set `ARES_ORCA_RUNS=1`,
`ARES_PARITY_REGION_PERIMETERS_REFERENCE=1`, explicit
`ORCA_APPDIR=/tmp/orca-official-242/squashfs-root` and distinct artifact roots.
The untouched native executable SHA256 is
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

- Initial RED: `/tmp/ares-compatible-regions-higher-density-red.log`, exit100,
  1.439s. Baseline case-jbDTLa PASS; modifier case-yYpu6y ARES_ERROR at the
  early `sparse_infill_density` guard after native sparse witness passed.
- Input-strengthened RED:
  `/tmp/ares-compatible-regions-higher-density-verified-input.log`, exit100,
  1.415s. Baseline case-Y6FPJh PASS; modifier case-3WXE52 fails at that same
  guard after all new input assertions and native sparse witness passed.
  Input SHA256 in both cases:
  `a06ce5221a38679f5fd5a3fdde7531fe1df7d6d0db5a1070be41a072aec1453b`.
  Strengthened native output183413bytes SHA256
  `cb6f702c71b2a2e72872544aca58b2f9c0b427aedb148f94e1221f7b9f0ac72e`
  matches actual command output `compatible-regions-native/plate_1.gcode`.
  Export and both slice command journals show exit0 and the explicit official
  appdir. Cache manifests still mark producer identity unknown; separate
  journal evidence is not a claim that those manifests were repaired.
  The complete native-to-native diff between executions contains only the
  generator timestamp line. Neither RED was discarded or chosen as a PASS.
- Initial flow/context-only preservation:
  `/tmp/ares-region-layer-context-preservation.log`, exit0, 201PASS/61.970s.
  This snapshot predates the horizontal/extra-bridge address changes.
- Full addressing preservation:
  `/tmp/ares-region-layer-addressing-preservation.log`, exit0,
  284PASS/92.036s/6slow. Single-region/previous stage behavior only; no claim
  that public multi-region neighbors or empty slots executed through the guard.
- Final fixture-inclusive clippy:
  `/tmp/ares-region-layer-addressing-clippy-final.log`, exit0/3m22s, with
  existing workspace warnings. No diagnostics identify this slice's changed
  context/preflight/propagation/extra-bridge/compatible-region files.
  `cargo fmt --check` and `git diff --check` exit0.
- Default workspace checkpoint:
  `/tmp/ares-region-layer-addressing-workspace.log`, exit100/89.643s,
  5718/7485 run, 5717PASS/1originalTreeFAIL/9skip. Fail-fast left1767 unrun;
  this is not a full workspace result. A `--no-fail-fast` full rerun follows.
- Complete no-fail-fast checkpoint:
  `/tmp/ares-region-layer-addressing-workspace-no-fail-fast.log`, exit100,
  250.053s, 7485run/7484PASS/1originalTreeFAIL/9skip/35slow. The unchanged
  support-header test fails at `support_type/support_style`; initialized-KSR
  complete ordered bytes still PASS. No other workspace failure. The new
  live-oracle tracer is env-gated in this general run; its actual provisioned
  executions remain RED above. This does not close any modifier/Tree gate.

Six-axis independent WIP review found no definite addressing index defect and
closed the fixture-input repair. It explicitly withheld multi-region execution,
commit, modifier and Normal/Tree acceptance. Downstream grouping, usage and
emission remain unported; no new guard, fixture relaxation or Normal merge.

The configuration/usage scope supplement passed independent recheck. Its next
active public tracer must include parent density0 and a positive-density
modifier inheriting a non-default sparse infill filament, so real tool selection
as well as geometry is observed. The current single-filament density tracer
cannot prove that role collection or normalization. Source bounds/parent
inheritance must be shared with actual slicing; do not synthesize usage by
assuming every modifier overlaps every model part.

### Source-boundary correction before the next implementation slice

Re-read `PrintObject.cpp:3737,3784-3811`, `Print.cpp:451-489` and
`Slicing.cpp:130-141`. The previous review's proposed missing raw-range loop
in `project_slice/extruders.rs` is not an upstream per-object behavior:
`PrintObject::slicing_parameters` calls its own collector, not Print's.
Do not implement that proposal or change the existing occupied/unoccupied
range expectations solely to satisfy it. Public range slicing remains an
open separate rewrite, including the existing raw-intersections range guard.

Re-read `SupportMaterial.cpp:1414-1430,1524-1528`: lower-slice filtering and
cantilever classification explicitly use the first region's external width.
Preserve these two choices while repairing the all-region input adapter.
That adapter also requires actual region-owned fills; its current physical-
layer aggregate cannot be borrowed once per region as if it were regional.
No production change, new test result, commit eligibility, or modifier/Normal/
Tree acceptance is claimed by this source-boundary correction.

### Physical-layer perimeter handoff

Analysis found fill_entities::move_thin_fills still zips layer-major output
with region-major records, dropping later-region paths and ownership. Preserve
the existing public modifier RED seam, route every slot to its physical layer
and store regional borrowed ranges over one perimeter owner. Switch support
Region views to those ranges; do not pretend aggregate fill collections are
regional. Verify unchanged single-region output and full regressions, then
independent six-axis review. This is not full modifier/Tree acceptance or
permission to submit Normal-dependent/unconsumed prerequisites.

Public RED `/tmp/ares-region-handoff-public-red.log` exits100/1.610s at the
existing multi_region_layer_slices guard after the single-region baseline
comparison succeeds. First Clippy exits101: added private range field blocked
an existing external struct-update test. Match existing project_slice field
visibility; use let-else for empty records to remove new excessive nesting.
Independent static review found slot/range/ownership routing consistent and
confirmed support borrows before extrusion_islands consumes perimeters.
It retained all-region fill bridge detection and post-move ownership gates.
Final snapshot verification/review remains pending; no fixture relaxation,
new guard, modifier/Tree GREEN or Normal submission.

Frozen source SHA256: fill_entities
eabfb9def3ee137c7380a39f8882ad94de00f5c12d008b3f840c6c591ba7fd3c;
types d7f09a2e9ec57ce87904d987f9f0e26ee9a0369bd41c02c5b75a1fc4a6a986a3;
support region821cd251032c146f704532ee9e7b6918220eb44f81209af9db26881950dffaa1.
Independent static re-review verified SHAs and quality repairs without new
mandatory finding. Final Clippy `/tmp/ares-region-handoff-clippy-final.log`
exits0/1m01s,326core-test warnings/309duplicates; added nesting warnings are
gone. Fmt check exits0. Final workspace/public run the unchanged snapshot;
their finite terminal evidence remains to be recorded.

Final public `/tmp/ares-region-handoff-public-final.log` exits100/3.401s.
Single-region case-hoWNBh remains PASS using the existing
ordered_bytes_generator_only comparator, not raw file hash equality:
input SHA2567c1b981aba422da7d9e722dcee111fde56486866af204c7799df0e075d18802e;
Ares264798bytes SHA256b006e5c6e68605ee53377fa2c8a9e9fa5e4c44d6eed45b30c5a5f1dd4e02aa0c;
native264804bytes SHA256d5dd95d12c7c7d830df70b221fb5d1902ce9052c9f446cc7bf9cd9ac9edcf8d1.
Modifier case-MpE1Lo remains RED at the original multi_region_layer_slices
guard: input SHA256867e55b66a9b90c3a98577b339311f06637607f7a2a80048e659f0a26afdd710;
native308694bytes SHA256c8f86fe35172844803515e97794f93e36dadfd2b5d5eb1f24308ca9975a43321.
Both manifests retain runner_cache/unknown producer; no fresh-native or global
parity attribution. No native cache/scheduling/reference/comparator policy
change, fixture relaxation or new guard. This public run does not exercise
the multi-region handoff and cannot close that acceptance gate.

Frozen workspace `/tmp/ares-region-handoff-workspace-final.log` terminates
exit100/254.838s:7494run/7493PASS/one original KSR Tree-width FAIL/
9skipped/35slow. No failed-exec or SIGINT events. The initialized KSR
ordered-byte comparison passes; original Tree-width fixture/assertions have
empty diffs. Diff check0/index empty; no commit/push. This regression scope
does not prove real multi-region execution, fill ownership, support or Tree
complete-output/platform/global inventory acceptance.

Independent six-axis final review verified logs/plan/diff and found no new
static mandatory finding. It limited single-region PASS to the existing
generator-only comparator and retained actual multi-region execution,
regional fill/support consumption, Tree/platform/global inventory gates.
No approval to submit Normal dependencies or unaccepted components.

### Regional fill collection handoff

Before implementation, public `/tmp/ares-region-collections-red.log` exits100
at the existing modifier guard. Follow Fill.cpp1274,1354-1356 selected
SurfaceFill.region_id ownership: store append ranges, preserve collection
order, borrow only the requested region for bridge detection. Existing
ironing records its actual input region; native multi-region ironing grouping
remains deferred. Simplification preserves collection indices; ranges are
only valid before extrusion_islands moves collections. Full workspace and
unchanged public output checks plus six-axis review follow the frozen source.
No guard/fixture/comparator/native-policy changes or Normal/Tree acceptance.

Frozen SHA256: generate
e68bc3baf0774dc36c2527056cfe5935894cfb2b32dd282954fe911e0c366872;
types71e3db8ac5d7da483824616be2bc654ccdb541c7ef05d19e22509188d8bff77b;
support region9bc533a91ac18f67908c16fc6cc32066cd5a19f890bd3102a599c5d460c04709.
Fmt check0; Clippy `/tmp/ares-region-collections-clippy.log` exits0/49.99s,
326core-test warnings/309duplicates, no new mandatory quality finding.
Independent six-axis static review confirms selected owner/order and borrowed
geometry without mandatory finding. It retains post-extrusion_islands range
lifetime, actual all-region fill/ironing/support execution and public acceptance.

Frozen public `/tmp/ares-region-collections-public-final.log` exits100/3.409s.
Control case-geH7a5 PASS is limited to existing ordered_bytes_generator_only:
input5427c9619a82382f5f3e7aed7bbd47f3578fdb9bec9a3899a33277184d2c7727;
Ares264798bytes/b006e5c6e68605ee53377fa2c8a9e9fa5e4c44d6eed45b30c5a5f1dd4e02aa0c;
native264804bytes/dabaf93597b05f428e6947e5c58519751367d53c8c511501f6eb3fafdc27b84e.
Modifier case-FAaAtw stays RED at original multi_region_layer_slices.
Manifests retain runner_cache/unknown producer; no raw hash equality, fresh
native attribution, real multi-region path coverage or Tree acceptance claimed.
Workspace remains running on this unchanged Rust snapshot; terminal evidence
and final independent review pending. No submission of the Normal dependency.

Initial workspace `/tmp/ares-region-collections-workspace.log` exits100 at
89.849s:5727/7494 run,5726PASS,one original KSR Tree-width FAIL,9skip;
1767not-run due to default fail-fast. This is finite but NOT full regression
coverage. Initialized KSR ordered comparison PASS40.347s; original header
test/fixture diffs empty. Run the same frozen source with --no-fail-fast in
`/tmp/ares-region-collections-workspace-final.log` to cover all tests before
final review; no compile/Rust mutation between those runs.

Final workspace `/tmp/ares-region-collections-workspace-final.log` exits100,
253.691s:7494run/7493PASS/one original KSR Tree-width FAIL/9skip/36slow.
No failed-exec/SIGINT events. Initialized KSR ordered comparison PASS40.359s.
Modifier final input867e55b66a9b90c3a98577b339311f06637607f7a2a80048e659f0a26afdd710;
native308694bytes/263d38bb857a72b936a3891ff48528d56a99e1bf5807f22c6e623c8e5cefc723.
All source SHAs remain frozen; original KSR header test/fixture diff empty,
diff check0/index empty. No new guard/comparator/reference/native changes.
Real multi-region execution, Tree complete output and Tier1/global inventory
remain open. No commit/push; final independent six-axis review requested.

Final independent six-axis review verified all frozen SHAs, logs/plan,
clean diff check and empty index; no new mandatory static repair. It retains
the sole Tree-width failure, modifier guard RED and unknown reference producer;
generator-only control PASS is not raw bytes equality. No approval for Normal
dependency submission or Tree/global completion. All-region fill/ironing,
support assembly, complete output and platform acceptance stay open.

### All-region fill grouping

Public RED `/tmp/ares-all-region-fills-red.log` exits100/1.634s at unchanged
multi_region_layer_slices after control PASS. Source Fill.cpp828-1053 requires
shared projection state and LockedZag maps across all regions, then global
parameter interning and ordered first-owner selection. Port into group_fills,
params and coalesce; merge new region member no-overlap polygons with union_ex
once. Keep priority/narrow processing once after combined grouping. No new
private structure/source pin tests, fixtures, guards or native policies.
Run fmt/Clippy, freeze code, workspace --no-fail-fast and original public
comparison, then six-axis independent review. This cannot close actual
multi-region/Normal/Tree gates while earlier/later production consumers remain
unported. Included/deferred scope is recorded in spec/ARD before code edits.

Independent six-axis static review finds no deterministic mandatory defect;
shared state, first-owner, new-member union, ordering and error propagation
match the selected source scope. Public RED does not exercise cross-region
sticky state/LockedZag/union, so those behavioral gates remain open. Initial
Clippy0/47.96s adds one nesting warning (327core-test warnings/310duplicates).
Use let-else for first-owner assignment, without changing behavior. Final
`/tmp/ares-all-region-fills-clippy-final.log`0/47.20s restores326/309.
Fmt check terminal and frozen tests follow; no Rust edits during final runs.
Frozen SHA256 group_fills
4afcf0e59cf30f3f495cc6d5fb67e4346995f8478b0c37518f0a7f7027a733f5;
params f6367c296969c6e7db39572849890280308f5c4a982367c8196355c2c7c8874b;
coalesce d6d3f1a927f819c666e9520e2fad0b730073e114697145e74afea9c5aa59e404.

Fmt check exits0 after formatting; independent static re-review confirms
all three SHAs and let-else ordering/error behavior, no mandatory finding.
Public final `/tmp/ares-all-region-fills-public-final.log` exits100/3.389s:
control case-FLVRLe PASS only under existing generator-only comparator,
not raw byte equality or real cross-region branch coverage.
input030c71f2eeb205fac4e716580171ea6abd4b82476933f3b77e64bbb97de048c1;
Ares264798bytes/b006e5c6e68605ee53377fa2c8a9e9fa5e4c44d6eed45b30c5a5f1dd4e02aa0c;
native264804bytes/c42c474be4331d89b6a80a8f11036f3f3565214cdfbcb7e1f1e01e16fb1bcd2e.
Modifier case-bFZ6Ly remains RED at original multi_region_layer_slices:
input867e55b66a9b90c3a98577b339311f06637607f7a2a80048e659f0a26afdd710;
native308694bytes/3a2a0a0cd9d38d1b3806694081e5ddf110ecab7711a10b7ef97892b67d692648.
Both manifests retain runner_cache/producer unknown. No policy/fixture/guard
change or native producer attribution. Workspace still live on frozen code;
terminal evidence and independent final review pending, no commit/push.

Final workspace `/tmp/ares-all-region-fills-workspace-final.log` exits100,
257.038s:7494run/7493PASS/one original KSR Tree-width FAIL/9skip/35slow,
no failed-exec/SIGINT. Initialized KSR ordered comparison PASS41.317s.
Fmt check0/diff check0/index empty; SHAs remain frozen, original KSR header
test/fixture diff empty. No actual cross-region grouping output coverage from
the guard-RED tracer; modifier/Tree/Normal/Tier1/global acceptance stays open.
Final independent six-axis review requested; no commit/push.

Independent six-axis final review verifies terminal logs/plan and frozen
quality repair, clean diff check/empty index, no new mandatory static problem.
It explicitly withholds submission/completion: original Tree-width FAIL and
modifier guard RED remain; generator-only control PASS is not raw equality,
reference producer unknown and actual cross-region behavior untested.
Continue the source-owned production path; preserve complete-output assertions,
Normal-before-Tree restriction and Tier1/global acceptance requirements.
