# Normal support complete-output parity

Owning upstream boundary: `PrintObject.cpp::generate_support_material` /
`_generate_support_material`, `Support/SupportMaterial.cpp::
PrintObjectSupportMaterial::generate`, `Support/SupportLayer.hpp`,
`Support/SupportCommon.cpp` and `GCode.cpp::collect_layers_to_print`.
Destination: `ares-core::project_slice/support_material/` and the existing
`gcode_emit/layers/` object/support scheduling and emission boundary.
Object-only and raft scaffolding is a temporary compatibility shell.
Do not design an independent Ares support pipeline.

The landed-support diagnostic includes an AGG prerequisite used by
`SupportMaterial.cpp::rasterize_polygons`: rewrite the contribution semantics
of `deps_src/agg/agg_rasterizer_cells_aa.h::set_curr_cell/line` and
`agg_rasterizer_scanline_aa.h::sweep_scanline` in
`geometry/agg_raster/cells.rs::Outline`. Its coordinate-keyed aggregate map
is a temporary storage shell around native local-cell flushing and sweep
summation. Vertical intermediate cells must preserve previous contributions
to the same coordinate. Include only this accumulation repair; defer AGG
layout/clipping redesign and neighboring algorithms. Use the existing
public bridge-flow-modifier test's unchanged landed control and modifier
complete streams, plus retained same-input CLI replay; no raster-private
shape pins or replacement oracle. The focused public landed-control test
reuses the unchanged model/configuration export of the modifier witness so
the accumulation regression has a separately runnable complete-output gate;
the modifier witness retains all its existing assertions. Modifier/Tree
acceptance remains required.

Prerequisites include `Flow.cpp::support_material_flow`,
`support_material_1st_layer_flow`, `support_material_interface_flow`, and
`SlicingParameters::create_from_config`'s support-nozzle limits and gap
assignment. Reuse these native contracts in the existing raft caller;
public complete-output raft probes cover first-layer width fallback,
automatic widths and zero/synchronized gaps without private/source pinning.
Native errors and remaining byte differences are failures, not completion.

The zero-top-gap dependency also includes the native
`SupportParameters::contact_fill_pattern` selector and
`SupportCommon.cpp::generate_toolpaths`'s raft-contact filler selection.
Rewrite into `raft/fill_params.rs` and `raft/fills/patterns/` using native
`FillConcentric::_fill_surface_single` and `FillGrid::fill_surface`
contracts. Preserve scaled-integer spacing, float density division,
outside-in contour order and nearest-point splitting; do not reuse a
similar filler whose numeric domains differ. Existing raft contact plans
are only the compatibility caller of these upstream support contracts.

Preserve `FillBase.cpp:2464-2466` first-call connection cost thresholds in
private per-job `SlicerContext` state shared across actual callers and
objects. The reference contract is one fresh native process per job, not
native long-lived process-global state. Keep other connection quantities
per-call and prove complete-output task isolation with repeated inputs
around jobs using different support spacing.

Generate complete output matching native Orca for both enable_support
Boolean values on the existing Ender-3 overhang input. Enabled normal(auto)
grid support must produce actual support X/Y motion with positive relative-E
extrusion, not merely an option string, role comment or matching header.
Disabled support must not produce support roles. Persist both sides and
the exported 3MF before output assertions. Strict ordered-byte comparison
excludes only generator identity/time and bypasses no differing commands.

Include upstream normal-support contacts, projections, independent heights,
base/interface regions, generated paths and object/support Z merging needed
by this fixture. Tree Auto/Manual and upstream style dispatch, nonzero
support angle, enforcers/blockers and all modifier regions are subsequent
implementation slices but mandatory before this Normal batch can merge.
See ARD0086 and the Tree spec/plan; rejection guards or silent omission do
not satisfy parity. Adjacent multi-material support and other option domains
remain open and receive no coverage credit from this fixture.

Require witnessed public RED/GREEN, separate test modules, sub400LOC Rust
files without include-based splitting, full Nextest workspace, fmt/Clippy,
four-platform evidence and independent six-axis approval before the normal
support completion commit and push, including completed Tree acceptance and
the shared angle/annotation/multi-region repairs above. Independently validated
behavior-preserving slices may commit earlier only when independent of this
unmerged Normal batch and free of newly rejecting guards. Completed dependencies have a separate
ARD0085 checkpoint with their own bounded gates, not a replacement for this
ordinary-support exit. Global all-option/default/printer parity remains
open until its full original acceptance criteria are proved.

## Nonzero Normal support angle

Rewrite `SupportMaterial.cpp:659-666,841-842::SupportGridPattern` rotation
into `geometry/support_grid.rs` and remove the project adapter's angle guard
only with this behavior implemented. Rotate support/trimming polygons by
negative radians before rasterization, retain them for extraction and rotate
selected output back. `Polygon.hpp::polygons_rotate` and
`MultiPoint.cpp:27-35::rotate` own cosine/sine order and nearest integer
rounding. Toolpath angles continue through `SupportParameters.hpp`'s existing
parameter port. Drive a public real overhang project at 45 degrees, verify
effective angle and real native support extrusion and compare complete bytes.
Keep zero-degree support, cooling and unequal-height skirt controls unchanged.

## Annotation prerequisite source slice

`PrintObjectSlice.cpp:1545::PrintObject::slice_support_volumes` and
`:77::slice_volume` own support annotation mesh slicing: retain centered
object/volume transforms, slice at actual float slice Z, repair loops and
union only layers where multiple nonempty annotation volumes meet.
`SupportMaterial.cpp:1329-1330` consumes these enforcer/blocker layers during
contact generation. Rust destination is a private support_material input
module called by project.rs; reuse source-equivalent mesh intersection and
loop primitives, not normal model-part slicing/compensation. Replace the
annotation guard only after public real-volume RED/GREEN. Each annotation
type must demonstrably affect actual native output. No volume may be
discarded or treated as a normal model part to make the fixture pass.

## Unequal-height multi-object skirt verification

Verify the existing rewrite of `Print.cpp::_make_skirt`,
`GCode/ToolOrdering.cpp:137-148::calc_max_layer_height`,
`1407-1448::mark_skirt_layers`, and `GCode.cpp:4293-4328` skirt_done
through the same public 3MF-to-complete-G-code seam. Rust destinations are
`gcode_emit/skirt.rs` and `gcode_emit/layers/skirt_schedule.rs`; this is
verification of the source-owned rewrite, not a new Ares pipeline.

Use two distinct model objects, bed-grounded cubes of4mm and10mm height,
separate XY placements,0.2mm layers, one skirt loop and skirt_height99.
Verify the loaded public object heights and effective option values.
Require native skirt on all50 global layers, including after the shorter
object ends, and compare complete output without removing motion or time.
Persist both streams/input even when comparison fails. Use a fresh native
output directory, not a cached reference. Defer sequential printing,
multi-tool skirts and additional support-gap contexts; none receives
coverage credit from this single by-layer case. Any resulting failure must
be diagnosed at its owning upstream boundary before production changes.

The independent-parts public RED identifies a prerequisite identity bug:
`bbs_3mf.cpp:744` defines `Id` as `(model part path, local object id)` and
`_generate_current_object_list:4867-4890` looks up that key. Rewrite the
temporary `raw_intersections::validate_shared_meshes` key to this same
leaf-resource identity; do not treat equal numeric IDs in distinct parts
as shared meshes. Preserve existing explicit shared-mesh/same-resource
unsupported behavior as a deferred Ares capability, not native invalidity.
Existing private rejection fixtures must represent actual same-resource
sharing, and public regression keeps independent parts with local ID1 in
each. Preserve the first shared-leaf RED separately; no ID renumbering.

### Object-origin transition prerequisite

Port `GCode.cpp:5659-5669::set_origin` into
`gcode_emit/motion/origin.rs` and replace the direct object-origin assignment
in `gcode_emit/layers.rs`. `Point.hpp:280` rounds the scaled translation
to nearest. Translate `last_scaled_position` (local scaled integers) and
`wipe_path` (local millimeters representing scaled integers) by old minus
new origin. Preserve writer X/Y and `wipe_start`, which are already in the
absolute emitted frame. Update origin and the derived extruder offset.
No RNG/cache policy changes belong to this slice. Combined skirt occupied
hulls remain a separate `Print::_make_skirt` prerequisite; a partial motion
improvement cannot count as complete-output parity. Keep the existing
public unequal-height fixture and strict comparator unchanged.

### Combined skirt plate-frame hull slice

Rewrite `Print.cpp:2669-2696` local occupied hull collection,
`2794-2803` PrintInstance::shift, `2842-2845` combined group union and
`2973-2985` final hull/loop generation in `gcode_emit/skirt/occupied.rs`
and `skirt.rs`. Translate per-object contour/support/installed-raft hulls
for every printable instance using the source-derived separately-scaled
model center plus instance XY translation (`PrintApply.cpp:150`,
`PrintObject.cpp:110,130`). Share that integer shift calculation with
existing object-origin emission; do not recompute mesh bounds per instance.

Store skirt loops in plate scaled integers. `GCode.cpp:4411::generate_skirt`
uses `set_origin(unscaled(offset))`, with zero offset for the combined group;
use the existing source-owned set_origin operation before selecting the
skirt seam, splitting or clipping. No first-object local compatibility
translation may move a combined skirt to the last-printed object.

Move first-layer bounds ownership into `footprint/first_layer.rs`: use the
same per-instance shift for object contours (`Print.cpp:2995-3019::
first_layer_islands`), combine plate-frame skirt bounds directly, and
translate the existing first-object local brim shell once. Do not add the
first object's center to plate-frame bounds a second time. These extractions
are functional Rust mod boundaries, not include-based source splitting.

Included: existing combined, single-tool by-layer skirt with multiple
independent objects, existing support/raft hull contributors and first-layer
extent consumers. Deferred: per-object grouping/collision merge, wipe-tower
occupied hull, full multi-object brim rewrite, sequential and multi-tool
skirts; existing raft/brim shells remain temporary compatibility around
their cited source behavior. Keep all public input/settings/native-output
activity assertions and generator-only whole-byte acceptance unchanged.

### Empty-tool layer emission prerequisite

Port `GCode.cpp:4580-4582::process_layer` and
`GCode/ToolOrdering.cpp::collect_extruders` into `gcode_emit/layers/group.rs`:
an installed support layer with no support extrusion roles contributes no
tool and returns empty G-code before changing emission state. Preserve its
geometry and raw schedule count. Pass the empty result, native layer ID and
raw last-layer cooling-flush flag through the existing equalizer/cooling pipeline
(`GCode.cpp:3708-3715`, `PressureEqualizer.cpp:174-200`).
The final header count follows emitted `LAYER_CHANGE` tags
(`GCodeProcessor.cpp:3323-3325,1136-1139`); progress and compatible tail keep
the raw sliced count. Included: current single-tool object/support/raft
adapter. Deferred: complete multi-tool/wipe-tower ordering. The unchanged
manual-enforcer public 3MF/full-output test is the RED-to-GREEN acceptance;
do not remove geometry layers or change its fixture to reduce the count.
Count exact whole-line reserved comments after end templates and FanMover,
using `GCodeProcessor.hpp:375` printer-selected BBL/compatible vocabulary
through the existing `gcode_emit/tags.rs::Tags`,
not arbitrary tag substrings before the tail. Public custom-end tags cover
both a nested comment false positive and a real final reserved tag. A closed
cube with only bottom solid layers is the public final-empty reachability
case; delayed support-cache flushing additionally needs metadata regression
coverage with and without the equalizer.

### Modifier/all-region input slice

Rewrite `SupportMaterial.cpp::collect_region_slices_by_type`,
`SupportMaterialInternal::has_bridging_extrusions`, top-contact per-region
overhang/margin loops (`:1430,:1613`), bridge-height averaging (`:1786`) and
trim-object LayerRegion traversal into `support_material/project.rs` and
`project/region.rs`. Each layer must expose all native regions, matched through
the actual perimeter input's region/planned-layer identity, not record-list
position or `raw_slices(0, layer)`.

The first real modifier input stops before support: the earlier single-region
guards are temporary scaffolding. Included upstream prerequisites are
`PrintObjectSlice.cpp:1262-1365` region-aware compensation and
`Layer.cpp:185::make_perimeters` compatible-region grouping plus each actual
LayerRegion's flow/input generation. Port these owners into existing
`compensation/` and `perimeters/` boundaries before removing either guard.
Aggregate layer islands are owned by `Layer.cpp:34-67::make_slices` and
`ClipperUtils.hpp:364::union_safety_offset_ex`, not plain union or a replacement
for region surfaces. Port this into `slice_ordering.rs`, preserving region
identity, surface metadata and holes; supplemental helper tests do not close
the modifier complete-output acceptance gate.
Keep layer and region identity separate through existing surface preparation;
Included compensation behavior: source positive-growth first-region priority,
merged active-region outline, negative-growth trimming and elephant-foot
backup/first-region flow, then intersection back into every region. Port
`PrintObjectSlice.cpp:1314-1365` and `Layer.cpp:119-138::merged` into
`compensation/multi_region.rs`. Preserve lazy first-branch epsilon within one
project invocation; cross-project static lifetime and native parallel
first-arrival policy remain unresolved, not permission to alter the oracle.
do not assume the present one-region record list already supports flattening.

The included ownership prerequisite is
`Fill.cpp:1216-1367::Layer::make_fills`: fills return to
`m_regions[surface_fill.region_id]->fills`; moving each region's perimeters
and thin fills uses that same region identity. Preserve this association in
`fill_entities/` until support generation can borrow region-specific entities;
the current layer-only aggregate is a temporary shell. Do not give each
region the same aggregate or clone extrusion entities merely for support.

Included: real modifier volumes with distinct options, all top polygons,
bridge detection and average height, per-region contact width and trim inputs,
retained ordered object/support G-code. Preserve upstream's two intentional
front-region width sites (`:1422,:1527`), not a speculative upstream bug fix.
Deferred: unrelated multi-tool/sequential emission and viewer presentation;
neither is permission to omit modifier geometry or support output.

Acceptance: public exported overhang 3MF plus a real intersecting modifier,
loaded volume classification and effective distinct region options verified,
native actual support extrusion and generator-only complete ordered bytes.
Keep a same-model unmodified control, existing annotation/angle cases and the
unchanged KSR tests. Diagnose any earlier layer-only scaffold failure exposed
by this input before considering the support adapter repair complete.

The all-region input assembly slice targets support_material/project.rs.
Use physical-layer region vectors, actual region-layer raw/typed addresses
and previously recorded region-owned perimeter/fill views. Collect top
surfaces and trim views from every region in source order. Source1784-1788
gates on any bridging region but averages heights over all regions, including
non-bridging/empty ones. Source1422/1527's front widths remain unchanged.
Do not expand this slice into compatible perimeter/ironing/island/tool
generation or Tree dispatch; those remain required adjacent production work.
The existing public modifier complete-output tracer is the acceptance seam,
not private record/shape pins. Its earlier guard failure remains a failure.

Bottom-interface ratio follows SupportParameters.hpp85-97 and Print.hpp423-424:
average bridge_flow of object printing regions/all_regions, not first region
or only current nonempty slices, for both Flow branches. Add a separate public
modifier E2E with parent bridge_flow1 and modifier2 on a model with a lower
landing platform. Native positive Support interface XY/E extrusion at height
0.6 (versus control0.4 on nozzle0.4) must witness the two-region average1.5.
Retain both full ordered output comparisons, artifact manifests and original
fixtures/assertions. Coverage remains incomplete if an earlier gate prevents
Ares from executing this behavior; native witness alone is not Ares parity.

Selected support-grid islands must retain contour and holes in that order,
following SupportMaterial.cpp739-820::SupportGridPattern::extract_support
and ExPolygon.hpp415-422::polygons_append. Destination: geometry/support_grid.rs.
The existing public landed-support project tracer is the test seam; its
complete ordered output must remain unrelaxed. No private source-shape pins.
Included: move-based flattening of retained islands. Deferred: changes to
rasterization, sampling, fill chaining, estimator, styles or Tree dispatch.
The first M73 mismatch is not proof of a processor bug when later support
geometry differs. Full control/modifier/Tree acceptance remains required.
