# ARD0089 — Canonical region and physical-layer ownership

Status: single-tool regional ownership integrated; multi-tool emission gap confirmed.

## Source owner and Rust destination

Rewrite `Layer.cpp:139-291::is_perimeter_compatible/Layer::make_perimeters`
and `LayerRegion.cpp:82-142::LayerRegion::make_perimeters` in
`ares-core/src/project_slice/perimeters/`. Carry canonical ownership through
PrintObject surface preparation, `Fill/Fill.cpp::group_fills/Layer::make_fills`,
extrusion_islands, island_print_order, SeamPlacer and GCode layer emission.

Configuration ownership follows `PrintApply.cpp::generate_print_object_regions`,
`PrintObject.cpp::region_config_from_model_volume/object_extruders`,
`Print.cpp::object_extruders` and
`PrintRegion.cpp::collect_object_printing_extruders`. Shared destinations are
`project/region_graph/`, effective_config usage and project_slice/extruders.
Former single-record-per-layer scaffolding is replaced, not preserved as an
independent Ares pipeline.

## Canonical configuration and invocation contracts

- Keep each object's original region identity, options and slices. Do not
  clone whole objects to simulate regions, apply region0 configuration globally
  or infer regional ownership from path coordinates/roles.
- ResolvedProjectObject::all_regions retains deduplicated source-created
  configurations. Region graph bounds preserve composed transforms, float
  range-clipped triangle bounds, contour inflation/epsilon, source volume order
  and reverse parent traversal. Slicing attaches actual occurrence/bound IDs;
  early normalization must not invent global IDs.
- Modifier-derived density/configuration reaches canonical parent-derived
  options and usage gathering. Inherited sparse/solid filament roles differ
  from a modifier's generic extruder selector.
- Keep print-wide raw-range extruder usage separate from per-object slicing
  parameters: Print.cpp451-489 includes positive raw layer-range extruders;
  PrintObject.cpp3784-3811 uses actual all_regions/volumes instead.
- Compatible perimeter groups execute in physical-layer/source order, including
  shared fuzzy RNG. Singleton surfaces are borrowed; multi-member surfaces are
  safety-offset and bucketed by extra-perimeter count. Selected owner Flow
  follows upstream sparse density; redistribute generated fill boundaries back
  to original member regions.
- Compatibility is not necessarily transitive. Preserve native comparisons
  (strict float epsilon, exact explicitly compared values, serialized width
  tokens) and the later-member scan without an invented done check. Invocation
  storage is separate from regional output slots so repeated participation is
  representable.

## Physical layers, flows and downstream consumers

RegionLayerIndex/slot identity is not a physical layer number. Region-local
neighbors differ from aggregate physical neighbors. GCode scheduling, layer
labels, cooling and seam preparation consume the physical plan even when
regional records are empty.

`PrintObject.cpp::discover_vertical_shells:2024-2440` owns shared physical
Top/Bottom/hole caches and per-region Flow/configuration. Shared cache operands
combine all regions, while regional projections retain their source solid
spacing and canonical neighbor identities. The source spiral num_layers bound
applies to cache construction, projection and trimming, not only one stage.
Inactive trimming preserves surfaces. The unchanged spiral regression and
four current-spacing assertions remain output/behavior gates.

`PrintObject.cpp:3315-3391` bridge-over-infill consumes aggregate current/
upper cuts and region-local upper/perimeter Flow. Candidate Flow controls
spacing/depth; every region contributes physical surfaces and patterns. Empty
front records cannot discard a later-region candidate. Lightning generator
activation/configuration follows its separate source owner on main, rather
than adding a regional alternate algorithm.

Perimeter/fill entities carry region ranges/identities through thin-fill
transfer and extrusion island moves. Per-tool emission grouping is not yet
implemented; regional identity alone cannot establish it. Borrowed ranges are valid
before the move, not after ownership leaves the shell. Preserve allocation
and source order; support bridge views borrow only their own region's range.
This ownership is consumed by actual emission, not metadata-only plumbing.

`SupportMaterial.cpp` all-region traversal and
`SupportParameters.hpp:85-97` average bridge_flow are shared Normal/Tree
consumers (ARD0084/0086). Native front() reads are retained only at their source
boundary; there is no generic all-region replacement for front().

## GCode state and header ownership

`GCode.cpp:6150-6178/7635-7636` chains infill separately per canonical region
and updates positive sparse density from the current nonempty emitted region
before travel, including ironing. Empty phases leave active configuration
unchanged. Raft uses the print default region installed at5341.

`GCode.cpp:5453-5468` retains separate ordinary-infill and ironing phases,
including infill-first wall order. Output cursor and regional configuration
are continuously consumed, not reset between flattening phases.

Width headers visit shared ModelObject all_regions in source order with first
actual PrintObject fallback/support operands and print-wide support presence.
Do not deduplicate equal configurations across distinct objects. Used filament
IDs must come from ToolOrdering-generated usage, not configured filament count
or unfiltered configuration usage. The current count-as-ID header consumer is
incorrect for multiple tools and is part of the confirmed emission gap.

`GCode.cpp:5796-5805` clipped-empty loops make no emission-side changes.
Preserve existing wipe state and skip wipe-before-external/inward motion.
This is source-defined empty geometry, not silently omitted support.
The mainline clipped-loop regression and Lightning configuration-owner tests
remain registered in the combined snapshot.

## Included, deferred and acceptance

Included: modifier-derived configuration/usage, canonical per-region
perimeters/flows, compatible invocation groups, fill redistribution, physical
shell caches, downstream typing/fills/islands/seams, support flows and actual
single-tool GCode consumers. Public modifier-width, all-region bridge-flow and
later-region density controls use complete output comparison. The additional
inherited-second-filament baseline/modifier test is RED: the adapter lacks
GCode.cpp4971-5075 region/role-to-extruder buckets and5241/5308 tool traversal/
set_extruder. Its scope decision is open in the Tree plan; it is not accepted
output parity, and fixing the header alone cannot close its body difference.

Deferred: broader non-transitive/fuzzy/mixed-configuration combinations,
persistent native cross-job caches, default parallel first-arrival policy and
the all-printer/all-option matrix. Verified public inputs are not proof of
every configuration. No unsupported guard or region0 fallback substitutes for
included behavior; Normal/Tree merge together when their consumers are coupled.

Final nextest, original/seeded output, clippy/fmt, WASM and independent review
evidence belongs in the plan;
superseded slice-state notes are archived there.
