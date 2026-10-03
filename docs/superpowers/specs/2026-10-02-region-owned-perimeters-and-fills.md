# Region-owned perimeter and fill rewrite prerequisite

## Active-region retraction density continuation

Source GCode.cpp6155-6178::extrude_infill applies the nonempty emitted region's
config before chaining; needs_retraction at7635-7636 consumes its positive
sparse density. Port this consumed operand to gcode_emit/motion/layer.rs and
layers/object_entry.rs, borrowing canonical PostRegion options by the existing
infill region identity. Do not replace it with any-region-positive object state.
Apply on entry to each nonempty ordinary/ironing region before its first travel;
empty phases leave active state unchanged, as upstream does. Raft emission
retains default-region density from GCode.cpp5341, not a fabricated object region.
Split layer phase emission into its own Rust module when the change grows the
existing motion.rs beyond400LOC; no macro splitting or geometry/config cloning.
Included: this one active-region retraction operand and its actual full-output
tracer, with source-defined density changes in both directions. Deferred: other
regional MotionOptions, perimeter config installation, full tool ordering,
mixed-region infill-first, Tree/Normal and all printer/options/runtime coverage.
The unchanged public later-higher-density full output is RED at byte9291 and
contains two extra retract/wipe/hop sequences. Test the fix against the whole
output, without M73/retraction filtering or altering the input/assertion.

## Modifier density admission continuation

Source PrintApply.cpp1029-1054::generate_print_object_regions and
PrintObject.cpp3664-3720::region_config_from_model_volume apply modifier region
options to their parent; sparse density is not an unsupported capability.
Destination project/effective_config/candidates.rs: retire only the obsolete
density rejection. Existing region_graph::add_modifier and RegionOptions::resolve
carry the override, and collect_bounded_project_usage now gathers all_regions
including the sparse/solid roles activated by positive density. These are
required prerequisites, not a replacement pipeline or density fallback.
Included: admission into those canonical region owners and removal of the
obsolete density-rejection assertion. Deferred: remaining modifier role guards,
full regional configuration/emission and Tree/Normal/global parity.
The unchanged later-higher-density public 3MF-to-complete-G-code comparison
supplies RED; any downstream divergence remains a real failure to diagnose.
Do not weaken that comparison, alter its fixture, or claim admission is parity.

## Clipped-empty loop state continuation

Source GCode.cpp5796-5805::extrude_loop clips the loop and returns immediately
if paths is empty, before wipe-before-external, extrusion or wipe-state updates.
Destination gcode_emit/motion/loop_paths.rs::emit: preserve this no-emission
state transition for ordinary fully clipped loops, moving the wipe hop after
the nonempty result; preserve the existing eligible scarf branch separately.
Included: clipped-empty-loop state retention and placement of the wipe hop.
Deferred: complete clipping/scarf/config/kernel rewrite, regional motion config
and Tree/Normal parity. The unchanged public width-modifier complete-output
test is the agreed tracer; a pass would establish its specific missing-wipe
cause, while a remaining mismatch must be investigated without filtering.
No new private seam or defensive capability guard, fixture/assertion change,
or speed/timing normalization. Returning for a source-defined empty clipped
result is algorithm behavior, not skipping an unsupported project feature.

## Regional infill chaining continuation

Port the consumed GCode.cpp4971-5068::process_layer by-region infill ownership
and 6150-6178::extrude_infill regional chaining into fill_entities.rs/types.rs,
extrusion_islands.rs, island_print_order.rs and gcode_emit/motion.rs.
Use existing collection_ranges and source-record region identity; retain thin
fill ranges during ownership transfer. Carry one region identity per emitted
infill entity through sortable flattening and no_sort collections. At emission,
chain each region independently in source region order for both ordinary infill
and ironing, updating the same current cursor after every emitted entity.
Included: this consumed ownership/chaining defect demonstrated by the unchanged
public modifier complete-output tests. Deferred: per-region motion configuration,
mixed-region infill_first dispatch, runtime global tool IDs and adjacent existing
ironing generation gaps, repeated compatible invocations, Tree and full parity.
Existing flat entity storage is a temporary compatibility shell; its explicit
infill identities must not be reconstructed from geometry or extrusion role.
The public 3MF-to-complete-ordered-G-code seam remains unchanged; no new private
test, output normalization, guard or modifier fixture change.

## Regional width-header continuation

Consumed motion follow-up: RetractWhenCrossingPerimeters.cpp9-55 gathers
all LayerRegion slices of the physical layer, while GCode.cpp7635-7642 tests
the resulting internal islands. Port this gathering to
island_print_order.rs::internal_surfaces and the entry/motion borrowed view.
Return references from all canonical regional slots, preserving region/surface
order and per-island containment (not a union across modifier boundaries).
Keep classification and travel kernel unchanged; no geometry clone/cache.
Existing rectangle and travel behavior assertions remain unchanged, with only
reference representation adaptation in their fixture. Full modifier byte
comparisons remain mandatory and RED until all downstream differences close.

Port PrintApply.cpp1680-1772 shared ModelObject region traversal and
GCode.cpp2676-2693's print-wide width blocks to gcode_emit/header.rs. The
resolved object's all_regions is the existing shared-region owner; consume
each source object's configurations once in source order, not each transform
or only its first model part. The upstream registration loop never inserts into
region_set: preserve distinct ModelObject blocks even for equal configurations,
without inventing cross-object config deduplication or treating local IDs as
global IDs. All blocks use the first actual print object's fallback settings;
support presence is print-wide (Print.cpp2049-2056), support Flow settings
remain first-object-owned. Resolve each width using its role-selected nozzle,
including the inner-wall nozzle for first-layer perimeter width. Reuse the
existing Flow nozzle selector and width arithmetic, propagating errors.
Included: this consumed borrowed registry traversal and header operands.
Deferred: runtime global-ID assignment, full regional motion/tool/config and
compatible/fuzzy invocation parity, Tree/Normal and complete printer/options.
Public 3MF-to-ordered-G-code is the agreed seam; retain complete modifier
comparisons and single-region controls without header/body filtering.

## Public perimeter entry continuation

Consumed follow-up: GCode.cpp1718-1784::collect_layers_to_print and
process_layer use physical object layers, not present region records. Port that
identity to layers/schedule.rs, layers.rs and layers/cooling_result.rs via a
borrowed traversal physical-layers view. The bridge-flow public input reproduces
an index97/length75 panic after entering these consumers. Keep all physical
layers even when the front region is empty; do not deduplicate a regional Z list
or skip layers to conceal the identity mismatch.

Source boundary: Layer.cpp139-291::Layer::make_perimeters and
LayerRegion.cpp82-142::LayerRegion::make_perimeters; Rust destination:
perimeters/preflight.rs and existing region-owned generator/fill handoff.
Validate every printing region before resolving canonical regional Flows,
instead of rejecting all multi-region objects in the temporary preflight shell.
Included: regional raw option/layer alignment validation and execution of the
existing public modifier complete-output tests. Deferred acceptance: repeated
compatible invocations under non-transitive float equality, full modifier
output, and Tree/Normal completion. A later panic or mismatch is a remaining
port defect, not grounds to restore a capability guard or relax assertions.

## Regional Flow consumer prerequisite

Port `Flow.hpp::Flow::scaled_width/scaled_spacing`,
`PrintObject.cpp::detect_surfaces_type:1577`,
`LayerRegion.cpp::process_external_surfaces:497-518`
into perimeters/types.rs and prepare_infill's surface_type_detection
and external_surfaces stages. These consumers read each
canonical region/layer input's own Flow, not the Classic generator's scratch
record at the same vector index. Preserve float-to-double scaling followed
by integer truncation and the selected coordinate scale. Do not introduce
cached scaled-flow vectors, geometry copies or a second prelude per region.
Included: these two production consumers and removal of their generator
slot coupling. Deferred: bridge candidate's physical-layer front-region Flow
(`PrintObject.cpp:2512`), remaining shell consumers, independent generator
invocations, fuzzy partitioning, regional neighbor traversal and modifier
gate removal. Public complete-output modifier REDs remain acceptance failures;
existing single-region complete-output controls must remain unchanged.

## Objective and upstream boundary

### Lightning generator configuration owner

Port PrintObject.cpp1010-1020::prepare_lightning_infill_data's positive-density
Lightning activation and Fill/Lightning/Generator.cpp66-100's first shared
printing-region configuration into candidate_expansion.rs::build_lightning_generator.
The activating region does not become the parameter owner. Included: these
consumed configuration operands and supplemental generated-polyline invariance
when an otherwise empty later Lightning region changes its width. Deferred:
all-region outlines, regional bridge expansion, native complete modifier output
and Tree; multi-range/shared-object ordering equivalence remains unproven.
Current single-range graph preserves the global first-region order. Existing
single-region output and public acceptance gates remain.

### Lightning regional expansion

Port PrintObject.cpp2593-2723 into candidate_expansion's expansion producer and
generator consumer, plus lightning_expansion.rs geometry. Lower gates read
original all-region Internal geometry with each region's own Lightning pattern.
Current regions preserve source order and first-empty Internal break, filter
candidate polygons by region, expand/intersect per candidate, union solid and
subtract sparse with holes intact. Immutable original surfaces preserve candidate
identities; only temporary expanded surfaces feed generator. Canonical regional
slot keys replace physical-layer-only expanded storage. Included: this consumed
source slice and generated-polyline behavior tests. Deferred: public modifier
parity, remaining regional consumers, Tree and full printer/options acceptance.
No unsupported guard or reference scheduling/randomness/build changes.

### Lightning physical-layer outlines

Port Generator.cpp151-171/180-193's all-region Internal/InternalVoid contours
and holes into build_lightning_generator using canonical region/layer slots.
No regional density/pattern filtering. Preserve generated tree polylines when
identical geometry moves from front to later region with configuration fixed.
Existing front-region bridge expansion is applied only to that region; full
regional expansion, modifier complete-output parity and Tree remain required.
Internal stage injection supplements the public ordered-G-code acceptance
seam; it does not substitute for it or permit a new unsupported guard.

### Physical-layer sparse anchor generation

Port PrintObject.cpp2736-2756 and Fill/Fill.cpp1394-1512's
Layer::generate_sparse_infill_polylines_for_anchoring call boundary into
transaction/candidate_expansion/layers.rs::prepare_infill_lines. Generate
anchors once per candidate's physical lower layer, using all-region
group_fills rather than the front region's presence or density as a gate.
Retain option validation for each present region using its own nominal Flow.
Included: this consumed call boundary and supplemental expansion invariance
when unchanged lower geometry/configuration moves to a later region.
Deferred: lightning construction, adaptive generators and complete modifier
output acceptance; existing public comparators and fixtures stay unchanged.

### Physical-layer deep sparse operands

Port PrintObject.cpp::bridge_over_infill's gather_areas_w_depth2819-2846
into deep_sparse_area.rs and transaction/candidate_expansion.rs. Each lower
physical layer contains borrowed regional surfaces and its own region density.
Traverse source layer/region/surface order; aggregate sparse and non-sparse
geometry across all included regions before the existing shared union/closing/
difference. Preserve f32 depth arithmetic, immediate lower-layer inclusion,
InternalVoid classification and all existing assertions. Included: physical
regional view and consumed construction, later-region lower geometry behavior.
Deferred: all-region sparse anchoring/lightning, modifier guards and Tree.
Public full-output acceptance remains unchanged and required.

### Regional bridge expansion operands

Port PrintObject.cpp::bridge_over_infill3154-3214 and3242-3265's first-candidate thick
Flow, all-current-region context and per-candidate Flow/configuration into
transaction/candidate_expansion/layers.rs. Candidate region/layer identities
address canonical inputs; physical layers enumerate all original regional
surface/boundary/pattern operands, including an empty first region. Included:
these consumed expansion operands and supplemental later-region behavior
coverage. Deferred: all-region deep lower geometry, sparse anchoring and
lightning expansion/generator consumers, modifier guard removal and Tree.
Preserve public full ordered output gates; injection coverage cannot replace
modifier output acceptance.

### Physical-layer bridge surface rewrite

Port PrintObject.cpp::bridge_over_infill3315-3391 into
bridge_over_infill/transaction/surface_rewrite.rs. Compute aggregate current
cuts and upper ensuring areas once per physical layer, then rewrite every
region in source order. Upper ensuring uses the candidate's own regional
solid Flow; local perimeter ensuring uses the current region's Flow. Resolve
both through canonical region/layer slots, preserving original candidate
surface ownership. Included: this consumed surface rewrite, borrowed operands
and supplemental regional behavior coverage. Deferred: lightning/expansion,
shared shell cache, invocation execution, modifier gate removal and Tree.
The unchanged public complete-output modifier tests remain acceptance gates.

### Physical-layer bridge candidate gathering

Rewrite PrintObject.cpp::bridge_over_infill2493-2589 in
prepare_infill/bridge_over_infill/{stage,candidates}.rs. CandidateLayer becomes
a physical-layer view containing borrowed CandidateRegion operands, not a
region-major canonical slot. The current physical first-region solid Flow
controls spacing even if that region has no slices, following
LayerRegion.cpp21-28 / PrintRegion.cpp25-53. Reuse canonical first-region Flow
where present; otherwise resolve the same configured nonbridging solid Flow,
with physical height/first-layer ID/nozzle/width inputs. Lower geometry and
100% density classification aggregate all regions before shared closing and
offset/difference. Iterate current regions/surfaces in source order; preserve
CandidateSource physical layer/region/original surface index. Remove the last
candidate Classic scratch zip without cloning geometry. Existing geometry
tests adapt only their region packaging, preserving all assertions. Deferred:
regional lightning, expansion/rewrite addressing, shared shell cache,
invocations, fuzzy partition, guard removal and Tree. The unchanged public
modifier complete-output tracer remains the acceptance seam, not an internal
candidate shape test; earlier guard RED is not cross-region behavior proof.

### Canonical shell alignment continuation

Port ownership at `PrintObject.cpp::discover_vertical_shells:2334-2435` and
`discover_horizontal_shells:3955-4161` into vertical_shell_trimming/filtering/
assignment and horizontal_shell_promotion/propagation. Their production
operands are canonical inputs and shell data, not Classic scratch records.
Included: remove unused scratch count/presence zip/index checks and only the
mixed tests' matching subcases. Keep every remaining canonical/cache/plan/
scale/identity check, outer object alignment, geometry and transaction test,
and current-versus-stopped-spacing assertions. This is prerequisite to
`Layer.cpp:139-291`'s repeated compatible-owner invocation semantics, not a
new invocation scaffold. Deferred: physical regional neighbors, shared cache,
candidate front-region Flow, invocation execution, modifier gate removal and
Tree. Public complete-output tests remain unchanged acceptance gates.

### Physical vertical-shell traversal prerequisite

Next rewrite boundary is PrintObject.cpp2008-2446::discover_vertical_shells,
particularly cache ownership2024-2148 and regional projection2154-2311.
Destinations are prepare_infill/vertical_shells/{stage,cache,types,physical_cache}.rs,
vertical_shell_projection/{stage,gather}.rs, and the source execution-window
predicate in vertical_shell_trimming/stage.rs. Physical layer index is distinct
from canonical region/layer storage slot. Shared top/bottom cache applies when
multiple printing regions exist and interface_shells is false; otherwise
top/bottom is region-local while holes still include all physical-layer regions.
Include source regional Flow expansion, simulated perimeter offset/min spacing,
lslices-based extra holes, source spiral num_layers over the entire shell operation
and source-order neighbor visits. Shared-cache Flow resolves SolidInfill first,
then External/Internal only for positive perimeter count; no Overhang evaluation.
Replace flattened-slot
neighbor traversal and one-region alignment assumptions only with the owning
source semantics; do not just relax asserts. Existing private Flow spacing
regression retains all four assertions. Public complete-output modifier tests
remain acceptance, not errors accepted as a substitute. Supplemental regional
shell-geometry seam awaits user preference before any new test is written.
Deferred adjacent consumers are downstream regional trimming/regularization/filtering
and horizontal shell propagation, compatible perimeter execution closure, public
guard removal and Tree support. These remain required for full goal completion.

### Shell Flow consumer continuation (implemented operands)

Horizontal entry continuation owns PrintObject.cpp3955-4161's
discover_horizontal_shells in horizontal_shell_{promotion,propagation}/stage.rs.
Included: canonical region/layer cardinality and identities, physical pattern i,
retained same-region source/neighbor traversal and Flow, EnsureAll skip and
transactional dirty commit. Existing single-region entry checks are temporary
compatibility scaffolding and are replaced by the owning regional invariants.
Public perimeter entry/compatible execution and complete regional output remain
deferred but required, not replaced by any rejection or test assertion relaxation.

The downstream regional continuation owns
PrintObject.cpp2314-2440::discover_vertical_shells in
vertical_shell_{trimming,regularization,filtering,assignment}/stage.rs.
Included: canonical regional records/Flow and count/identity invariants;
filtering visibility reads global physical idx_layer±1 lslices. Keep the entire
source spiral execution window and existing transactional commit/disposal.
The previous single-region validation is a temporary shell, replaced by actual
source region/layer ownership, not simply deleted. Deferred horizontal shell
neighbors, compatible execution and public entry closure remain mandatory for
complete modifier-output parity. No public guard removal in this portion.

Rewrite `PrintObject.cpp::discover_vertical_shells`' regional Flow operands
at2132,2174-2175,2242/2269 and2334-2403 into prepare_infill's vertical_shells,
vertical_shell_projection, vertical_shell_regularization and
vertical_shell_filtering stages. Cache expansion, anchor spacing,
regularization and tiny-shell filtering read canonical region Flow, not
Classic prelude records. Remove those four consumers' generator record-count
and presence coupling and obsolete private subcases that pin it; preserve real
canonical object/region/layer identities and empty regional outputs. No
geometry clones, additional caches or replacement private pin tests.
Deferred: the native all-region/shared cache branch2024-2106, physical-layer
neighbor traversal, other downstream generator alignment checks, independent
invocations, fuzzy partition, modifier gate removal and Tree support.
Use the unchanged public complete-output modifier tracers and single-region
controls; blocked entry REDs do not establish regional behavior coverage.
Preserve the current-versus-stopped-neighbor spacing behavior test and all
four assertions. Its user-authorized supplemental injection moves to the
canonical regional external Flow, using the selected scale; it must not be
deleted merely because the old Classic scratch injection is obsolete.

Port `Layer.cpp::is_perimeter_compatible` / `Layer::make_perimeters`,
`LayerRegion.cpp::LayerRegion::make_perimeters`, downstream region-local
surface processing in `PrintObject.cpp`, and `Fill/Fill.cpp::group_fills` /
`Layer::make_fills`. Destination: the existing `ares-core/src/project_slice/`
perimeters, prepare_infill, fill grouping / fill_entities and emission modules. Exact surface-processing
functions and neighbor scopes must be cited in each implementation slice before
that slice changes code. This is a libslic3r rewrite, not a new Ares pipeline.
Island/region grouping and emission follow `GCode.cpp::GCode::process_layer`;
seam preparation follows `GCode/SeamPlacer.cpp::SeamPlacer::init`. Include the existing
extrusion_islands, island_print_order and seam_placement Rust destinations.
Configuration/usage prerequisites follow
`PrintApply.cpp::generate_print_object_regions`,
`PrintObject.cpp::region_config_from_model_volume`,
`Print.cpp::object_extruders`, `PrintObject.cpp::object_extruders` and
`PrintRegion.cpp::collect_object_printing_extruders`, targeting
`project/effective_config/` and `project_slice/extruders.rs`.

## Included behavior

- The emission prerequisite follows `GCode.cpp:5453-5468,6131-6178`:
  ordinary infill precedes infill-first walls, but ironing follows those
  walls. The single-region destination is `gcode_emit/motion.rs`; regional
  ownership, configuration switching and compatible perimeter groups remain
  deferred until their production consumers are ported. Validate through the
  public exported 3MF and complete ordered G-code comparator, not private
  ordering helpers.

- All region configurations and physical layer identities survive generation.
- Modifier density activation participates in all-region filament usage and
  normalization; generic volume extruder selectors alone are not sufficient.
- Preserve the two configuration collectors: print-wide usage retains raw
  positive range selectors (`Print.cpp:451-489`), but per-object slicing
  parameters use actual regions and volume extruders without a raw-range
  loop (`PrintObject.cpp:3784-3811`). Neither is generated-path ToolOrdering.
- Shared source-owned full/range-clipped float3D bounds determine modifier
  parents; nonintersecting modifiers do not create enabled region roles.
- Generated-path tool usage remains distinct from configuration usage.
  `Print.cpp`'s `ToolOrdering`-derived slice filament list and
  `GCode.cpp:2623-2632`'s one-based ID serialization own the header behavior.
- Source-compatible groups merge slices by extra perimeter count and use the
  greatest-density member's configuration, flows and actual perimeter owner.
- Classic/Arachne dispatch retains spiral and upper-same-region behavior.
- Fill surfaces and no-overlap geometry return to every originating region;
  counterbore extra-fill recovery remains source-owned.
- Typing, shells, fill groups and emission consume the correct region/layer,
  without cross-region neighbor walks or duplicated aggregate-layer output.
- Existing single-region full output is preserved on all Tier1 adapters.
- Storage layout must not change source-selected execution order or shared
  seeded fuzzy RNG consumption, including Classic and Arachne generators.

## Scaffold replacement and exclusions

Replace the one-record-per-physical-layer assumption only where needed to
carry the cited region outputs. Keep one object owner; no proxy objects, whole
object clones, invented pipeline, compatibility fallback or new guard.
The existing perimeter rejection is removed only when its entire production
consumer path is region-aware; leaving it open temporarily is not parity.
Support-material iteration, Tree generation and native reference cache lifetime
are deferred to their own source slices. Normal/Tree merge restrictions remain.

## Acceptance

The existing public test
`runner::support_modifier::orca_parity_support_modifier_regions_preserve_complete_output`
is the tracer: identical project fixtures and complete-output assertions,
including active native support and both region-width witnesses. Its current
downstream perimeter error is a failure, not an allowed feature outcome.
Full modifier acceptance requires complete ordered native output equality,
not only geometry, header widths or source/structure pinning.
Also exercise compatible wall settings with the greatest-density region in a
later group member, equal densities retaining the first member, and layers
where a region is empty. Compare complete public output with active independent
native witnesses; private owner indices are not acceptance. Retain a
single-region control and seeded fuzzy ordering regression. The existing
different-wall-width tracer covers incompatible groups, not these branches.
The actual dual-nozzle J1 tracer also retains parent density0, inherited sparse
filament2 and a density-only modifier. Require positive native sparse XY
extrusion under selected T1 plus complete baseline/modifier output equality.
Any earlier baseline printer failure remains reported, not hidden by changing
profiles or bypassing the complete comparison.

Retain the unchanged KSR support-header test and complete native comparison.
Test each observable slice before implementation; use existing agreed stage
observations only as supplementary evidence, not a replacement acceptance seam.
Verified preserving prerequisites may submit independently with code and docs,
full regression/platform/browser evidence and six-axis independent approval.
No documentation-only commit and no Normal batch submission before Tree parity.

## Physical-layer perimeter handoff slice

Rewrite the remaining fill_entities::move_thin_fills zip using the source
Layer/LayerRegion perimeter ownership from Layer.cpp139-291 and
Fill/Fill.cpp1212,1361-1369. Visit all region-major record slots, route each to
its physical layer, and retain empty region slots. Keep one owned flat
perimeter allocation per layer with explicit per-region ranges; borrowed
region views must use those ranges, never repeat the layer aggregate. Retain
the first nonempty perimeter allocation and move subsequent collections,
without geometry clones. Move every region's thin fills to its physical layer
in region order. This slice does not claim regional fill-collection ownership,
compatible-group execution, all-region support traversal or Tree output.
The existing complete-output modifier tracer is the acceptance seam; the
unchanged single-region KSR control must remain byte-identical. No private
storage/source pinning tests or new guards are introduced.

## Regional fill collection handoff slice

Port the ownership at `Fill/Fill.cpp:1274,1354-1356` into fill_entities/types
and generate: tag append ranges with SurfaceFill.region_id without copying
geometry or changing order. Borrow region-filtered collections in the support
Region bridge view. Record the actual input region for existing ironing;
native multi-region ironing groups (`Fill.cpp:1507-1724`) are deferred.
Do not assign merged fills to all region_id_group members. Separate thin gap
fills cannot trigger the bridge-role check. Ranges expire when collections
move to extrusion_islands; this slice does not wire those later consumers or
replace group_fills' region0 shell. The same unchanged public RED tracer and
single-region control remain required; no private metadata-pinning tests.

## All-region fill grouping slice

Port Fill.cpp828-1053 into group_fills.rs/params.rs/coalesce.rs: source-ordered
region-layer slot lookup, global parameter interning, shared projection state
and LockedZag maps, first-surface owner and representative, ordered distinct
region_id_group membership and union_ex of their no-overlap geometry. Preserve
single-region complete output and apply clipping/narrow splitting only once
after global grouping. Internal-void extension1076-1152, native multi-region
ironing, compatible perimeter execution, downstream region/island/tool ownership
and support assembly are deferred; do not add guards or change public fixtures.
The existing public modifier RED and control remain the acceptance seam;
reaching the grouping code in actual multi-region execution is still required
before claiming this branch's output parity.
