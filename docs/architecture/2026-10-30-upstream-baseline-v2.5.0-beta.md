# Upstream baseline migration: OrcaSlicer v2.5.0-beta

Date: 2026-10-30. Decision: **accepted**. This document is documentation
only, not a parity switch.

## Decision, immutable inputs and current limits

The next parity target is tag **v2.5.0-beta**, commit
**`345efec64c7b95a05ce5cf96df0dde66a158835c`**, authored/committed
2026-10-27T22:30:27+08:00. The previous reference is v2.4.2,
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The fetched tag was resolved to the requested commit before analysis.

The user chose the new upstream baseline rather than extending 2.4.2-specific
parity indefinitely. New work must port **the pinned beta's libslic3r/libvgcode**,
not design an Ares pipeline or preserve accidental 2.4.2 output against beta.
No dual engine, version-detection fallback, tolerance, comparator relaxation or
fixture-specific rule is introduced by this decision.

**This analysis neither binds nor executes a beta oracle.** Producer
availability/qualification is a separate contract, not a prerequisite for this
source-only document. There are no runtime parity, timing, determinism or
cross-platform PASS claims here. Existing executable paths, fixtures, reference
caches and integration gates still belong to their recorded old revisions until
their producers are explicitly switched. A version
string alone must never relabel old outputs as beta outputs.

## Complete source-change inventory

Requested command, executed against the fetched tags:

```sh
git diff v2.4.2..v2.5.0-beta -- src/libslic3r
```

The name-status manifest, independently checked with `--no-renames`, contains
**582 files: 437 modified, 130 added, 15 deleted**. Every path occurs once in the
linked tables, including headers, deletions, native-only utilities and unported
features. These are **274 bounded current-port/counterpart mappings and 308
"not ported" entries**, not a claim that 274 complete native files are rewritten.
Mappings describe the Ares code at the time of the inventory.

| Proposed sole owner | Inventory | Files |
|---|---|---:|
| config owner | [Config/profiles](2026-10-30-upstream-baseline-v2.5.0-beta-inventory-config.md) | 23 |
| geometry owner | [Geometry/mesh/import](2026-10-30-upstream-baseline-v2.5.0-beta-inventory-geometry.md) | 269 |
| walls/seams owner | [Walls/seams](2026-10-30-upstream-baseline-v2.5.0-beta-inventory-walls-seams.md) | 48 |
| fill/support owner | [Fill/support/surfaces](2026-10-30-upstream-baseline-v2.5.0-beta-inventory-fill-support.md) | 84 |
| emission/tooling owner | [Emission/tool/material planning](2026-10-30-upstream-baseline-v2.5.0-beta-inventory-emission-tooling.md) | 73 |
| processor owner | [Processor/platform/native-only surfaces](2026-10-30-upstream-baseline-v2.5.0-beta-inventory-processor-platform.md) | 85 |

A `partial port` row identifies an actual bounded Rust owner, not feature parity
or full class coverage. A `not ported` row is an explicit gap; borrowing a constant,
a same-named module or a citation in a comment is not itself a port. New beta-only
files are marked unported even where an older adjacent concept exists. This is
file-level planning, not an exhaustive proof of every changed function's behavior.

Adjacent changes are prerequisites, not hidden in the 582-file total:
`resources/profiles` has **13,289 changed paths** with rename detection disabled
(7,167 M / 4,460 A / 1,662 D); this is an inventory count, not semantic-profile
count. The config owner owns the vendor/inheritance/default manifest, including
BBL, Creality, Qidi, OrcaFilamentLibrary and other changed vendors, not just smoke profiles.
`deps_src/clipper2` is the geometry dependency boundary; changed libvgcode data
headers/conversion code belong to the processor owner. GUI/OpenGL/runtime changes
there are not implicitly included in rendering-neutral Ares crates.

## Migration order and bounded exit criteria

The order is dependency-first. Within a layer, use small upstream-file/function
slices, not one monolithic rewrite. Work that does not consume an unfinished
contract may be analyzed in parallel; final integration follows the contracts.
Each exit below means a source-complete bounded slice plus its **single targeted
public RED/GREEN filter and fmt/Clippy**, not per-slice FULL or workspace E2E runs.
End-to-end interaction coverage belongs to integration-level E2E runs.

### 0. Producer/source contract — config owner, before any beta parity result

Pin the unmodified target source/resources/compiler/build options and each
instrumented oracle's exact patch, ELF and producer identity. Seed initialization
or observability patches must not change the algorithm under test. Never infer
all determinism from the new mesh sorts: RNG-dependent seam/template paths still
need a declared producer contract. Keep old evidence read-only and separately named.
**Exit:** pinned beta producers are available and the targeted
runner consumes them explicitly; missing oracle means blocked, not skipped PASS.
No oracle build or harness switch is implemented by this ARD.

### 1. Config, presets and profile resources — config owner

Source: `PrintConfig.cpp:PrintConfigDef::init_fff_params`, `handle_legacy`,
`DynamicPrintConfig::normalize_fdm_2`, `update_values_to_printer_extruders_for_multiple_filaments`;
`Config.cpp:ConfigBase::set_deserialize_raw`; `Preset.cpp:Preset::normalize`;
`PresetBundle.cpp:PresetBundle::full_fff_config`, `load_system_presets`.
These functions, associated `PrintConfig.hpp` types and the pinned profile tree
are one contract: changed enums/defaults/variant columns must not be patched only
in the footer serializer.

Rust boundary: `ares-core` `options/registry`, typed option families, legacy and
FDM normalization, `options/project_variants`, `profiles/{catalog,composition,compatible}`,
`project/effective_config`; byte-oriented import remains in core, filesystem in CLI.
Include definitions/defaults/normalization used by already-supported FFF inputs;
recognizing a new option does not authorize executing its deferred feature.
**Exit:** literal public config/admission or public slice witnesses distinguish
old/new defaults, enum/legacy handling and relevant preset inheritance; unsupported
feature activation is explicit, never silently ignored or coerced to an old mode.

### 2. Geometry: Clipper2, mesh slicing and ordering — geometry owner

Source: `ClipperUtils.cpp:c2_to_expolygons`, `c2_offset_setup`, `offset`, `union_ex`;
`ClipperZUtils.cpp:clip_zpaths`; `deps_src/clipper2` engine/offset implementations.
Beta deletes `clipper.cpp/.hpp`, `Clipper2Utils.*`, `Clipper2ZUtils.hpp` and routes
ordinary wrappers through Clipper2. The wrapper is NOT a transparent namespace
rename: it handles stray-hole reassignment, collinear retention, degenerate/CW
paths, offset shortest-edge and arc-tolerance policy. Z clipping deliberately
keeps coincident input Z before invoking the fill callback.

`TriangleMeshSlicer.cpp:coplanar_edges`, `slice_make_lines`, `slice_slabs_make_lines`,
`chain_lines_by_triangle_connectivity`, `slice_mesh`, `slice_mesh_ex` are a separate
ordering/topology slice. New canonical line ordering uses the upstream tuple
`(edge_a_id, edge_b_id, a_id, b_id, a.x, a.y, b.x, b.y, edge_type, flags)`;
it is not permission to invent a tie key elsewhere. Coplanarity checks opposite
vertices against both transformed face planes at `SCALING_FACTOR`, preserving
face orientation and genuine shallow corners. Single-plane masked edge IDs have
their own classification; the seed junction and slab paths have distinct handling.

Rust boundary: replace the old engine behind `geometry/clipper.rs` and its Z/offset/
polytree modules; port canonical ordering/coplanar treatment into `mesh_slicer`
intersection/topology/chaining/slabs and their `project_slice` consumers. Existing
Clipper1 and encounter-order scaffolds are temporary old-boundary implementations,
not a fallback backend. Keep geometry integer/numeric operations source-faithful.
No new geometry crate is proposed without a separate justified extraction decision;
no native-only C++ dependency may break browser WASM.
**Exit:** targeted public slice cases with literal/oracle output distinguish nested
holes/open Z paths/offset behavior and coplanar versus shallow mesh corners; ordered
loops and starts are preserved, not canonicalized by the comparator. Integration-level
E2E, not each slice, carries platform and scheduling/reproducibility coverage.

### 3. Walls and Arachne — walls/seams owner, after the geometry contract

Source: `PerimeterGenerator.cpp:PerimeterGenerator::process_classic`, `process_arachne`,
`split_top_surfaces`; `Arachne/WallToolPaths.cpp:WallToolPaths::generate`,
`separateOutInnerContour`; `Arachne/SkeletalTrapezoidation.cpp:generateToolpaths`,
`applyTransitions`, `propagateBeadingsUpward`; changed beading strategies and
`Feature/FuzzySkin/FuzzySkin.cpp` at their concrete changed methods.
Rust: `project_slice/perimeters/{classic,arachne}`, `arachne/{beading,trapezoidation,wall_toolpaths}`,
`extrusion_entity`, bounded fuzzy-skin modules. Include supported classic/Arachne
ordering, narrow-hole/top/overlap and width/junction behavior; do not compensate
for wrong Clipper2 results with perimeter-only numeric tuning.
**Exit:** targeted public wall cases retain exact supported extrusion geometry,
width-derived E and final output; any retained GCC ordering/float evaluation must
be justified against beta source, not only the old Arachne geometry repair.

### 4. SeamPlacer, SeamContour and PreciseSeam — walls/seams owner

Source: `GCode/SeamPlacer.cpp:SeamPlacer::init`, `gather_seam_candidates`,
`find_next_seam_in_layer`, `align_seam_points`;
`GCode/SeamContour.cpp:SeamContour::Resolver::resolve`;
`GCode/PreciseSeam.cpp:init_precise_seam_data`, `extract_perimeter_segments`,
`insert_strong_seam_point`, `collect_weak_modifier_segments`,
`apply_weak_modifiers_to_perimeter`, `restore_precise_seam_positions`.
Port old-mode changes first, then new precise-seam geometry/priority as its own
source-cited slice after the geometry owner's ModelVolume type/import contract is ready.

Rust: `project_slice/seam_placement`, seam candidates/split/spatial helpers;
new beta-only structures live under that owning boundary, not a second seam pipeline.
Include contour identity/reliability, source-edge binding, strong/weak modifier order
and holes. Defer UI editing of modifiers; do not defer required core semantics by
silently treating a precise modifier as an ordinary legacy enforcer.
**Exit:** targeted public projects distinguish contour continuation and strong/weak
modifier selection with literal expected seam/output positions. No private seam
planner tests or computed copies of the placement algorithm.

### 5. Fill, surface preparation, support and brim — fill/support owner

Source: `Fill/Fill.cpp:group_fills`, `Layer::make_fills`, `Layer::make_ironing`;
`Fill/FillBase.cpp:Fill::new_from_type`;
`Fill/FillConcentricInternal.cpp:FillConcentricInternal::fill_surface_extrusion`;
changed pattern methods in `FillRectilinear.cpp`, `FillAdaptive.cpp`, lightning
and TPMS sources; `PrintObject.cpp:PrintObject::prepare_infill`;
`PrintObjectSlice.cpp:PrintObject::slice`;
`Support/SupportMaterial.cpp:PrintObjectSupportMaterial::generate`,
`Support/TreeSupport.cpp:TreeSupport::generate`,
`Support/TreeSupport3D.cpp:generate_tree_support_3D`; `Brim.cpp:make_brim`.
Rust: `fill`, `project_slice/{prepare_infill,group_fills,fill_entities,region_slices,
support_material,raft}`, `brims`, `surface` and flow adapters.
Include existing admitted patterns/support/contact/shell/brim behavior with new
geometry semantics. New adaptive fields, spiral-inset and corner-smoothing feature
activation are separate follow-ups, not reasons to skip changed defaults/call sites.
**Exit:** targeted public fill/support witnesses preserve roles, boundaries, tool
usage, spacing/E and output ordering, without rebuilding an Ares-owned scheduler.

### 6. GCode, GCodeWriter and travel contracts — emission/tooling owner

Source: `GCode.cpp:GCode::_do_export`, `process_layer`, `travel_to`, `needs_retraction`,
`append_full_config`; `GCodeWriter.cpp:GCodeWriter::toolchange`, `travel_to_xy`,
`travel_to_xyz`, `extrude_to_xy`, `_retract`;
`GCode/AvoidCrossingPerimeters.cpp:AvoidCrossingPerimeters::travel_to`;
`PlaceholderParser.cpp:PlaceholderParser::process`.
Rust: `gcode_writer`, `gcode_move_emit`, `project_slice/gcode_emit` orchestration,
motion/travel/path routing, template/expression and config-export consumers.
Establish ordinary fixed-tool emission/travel first. The emission/tooling owner
then completes layer7's material-planning producers and integrates their consumers
here; the actual
ToolOrdering-before-emission upstream dependency is not inverted by this numbering.
**Exit:** one targeted public filter proves the selected writer/travel/render-site
slice with literal/oracle complete output, including error/admission when relevant;
no new metadata normalization or suppression of config/footer/motion differences.

### 7. ToolOrdering, WipeTower and FilamentGroup — emission/tooling owner

Source: `GCode/ToolOrdering.cpp:ToolOrdering::get_recommended_filament_maps`,
`reorder_extruders_for_minimum_flush_volume`;
`Print.cpp:Print::update_filament_maps_to_config`, `update_to_config_by_nozzle_group_result`;
`FilamentGroupUtils.cpp:update_used_filament_values`;
`FilamentGroup.cpp:FilamentGroup::calc_group_by_enum`, `calc_filament_group`;
`GCode/WipeTower.cpp:WipeTower::plan_tower_new`, `generate`, `nozzle_change` and
changed `WipeTower2.cpp` methods.

Important beta break: recommendation now returns `MultiNozzleUtils::LayeredNozzleGroupResult`,
not the old integer vector. Print's updater accepts extruder/volume/nozzle maps;
used-slot merging, variant expansion and map_2/retract rebuild lifecycle changed.
An old auto-only vector refresh/37-key result is NOT a beta proof or a reason to
restore 2.4.2 behavior. Static grouping data needed by existing one-/two-extruder
machines is required; selector/rack/dynamic multi-nozzle optimization remains deferred.

Rust: `options/{filament_map_recommend,project_variants}`, `project/effective_config`,
`project_slice/gcode_emit/{tool_ordering,wipe_tower,tool_transition}`. The config owner
applies schema/PrintApply-owned changes on request; the emission/tooling owner alone
writes Print/GCode planner files.
The general FilamentGroup optimizer is not currently ported: it cannot be declared
covered by the bounded TPU helper. Scope its required static branch before GREEN.
**Exit:** targeted public fixtures distinguish manual/auto, used/unused assignments,
variant maps and supported tower paths; emitted config and tool sequence match beta,
with no fixture-dependent regrowth or old-runtime snapshot fallback.

### 8. GCodeProcessor and rendering-neutral data — processor owner

Source: `GCode/GCodeProcessor.cpp:GCodeProcessor::process_G1`, `process_G2_G3`,
`calculate_time`, `run_post_process`, `TimeMachine::plan_klipper`;
`GCodeProcessor.hpp:GCodeProcessorResult::MoveVertex`, statistics/role data;
`GCodeReader.cpp:GCodeReader::parse_line`;
`GCode/CoolingBuffer.cpp:CoolingBuffer::process_layer`, changed FanMover/SpiralVase/
PressureEqualizer passes; adjacent libvgcode `GCodeInputData.hpp`, `PathVertex.hpp`,
`Types.hpp` and their data conversion methods.
Rust: `gcode_processor`, `project_slice/gcode_emit/processor`, cooling/fan/spiral/
pressure modules, `ares-vgcode` data conversion and `ares-wasm` byte/data adapter.
Audit time-planner/default-density/statistics changes as real semantics; do not
transfer the old 7348/7347 inherited-failure allowance into the new baseline.
**Exit:** targeted public processed-slice/output cases compare literal/oracle move,
role/layer/time/statistics data and final postprocessed bytes. No OpenGL, native
viewer, filesystem or slicer logic may leak into rendering-neutral vgcode/WASM.

## Explicitly deferred new feature execution

| Feature | Pinned upstream boundary | Scope decision |
|---|---|---|
| Parametric CAD/sketch/BRep, assembly editing | `CAD/CadDocument.*`, `GeometryEngine.*`, `SketchEngine.*`, `Format/STEP.*` | No CAD/OCCT kernel, editor or STEP rebuild in this baseline slice; existing mesh inputs remain the seam. |
| Texture bake/displacement/painting/to-colour | `TextureBake/TextureBakePipeline.hpp:PipelineSettings`, `TextureDisplacement.*`, `TexturePainting.*`, `TextureToColor/TextureToColor.*` | No texture mesh mutation, image pipeline or GUI; model/config feature identity must not be silently lost. |
| Mixer/virtual blended filaments | `FilamentMixer.cpp:parse_mixed_components`, `parse_gradient_curve`, `sample_gradient_curve`; `FilamentMixerModel.hpp` | No gradient blending/material expansion/G-code mixer behavior. |
| Belt printers | `BeltSliceStrategy::apply_preslice_transforms`, `BeltTransform.*`, `BeltGCode.*`, `GCode/BeltKinematics.*`, `Support/BeltFloorContext.*` | No belt frame/kinematics/brim/purge/support execution. Ordinary Cartesian frame handling still required. |
| New dynamic/rack/selector multi-nozzle and AMS preload search | `MultiNozzleUtils.*`, `FilamentGroup::calc_filament_group`, nozzle-centric ToolOrdering paths | Defer new hardware/optimization; port shared static result/config semantics needed by already-supported machines. |
| New fill activation | `Fill/AdaptiveInfillField::radial`, `depth`; `FillSpiralInset.*`; `FillCornerSmoothing.hpp:CornerSmoother::push` | Separate opt-in follow-ups; current pattern/default changes are not deferred with them. |
| New exclusion-volume routing and repair/arrange engines | `ExclusionVolumeGeometry.*`, `GCode/ExclusionVolumeTravelAvoidance.*`, `MeshRepair.*`, `IMEXArrange.*` | No new feature execution or native mesh-repair/packing service. |
| SLA, desktop/device/cloud/viewer runtime | `SLAPrint*`, `SLA/*`, AppConfig, ProjectTask/PublishSettings, libvgcode Viewer/OpenGL | Outside platform-neutral FFF migration; isolated reused geometry/data mappings are listed, not SLA/runtime coverage. |

Deferred means outside execution support, not a fallback to old algorithms. Schema
preservation and source-cited public admission behavior must distinguish unsupported
activation where needed. Every later feature requires its own owning upstream
slice; this table does not itself scope any implementation.

## Prior 2.4.2 work

Earlier 2.4.2 fixes, including the layer-region ordering/Arachne geometry and
numeric chain and the filament-map normalization/refresh chain, must be
source-compared against beta before retaining, adapting or superseding them. Preserve still-valid
concepts, not obsolete byte output or snapshot lifecycles. In particular, the
Arachne chain's Clipper1-sensitive geometry requires new engine provenance, and the
old vector map refresh cannot stand in for beta's layered/volume/nozzle-map update contract.

Reuse immutable input fixtures where beta admits them; obtain new expectations
from the new source/oracle without overwriting historical results. No automatic
baseline waiver, blanket golden regeneration or relabelling a 2.4.2 PASS as beta.
Old full-run counts and known failures are history, not beta exit criteria.

## Parity cutover

Cutover happens **once**, only after the beta producers are ready and the
prior-baseline integration, test cleanup and behavior-name migration have landed.
**One commit** switches the parity environment, both locally and in Tier 1 CI,
from the 2.4.2 producers to the pinned v2.5.0-beta producers. There is no
dual-baseline runner or fallback to the old producers.

Immediately after cutover, a **full parity run on the beta producers** is made.
Its failure set is recorded as the **beta migration baseline**, with the run's
producer/source provenance. That baseline is kept **outside
the repository**, not a checked-in waiver or replacement golden set.

From then on, changes must introduce **no new failures versus the current
baseline**, and the failure set must **only shrink**.

The 2.4.2 producers remain rooted read-only for historical evidence, not as an
active acceptance baseline. **No new 2.4.2 work is accepted after cutover.**

## Minimal per-slice testing and naming policy (effective now)

- RED first through a **public API**: parity/E2E, CLI, or the ares-core public slice
  API (processed output may cover public viewer data). No private/internal tests,
  module-structure coupling, mocks or mockall. Keep test logic declarative: no
  loops or computed expectations; Native bytes or independent literal values are
  the expectation, not a copied implementation.
- A slice runs **only its targeted RED/GREEN filter**, plus
  `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`,
**No per-slice full run, REQUIRE=1 non-parity gate, broad
parity sweep or rerun.** Integration-level E2E carries interaction coverage.
- Proptest is optional for a crisp upstream invariant through an existing public
  API; do not expose private geometry merely to test it. There is no external
  service, so do not introduce testcontainers.
- Every NEW test function, case and fixture directory uses a meaningful behavior
  name, e.g. `coplanar_mesh_keeps_shallow_corners` or
  `filament_refresh_preserves_unused_assignments`. No process labels, milestone/phase
  tags or opaque abbreviations. Printer models/G-code commands are permitted only
  together with the behavior being checked.
- Do NOT rename existing identifiers during port slices. Dedicated test-cleanup
and behavior-name migration commits handle renames; account
  for those commits explicitly when comparing manifests, rather than mixing name
  churn into migration behavior.

No beta oracle run or target RED is claimed here.

## Area partition

The six inventory tables are the exhaustive upstream-file partition;
each changed file belongs to ONE area. File counts include deferred/native-only
tracking, so they are not effort estimates.

Every executable slice must cite the pinned upstream file **and function/type**,
name its exact Rust destination, distinguish included/deferred behavior, and state
which old scaffold is replaced or retained only as a compatibility shell. Reject
independently designed Ares pipeline features. Review source fidelity and the
minimal targeted proof separately; oracle unavailability is never evidence of parity.
