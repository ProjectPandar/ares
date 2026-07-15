# Task 19B.3: Typed FDM Normalization and Effective Project Configuration

## Status and objective

This specification is a draft until its frozen bytes receive the independent
review approvals required below.

Task 19B.3 is the final configuration-resolution slice of Task 19B in
`2026-07-10-ksr-fdmtest-v4-gcode-parity.md`. It composes the already released
typed project settings, active variant materialization, export/runtime retract
views, model-option ownership, and layer-range association into the first
production effective-project configuration call.

The task ports the fixed Orca FDM normalization order and the configuration
side of used-filament discovery. It does not slice geometry or emit G-code.
After a valid project resolves, the public project path must still return
`ProjectSlicingIncomplete`. The persistent goal remains byte-for-byte
`ksr_fdmtest_v4` G-code parity after normalizing only the allowed generator name
and timestamp metadata.

### Pre-implementation review contract

The independent approvals that freeze this specification are design reviews,
performed before a Task 19B.3 implementation plan or implementation exists.
Reviewers must judge whether the specified behavior is source-faithful,
complete within its declared boundary, implementable through the named typed
destinations, and covered by acceptance criteria that can reject an incorrect
implementation. Missing Task 19B.3 production types, calls, and tests in the
current Ares tree are the expected pre-implementation state and are not a
reason to reject this specification.

A `REVISE` verdict at this gate must identify a defect in the frozen
specification itself, such as an inaccurate fixed-source claim, an omitted or
contradictory behavior, an unsafe ownership/API boundary, an unimplementable
requirement, a dishonest deferral, or acceptance criteria that cannot
distinguish the required behavior. Implementation conformance is reviewed only
after the independently approved plan has been executed.

## Corrected boundary from the older aggregate plan

The older aggregate Task 19B text grouped
`set_num_extruders`, `set_num_filaments`, and `get_parameter_size` with the
effective `Print::apply` work. Fixed-source audit shows that those functions are
preset/profile-ingress resizing behavior, not calls made by `Print::apply`.
They are explicitly excluded from this task.

This correction is behaviorally required. The committed project deliberately
contains raw four- and eight-position variant payloads. Task 19B.1A selects
from those raw positions before producing active physical/logical vectors.
Resizing them first would destroy the source indices and produce incorrect
values. Task 19B.3 resolves and validates active cardinalities but never runs a
registry-wide resize, first-element fill, or `set_num_*` compatibility helper.

`ProjectSettings` is already Ares' concrete typed representation of the full
dynamic project configuration. Task 19B.3 therefore does not create a second
flat struct containing the same 653 fields. The final full/export view remains
`ProjectConfigViews::full`; a new `BoundedResolvedProjectConfig` composes that
view with effective object, print-object-group, layer, and model-part region
state while making incomplete usage coverage explicit. This is a typed
compatibility representation of the included fixed `Print::apply` boundary,
not a new Ares-owned slicing pipeline or an arbitrary-project final result.

## Fixed upstream rewrite boundary

The baseline is OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/PrintConfig.hpp:628-631` declares
  `normalize_fdm`, `normalize_fdm_1`, and `normalize_fdm_2`.
- `PrintConfig.cpp:8520-8614` is the monolithic normalization reference;
  `:8617-8685` is the split first stage used by `Print::apply`; and
  `:8688-8740` is the changed-key-returning second stage.
- `PrintApply.cpp:1113-1194` reads the old `Print` usage state, calls
  `normalize_fdm_1`, calls the first `normalize_fdm_2`, then materializes the
  four variant families and prepares full/runtime state.
- `PrintApply.cpp:1256-1283` derives logical filament cardinality from the
  materialized `filament_diameter`, updates the full/default configuration
  owners, and preserves the full configuration separately from runtime
  filament overrides.
- `PrintApply.cpp:104-167` defines the exact 16-scalar transform ordering and
  equality used by the ordered transform-group set after removing XY
  translation. `PrintApply.cpp:1525-1621` creates those `PrintObject`s without
  attached regions, recomputes pre-region used filaments, and invokes the late
  `normalize_fdm_2`.
- `src/slic3r/GUI/PartPlate.cpp:3503-3510` calls `Print::apply` twice because
  the first call creates the `PrintObject`/region state that must inform
  normalization of the fresh full configuration on the second call.
- `PrintApply.cpp:1620-1657` applies the second-stage changed keys to the print,
  default-object, default-region, and full-config owners; only afterward does
  `PrintApply.cpp:1659-1768` generate, attach, and publish the effective regions.
  `:1662-1747` creates one region set per `ModelObject` from the lexicographic
  first transform group and shares it across every group of that object.
- `PrintObject.cpp:3555-3579` resolves object options and clamps only support
  selectors greater than logical filament count. Its sparse object-config call
  to monolithic `normalize_fdm` has no write-set intersection with the 126
  `PrintObjectConfig` fields; it is not a reason to add another dynamic object
  normalization shell.
- `PrintObject.cpp:3602-3709` defines the model-part precedence and feature
  filament fallback already represented by `RegionOptions::resolve`:
  process base, object, volume, optional material, then layer range, followed
  by selector/density/fuzzy normalization.
- `PrintApply.cpp:342-395` defines sorted continuous layer ranges, gap and
  overlap handling, `EPSILON`, the unconfigured tail, and range lookup.
- `PrintApply.cpp:548-553,595-660,886-945` combines print-object and volume
  transforms and admits a model volume to a multi-range Z slab only when a
  transformed triangle edge occupies that expanded slab.
- `PrintRegion.cpp:71-110` defines role-aware feature-filament collection and
  its print-wide brim input.
- `Model.cpp:2512-2564` defines raw object/volume `extruder` fallback and the
  model-volume types that may contribute it.
- `Print.cpp:451-546` composes region, raw volume, raw layer-range, support,
  custom-tool-change, and explicit wipe-tower filament sources, then sorts and
  removes duplicates.
- `Print.hpp:362-365,429-431` defines brim, support, and raft predicates.
- `Print.cpp:3290-3301,3385-3388` defines the bounded `has_wipe_tower` predicate
  used by explicit `wipe_tower_filament` participation.

The adjacent preset/UI sizing source remains cited only to explain deferral:
`PrintConfig.cpp:8765-8862`, `Preset.cpp:443-488`, and the preset construction
paths do not become project-path production calls.

## Rust destination and interface

The implementation uses the existing concrete owners and adds small modules
split by fixed source responsibility:

- `crates/ares-core/src/options/project_fdm_normalization.rs` for typed
  `normalize_fdm_1` and `normalize_fdm_2`.
- `crates/ares-core/src/options.rs` to register the new typed normalization
  module and expose only its crate-private project seam.
- `crates/ares-core/src/project/effective_config.rs` for the source-ordered
  orchestration and result ownership.
- Small siblings under `project/effective_config/` for layer normalization,
  Z-slab occupancy, and used-filament discovery when needed to keep every Rust
  file below 400 physical lines.
- `crates/ares-core/src/project.rs` to register and retain the crate-private
  effective-config boundary.
- `crates/ares-core/src/project/transform.rs` for crate-private
  `without_xy_translation`, fixed-data-order transform comparison, and
  cast-before-multiply `transform_z_f32` value transforms.
- `crates/ares-core/src/lib.rs` only for the compact
  `UnsupportedProjectFeature(String)` error variant and display text.
- `crates/ares-wasm/src/lib.rs` and its focused tests to keep the exhaustive
  `SliceError` to JavaScript string mapping compiling and stable.
- Focused option and project tests split along the same boundaries.

The crate-private shape is intentionally topological rather than flat:

```rust
pub(crate) struct BoundedResolvedProjectConfig {
    pub(crate) views: ProjectConfigViews,
    pub(crate) logical_filament_count: usize,
    pub(crate) usage: BoundedProjectUsage,
    pub(crate) print_object_count: usize,
    pub(crate) objects: Vec<ResolvedProjectObject>,
}

pub(crate) struct BoundedProjectUsage {
    pub(crate) supported_used_filaments: Vec<usize>,
    pub(crate) coverage: ProjectUsageCoverage,
}

pub(crate) enum ProjectUsageCoverage {
    TypedConfigSourcesOnly,
}

pub(crate) struct ResolvedProjectObject {
    pub(crate) object: ObjectOptions,
    pub(crate) print_objects: Vec<ResolvedPrintObjectConfig>,
    pub(crate) layer_candidates: Vec<ResolvedLayerCandidate>,
}

pub(crate) struct ResolvedPrintObjectConfig {
    pub(crate) transform: Transform3d,
}

pub(crate) struct ResolvedLayerCandidate {
    pub(crate) min_z: f64,
    pub(crate) max_z: f64,
    pub(crate) source_range_index: Option<usize>,
    pub(crate) model_parts: Vec<ResolvedModelPartCandidate>,
}

pub(crate) struct ResolvedModelPartCandidate {
    pub(crate) volume_index: usize,
    pub(crate) region: RegionOptions,
}
```

Equivalent names or private nesting are allowed when the approved plan finds a
smaller Rust expression, but the result must preserve object, ordered
print-object group, one shared normalized layer-candidate set per object, and
source-volume identity. Candidates must not be owned independently by each
transform group. A flat `Vec<RegionOptions>` that loses those associations is
not allowed.

`BoundedProjectUsage` is deliberately not convertible to a future complete
usage type. No final-G-code consumer may accept it as complete usage. A later
source-cited task must add the deferred owners, introduce a distinct complete
type/coverage state, and make completeness an exhaustive typed check before
usage drives arbitrary-project tool ordering or G-code.

Modifier candidates may be retained only as explicitly unresolved typed
identity. They must not be given an invented parent or silently counted as
effective feature regions.

## Exact typed normalization

### `normalize_fdm_1`

The function mutates a caller-owned typed `ProjectSettings` clone. It never
reads the project archive or reloads raw settings.

1. Global `extruder` is not a `ProjectSettings` field. That is intentional:
   project model `extruder` values live in the object/volume/layer sparse
   region owners and are handled by `RegionOptions::resolve`. No global erased
   value is introduced merely to mimic the dynamic container.
2. If `sparse_infill_filament_id` is positive and
   `internal_solid_filament_id` is zero, copy sparse into internal solid.
3. Snapshot internal-solid, top-surface, and bottom-surface selectors after
   the sparse rule. Use those three snapshots for all four remaining writes.
   Consequently, when the initial internal selector is zero and both top and
   bottom are positive, the later bottom write replaces the earlier top write
   to internal solid. Do not turn the source sequence into a mutually exclusive
   chain.
4. When `spiral_mode` is true:
   - preserve the current length of `retract_when_changing_layer` and replace
     every entry with false;
   - preserve the current length of
     `filament_retract_when_changing_layer` and replace every value, including
     nullable nil, with concrete false;
   - set `wall_loops=1`, `alternate_extra_wall=false`,
     `top_shell_layers=0`, and `sparse_infill_density=0%`.
5. Clamp `resolution` to at least `0.001`. Typed negative finite inputs also
   become `0.001`, matching `std::max`; this stage does not reuse the older
   dynamic helper's stricter parser.
6. Preserve every field outside this exact write set.

### `normalize_fdm_2`

Use a small typed changed-key enum or another compile-time representation. No
runtime string-key dispatch is allowed. The observable serialized names remain
`enable_prime_tower` and `independent_support_layer_height`.

1. `used_filaments == 0` is a no-op.
2. Smooth timelapse means `ProcessTimelapseType::Smooth`; wrapping means the
   typed `enable_wrapping_detection` boolean.
3. When timelapse is not smooth, wrapping is disabled, and either exactly one
   filament is used or sequence is by-object with more than one effective
   `PrintObject`, change a true `enable_prime_tower` to false and report only
   that key.
4. If the tower remains enabled, change a true
   `independent_support_layer_height` to false and report only that key.
5. Already-equal values are not reported. The function never re-enables either
   field.
6. The write set is exactly these two print-level fields and has zero
   intersection with the 126 object fields and 153 effective region fields.

The monolithic `normalize_fdm` is not called by the new project path. The
existing dynamic `SliceOptions` method remains only for the still-deferred
legacy/STL consumers and must not be called, expanded, or used as fallback.

## Source-ordered cold project orchestration

Ares project slicing is a stateless byte-in operation. Fixed Orca's caller
passes the same fresh full configuration through `Print::apply` twice: the
first call starts with no old objects/filaments and creates effective regions;
the second call normalizes another fresh copy using that state. Ares collapses
those two stateful applications into one pure final-state transform. It does
not collapse the first call by pretending the new project was old state.
The resolver therefore names and preserves the three normalization-input usage
phases plus the returned post-normalization projection instead of using one
ambiguous vector:

1. Clone `project.settings()` as the reusable unmaterialized source and run
   typed `normalize_fdm_1` on that clone. The original `ProjectSettings` is
   never mutated or reloaded.
2. Clone that normalized source for the first fixed apply and run the cold
   `normalize_fdm_2(0, 0)`. It must be an explicit no-op; passing counts from
   the new `Project` here is incorrect.
3. Materialize first-apply variants from that unmaterialized clone, resolve and
   validate cardinalities, sort/group printable instances by fixed transform
   order, and resolve effective ObjectOptions. Do not build region candidates
   yet: fixed Orca creates the new `PrintObject`s before the late call, but does
   not generate or attach their regions until afterward.
4. Compute **first-apply pre-region usage** for the included typed sources in
   the same phase as the first call: raw object/volume/layer selectors plus
   support, then the explicit wipe selector using the first materialized
   config. Do not add feature selectors from not-yet-created regions.
5. Run the first apply's late `normalize_fdm_2(print_object_count,
   first_apply_pre_region_usage.len())` on that materialized config. Preserve
   this result as the previous `m_config` equivalent used only by the second
   apply's early wipe-tower predicate.
6. Build the first preliminary normalized layer/model-part region candidates
   from the first late-normalized configuration, once per `ProjectObject` from
   its first sorted transform group and shared across that object's groups.
   Compute **second-apply early usage** from those regions plus raw and support
   sources, but evaluate explicit wipe participation against the
   previous-config equivalent from step 5. This matches the second call reading
   the regions and `m_config` left by the first call before it applies the fresh
   full configuration.
7. Clone the normalized unmaterialized source again, run
   `normalize_fdm_2(print_object_count, second_apply_early_usage.len())`, and
   only then materialize the second-apply variants from that fresh source.
   Never materialize from the first materialized, previous-config, or runtime
   result.
8. Rebuild each object's single shared preliminary candidate set from the
   second materialized configuration and the same representative-group rule.
   Compute **final pre-normalize usage**, now evaluating explicit wipe
   participation against that second materialized configuration.
9. Run the second apply's late
   `normalize_fdm_2(print_object_count, final_usage.len())`. Recompute supported
   usage against the resulting tower state and return that post-normalization
   vector. In the valid source-ordered double-apply lifecycle, the included
   predicates make it equal to `final_usage`: any reachable tower-disabling
   condition after second region construction was already visible to the
   second-apply early call. The recomputation still derives the returned
   projection from final state instead of relying on that invariant. Do not
   feed its length into another normalization call; fixed Orca performs no
   fallback or convergence loop.
10. Discard all preliminary projections and rebuild the final object and shared
    region candidates from the final materialized settings, again using only
    each object's first sorted transform group. Although the current two-key
    write set does not intersect object/region fields, this preserves the fixed
    ordering without stale-state assumptions.
11. Call `resolve_project_config_views` exactly once, after final normalization
    and rebuild. Its `full`, `runtime`, and `runtime_gcode` outputs must all
    derive from final state.
12. Return `BoundedResolvedProjectConfig` with only the stable final vector inside
    `BoundedProjectUsage::supported_used_filaments`, without exposing the three
    transient phase vectors or mutating `Project`.

Incremental diff/invalidation mechanics are not reproduced. The observable
final configuration of the fixed double-apply lifecycle is reproduced without
retaining mutable `Print` state. This collapse is valid for the included
boundary because variant materialization does not write either `_2` field and
the `_2` write set does not affect object/region usage; tests freeze both
non-intersections. The equivalence claim applies only after all detectable
deferred usage sources are rejected as specified and remains explicitly
bounded for sources the current loader does not retain.

## Cardinality and external-boundary validation

1. Physical extruder count is the raw/materialized
   `nozzle_diameter.len()` and must be non-zero.
2. Logical filament count is the materialized
   `filament_diameter.len()` and must be non-zero.
3. `filament_map.len()` must equal logical count. Every one-based map entry
   must name an existing physical extruder.
4. Every filament vector directly indexed by this resolver must cover logical
   count. This includes all four filament ironing vectors before
   `RegionOptions::resolve`; the already released retract-view transform keeps
   its own sixteen keyed cardinality checks.
5. Object and region selector normalization uses logical count, matching fixed
   `m_config.filament_diameter.size()`, not physical nozzle count. The current
   fixture has two of each, so tests must include unequal synthetic counts to
   prevent that mistake from passing accidentally.
6. Negative support selectors are rejected with their exact key at this
   untrusted project boundary. Existing source-supported selectors greater
   than logical count continue through `ObjectOptions::resolve` and clamp to
   one.
7. No validation may resize the raw 4-/8-stride variant families or synthesize
   a missing payload.
8. Raw object, volume, and layer-range `extruder` values must be in
   `0..=logical_count`. Zero retains the fixed fallback/ignore meaning for its
   scope; a negative or too-large value is rejected with `extruder`. This
   external-boundary rejection replaces the fixed C++ assumption that preset
   ingress already supplied a valid selector.
9. A non-zero `wipe_tower_filament` must satisfy both the fixed physical
   assertion boundary `selector < physical_count` and the logical output
   boundary `selector <= logical_count`. Reject either violation with
   `wipe_tower_filament`; do not return an out-of-range logical index.
10. `filament_shrink` and `filament_shrinkage_compensation_z` must each cover
    logical count and every active entry must be exactly 100% in this task.
    A non-100% value selects the explicitly deferred shrinkage/regrouping
    boundary and is rejected as unsupported with its concrete key rather than
    returning a misleading resolved usage result.

Invalid cardinalities/selectors are compact `SliceError::InvalidInput` values
naming the concrete Orca key. Add a distinct compact
`SliceError::UnsupportedProjectFeature(String)` for valid typed inputs whose
required source boundary is explicitly deferred. It also names the concrete
key/feature and must not contain an entire JSON document, XML document,
archive, or G-code buffer.

Its stable display/JavaScript form is
`unsupported project feature: {feature}`. Update the exhaustive
`ares-wasm::format_slice_error` match and freeze that exact mapping in a WASM
adapter unit test; do not add a wildcard arm that could hide later error
variants.

These checks are eager at Ares' untrusted complete-project boundary, including
when a fixed C++ assertion would only be reached by an active branch. This is
an intentional deterministic boundary divergence; it is not a fallback or a
license to resize/correct the supplied values.

## Layer candidates and minimal Z occupancy

Promote the existing staged `LayerRanges` behavior into the production project
resolver rather than writing a different algorithm.

1. Consume the already sorted raw `LayerConfigRange` sequence without mutating
   it. Use `EPSILON = 1e-4` and `last_z = 0`.
2. Skip a range whose end is not greater than `last_z`.
3. Clamp its start to zero. Insert an unconfigured gap only when the clamped
   start is greater than `last_z + EPSILON`.
4. Insert the configured interval only when its end is greater than
   `last_z + EPSILON`. Earlier lexicographic ranges trim later overlaps.
5. Empty output becomes `[0, f64::MAX]` with no source config. An unconfigured
   final interval extends to `f64::MAX`; otherwise append an unconfigured tail.
6. Range lookup subtracts `EPSILON` from both requested bounds and accepts a
   match only when both final bound differences are at most `EPSILON`.

The intervals are configuration candidates, not sliced geometry. For exact
participation of the included typed region sources, do not create the Cartesian
product of every model part and every interval:

1. If normalization yields one interval, preserve fixed Orca's single-range
   special case and admit each non-empty ModelPart without a Z-slab test.
2. For multiple intervals, expand each interval by `EPSILON` at both ends. In
   Ares terms, form
   `print_object_without_xy.then(volume.transform())`, which is the fixed
   matrix product `object_trafo * volume_trafo` and therefore applies the
   volume transform to a vertex first, then the print-object transform. Clear
   the combined XY translation before the numeric cast, matching
   `trafo_for_bbox`; this does not alter Z but removes any order ambiguity.
3. Match the fixed numeric boundary: cast the combined transform and mesh
   vertex coordinates to `f32` for the transformed edge Z values, then compare
   them with the expanded range bounds. Do not silently substitute an all-`f64`
   predicate at this source boundary.
   `Transform3d::transform_z_f32(Point3d)` must cast each combined-matrix
   coefficient and each point coordinate to `f32` before the multiply/add
   expression. Calling the existing f64 `transform_point` and casting its
   result afterward is observably different and forbidden.
4. For each triangle edge, sort endpoints by Z. The edge does not occupy a
   slab when `upper.z <= slab.min` or `lower.z >= slab.max`; any other edge
   makes that ModelPart occupy the interval. Only occupied ModelParts receive
   an effective region candidate for that interval.
5. This helper returns only occupancy. It must not compute or retain XY
   bounding boxes, polygon intersections, modifier parents, or sliced meshes.
6. Raw positive layer-range `extruder` remains an unconditional used-filament
   source, because fixed `Print::object_extruders` scans it independently of
   geometry occupancy.

Cold project resolution uses the project instance/volume transforms. Non-cold
incremental shrinkage-driven regrouping and region invalidation remain
deferred; the committed fixture's two typed shrinkage vectors are both 100%.

## Object, region, and print-object-group resolution

1. Call the new crate-private
   `instance.transform().without_xy_translation()` and group each object's
   printable instances by exact complete returned transform. Non-printable
   instances produce no PrintObject. Preserve groups from different
   `ProjectObject`s separately.
2. Sort and deduplicate each object's groups with fixed
   `transform3d_lower`/`transform3d_equal` semantics. Scan all 16 coefficients
   in Orca/Eigen `Transform3d::data()` order, which is column by column when
   indexing Ares' row/column matrix. At the first differing coefficient, use
   ordinary finite `<` and `>` comparisons; signed zero is equal and must not
   be separated with `f64::total_cmp`. The resulting order is independent of
   input instance order.
3. Effective print-object count is the total number of those groups, not raw
   project-object count and not raw printable-instance count.
4. Resolve `ObjectOptions` from the final process object base and the owning
   object's sparse overrides, using logical count for support clamps.
5. For each `ProjectObject` with at least one group, use only the first sorted
   group's transform as the representative transform for Z occupancy and
   region generation. Build one layer/model-part candidate set and share that
   set across every transform group of the object. Never union candidates from
   independently evaluating the other groups; groups belonging to different
   `ProjectObject`s never share candidates.
6. For each occupied ModelPart candidate, resolve `RegionOptions` in this
   order: final process region base, object region overrides, volume region
   overrides, no project material override, then the candidate layer override.
7. Preserve the source volume index in every candidate.
8. Negative volumes, support enforcers, and support blockers do not receive a
   printable region candidate.
9. Parameter modifiers require parent-region and XY bounding-box intersection.
   Do not invent a parent. Raw modifier `extruder` may participate as described
   below, but any modifier feature-region result remains explicitly unresolved
   and outside the claimed used-feature set.
   If a modifier override can change used-filament discovery
   (`wall_loops`, `sparse_infill_density`, `top_shell_layers`,
   `bottom_shell_layers`, or any of the six feature filament selectors), return
   `UnsupportedProjectFeature` naming the first such key. This prevents a
   bounded result from silently under-counting a detectable deferred source.
10. Fixed generic 3MF supplies no project material-config document. The project
   path passes `None` to the existing optional material precedence slot; pure
   typed material-precedence tests remain valid.

## Supported used-filament discovery

Each named phase produces a sorted/deduplicated bounded vector of zero-based
logical filament indices according to the composition points below. Only the
stable final vector is returned, inside `BoundedProjectUsage`; one-based
selectors are validated/normalized before conversion.

Evaluate region, raw model/layer, brim, and support sources only for
`ProjectObject`s that produced at least one printable PrintObject transform
group. A domain object with no printable instance must not contribute merely
because its volumes/configuration were loaded.

Feature-role discovery traverses each qualifying object's shared candidate set;
additional transform groups do not create or union additional regions. Raw and
support sources remain group-presence gated, with duplicates removed at the
fixed composition points below.

### Effective region roles

For occupied ModelPart candidates, match the fixed `PrintRegion` feature-role
predicates exactly, using the bounded supported brim input below:

- outer wall participates when `wall_loops > 0` or the print-wide brim flag is
  true;
- inner wall participates when `wall_loops > 1`;
- sparse infill participates when density is positive;
- internal solid participates when density is positive or either top/bottom
  shell count is positive;
- top surface participates when top shell count is positive;
- bottom surface participates when bottom shell count is positive.

The print-wide brim flag is true when any effective object has the supported
fixed config-driven brim condition and no raft. `AutoBrim` participates
regardless of width; any non-`NoBrim` type participates when width is positive.
When `Painted` has zero width, Ares cannot evaluate the separate painted-point
branch because it does not yet own painted brim points. Return
`UnsupportedProjectFeature("brim_type")` instead of guessing false.

### Raw model and layer selectors

For `ModelPart` and `ParameterModifier` volumes only, reproduce
`ModelVolume::extruder_id` fallback:

1. Use a non-zero volume `extruder` when present.
2. Otherwise use the owning object's `extruder` when present.
3. When both are absent, use one.
4. Add only a positive final value.

Negative volumes, support blockers, and support enforcers do not contribute.
Positive raw layer-range `extruder` values contribute independently of Z
occupancy. Painted-facet additions are explicitly deferred.

After composing effective region roles with these raw model/layer sources,
sort and deduplicate this object-extruder vector exactly like
`Print::object_extruders`.

### Support and raft

An object uses support material when `enable_support` is true,
`enforce_support_layers > 0`, or `raft_layers > 0`.

- A positive effective `support_filament` or
  `support_interface_filament` contributes its zero-based index.
- A zero selector means current object filament. If any supported object uses
  zero, append the bounded object-extruder set after scanning all objects.
- Sort and deduplicate the support vector independently, matching
  `Print::support_material_extruders`.

### Explicit wipe-tower filament

Port only the bounded fixed participation rule, not tool ordering:

1. Concatenate the already deduplicated object and support vectors before this
   check; duplicates between those two vectors are still present at this point.
2. `has_wipe_tower` requires preliminary `enable_prime_tower` and then
   either wrapping with more than two exclusion points, smooth timelapse, or
   non-spiral mode with more than one logical filament.
3. Add non-zero `wipe_tower_filament` only when the already collected vector
   contains more than one entry before final deduplication, matching the fixed
   call order.
4. Validate both `0 < selector < physical nozzle count` (the fixed assertion)
   and `selector <= logical filament count` (the bounded result invariant)
   instead of reproducing a C++ assertion or returning an invalid index.
5. Sort and deduplicate the final composed vector only after this optional
   addition.

## Deliberately unsupported used-filament sources

Task 19B.3 must not claim complete Orca used-filament discovery. The following
fixed sources have no complete current typed project owner or require deferred
geometry:

- MMU/painted facet extruders on `ModelVolume`;
- painted brim points used by zero-width `Painted` brim;
- per-plate custom G-code `ToolChange` items;
- feature-specific modifier regions, which require parent chaining and XY
  bounding-box intersection;
- project-supplied material configuration, for which the fixed generic 3MF
  reader has no material-config document;
- painted/fuzzy region construction and region deduplication;
- wipe-tower tool ordering beyond the explicit configured selector above.

The committed KSR project contains none of those sources. Production code may
not fabricate empty owners, infer them from the reference G-code, or label this
bounded result as universally complete. Later source-cited slices must add the
missing owners before using them for arbitrary-project final G-code parity.
Detectable deferred cases described above (non-100% shrinkage, zero-width
painted brim, and usage-affecting modifier overrides) return
`UnsupportedProjectFeature`; the typed coverage marker remains necessary for
painted/custom sources the current loader does not retain and therefore cannot
yet reject from domain state.

## Required TDD behavior

Each implementation slice begins with a genuine RED caused by missing
production behavior, reaches focused GREEN, and receives independent review
before the next dependent slice.

1. **Typed `normalize_fdm_1`**
   - sparse/internal/top/bottom propagation and the snapshot overwrite case;
   - spiral ordinary and nullable vectors preserve cardinality and become
     concrete false;
   - exact wall/shell/infill writes;
   - resolution below/equal/above `0.001`, including a finite negative input;
   - fields outside the write set remain unchanged.
2. **Typed `normalize_fdm_2`**
   - used counts zero, one, and many;
   - by-layer versus by-object with one and multiple PrintObjects;
   - traditional versus smooth timelapse;
   - wrapping disabled/enabled;
   - already-false/already-disabled values and exact changed-key behavior;
   - no reverse re-enable and exact two-field write-set proof.
3. **Cardinality boundary**
   - unequal physical and logical counts prove selector clamps use logical
     count;
   - empty nozzle, empty logical filament vector, invalid map entry, map/diameter
     mismatch, and each short resolver-indexed ironing vector name their key;
   - negative and too-large raw object, volume, and layer `extruder` values
     each reject with `extruder`, while zero preserves its scope-specific
     fallback/ignore meaning;
   - wipe selector tests distinguish its strict physical assertion bound from
     the separate logical-index bound under unequal counts;
   - short shrink vectors name their key and any active non-100% shrink value
     returns `UnsupportedProjectFeature`;
   - raw four-/eight-stride sentinels remain unchanged before Task 19B.1A
     selection, proving no `set_num_*` path ran.
4. **Layer normalization and lookup**
   - empty, negative, reversed, overlapping, gapped, exact-boundary, tiny-gap,
     and unconfigured-tail cases freeze the fixed algorithm;
   - raw `LayerConfigRange` values remain unchanged;
   - old dead staged layer-range tests are deleted after equivalent production
     tests are green.
5. **Print-object grouping and Z occupancy**
   - XY-only translation differences group together;
   - rotation, scale, or Z translation differences remain separate;
   - column-major 16-scalar lexicographic ordering is independent of input
     instance order, and signed-zero transforms group together;
   - non-printable instances are excluded;
   - one `ProjectObject` with two Z-distinct groups is supplied in reverse input
     order and its layer ranges use distinct feature-filament sentinels; only
     the lexicographic first group's Z occupancy contributes candidates, the
     object owns one candidate set shared by both group records, and
     used-filament/`_2` counts exclude the other group's selector;
   - groups from different `ProjectObject`s retain separate candidate sets;
   - `without_xy_translation` is a value transform and matrix composition is
     exactly object-without-XY times volume, not the reverse;
   - a precision sentinel where cast-before-multiply differs from
     f64-multiply-then-cast freezes `transform_z_f32`'s numeric order;
   - single-range admission and multi-range strict edge predicates are distinct;
   - a completely non-intersecting layer feature selector cannot inflate the
     used set, while a positive raw layer `extruder` still participates
     independently.
6. **Object/model-part candidates**
   - object support clamps use logical count;
   - object -> volume -> layer precedence is visible with unique sentinels;
   - source volume identity and normalized interval identity are preserved;
   - project material remains `None` while existing pure material precedence
     stays green;
   - modifier feature config is not assigned an invented parent, and every
     detectable usage-affecting modifier key returns the typed unsupported
     error;
   - zero-width painted brim returns typed unsupported instead of guessing its
     missing painted-point state.
7. **Used-filament composition**
   - every role predicate independently includes/excludes its selector;
   - config-driven print-wide brim and raft suppression;
   - raw object/volume fallback for every relevant volume type;
   - a configured object with no printable transform group contributes no raw,
     region, brim, or support source;
   - raw layer `extruder` remains independent of occupancy;
   - support zero/current and positive selectors;
   - explicit wipe-tower predicate and pre-dedup `len > 1` gate;
   - final sorted deduplication.
8. **Stage order and committed fixture**
   - spiral retract sentinels prove normalization happens before variant
     materialization and materialization reads the normalized source;
   - distinct sentinels prove first-apply pre-region usage excludes feature
     regions that have not yet been generated, second-apply early usage includes
     the first-call regions while its wipe predicate reads previous-config
     tower state, and final pre-normalize wipe participation reads the second
     materialized state;
   - a by-object/multiple-PrintObject case proves the second-apply early `_2`
     disables the tower before second materialization, final pre-normalize
     usage therefore excludes the explicit wipe selector, and the recomputed
     post-normalization vector remains equal without running `_2` again;
   - all preliminary candidates are discarded and views are built only from
     final state;
   - loading the committed 3MF yields logical count two, one effective
     PrintObject group, one implicit `[0, f64::MAX]` layer candidate, the sole
     ModelPart, and used set `[0]`;
   - the final full view has `enable_prime_tower=false` and
     `independent_support_layer_height=true` solely from typed 3MF inputs;
   - `resolution` remains `0.012`, raw project settings remain unchanged, and
     the already approved full/runtime retract distinctions remain intact;
   - the result exposes `[0]` only through `BoundedProjectUsage` with
     `TypedConfigSourcesOnly`, and no complete-usage conversion exists.
9. **Production call boundary**
   - `slice_project` calls the resolver after `load_project`;
   - a synthetic keyed cardinality failure returns before
     `ProjectSlicingIncomplete`, proving the call is not dead;
   - the valid committed fixture still returns exactly
     `ProjectSlicingIncomplete` until a later slicing consumer exists;
   - CLI and browser byte-oriented paths retain the same boundary;
   - `UnsupportedProjectFeature("filament_shrink")` formats exactly as
     `unsupported project feature: filament_shrink` through core Display and
     the exhaustive WASM JavaScript adapter match.

Tests may read the committed 3MF input. They must not read the reference G-code
to derive configuration expectations, invoke Orca, inspect a mutable Orca
checkout at runtime, or branch production behavior on fixture path, name,
hash, size, or values.

## Obsolete scaffold replacement

Once the production equivalents are green, delete only the staged structures
they replace:

- `print_apply/apply_normalization_state.rs` and its call-recording tests;
- the dead `LayerConfigRangeInput`, `NormalizedLayerRange`,
  `normalize_layer_ranges`, and `layer_range_config_id` shell in
  `print_apply.rs` plus their duplicate staged tests;
- obsolete dead-code allowances on `project_variants` and
  `project_config_views` after the production caller makes them reachable.

Do not delete unrelated behavior-bearing `print_apply` staging modules or the
dynamic `SliceOptions` path still used by the legacy/STL pipeline. No committed
test may read or pin Orca source text; fixed-source inspection is review
evidence only.

## Explicitly deferred

- All unsupported used-filament sources listed above.
- Incremental `Print::apply` diff, invalidation, cache reuse, and GUI
  double-apply mechanics.
- Non-cold shrinkage-driven print-object/regional regrouping.
- Modifier parent discovery, XY intersection, negative-volume geometry,
  painted/fuzzy regions, and region deduplication.
- `set_num_extruders`, `set_num_filaments`, `get_parameter_size`, and
  `extend_extruder_variant` preset/UI behavior.
- Config-block serialization (Task 19C).
- Dynamic `SliceOptions` consumer migration/removal (Tasks 20A-20E).
- Geometry slicing, toolpaths, G-code generation, metadata, post-processing,
  and final golden parity.

## Architecture and platform constraints

- `ares-core` remains byte/in-memory only and portable across browser WASM,
  Windows, macOS, and Linux.
- New production code contains no `serde_json::Value`, JSON map, raw JSON,
  erased option enum, `BTreeMap<String, _>`, runtime string-key dispatch,
  filesystem, terminal, process, clock, OpenGL, FFI, or native-only API.
- Existing concrete typed fields and monomorphized helpers are used directly.
- Every new or changed Rust source file remains below 400 physical lines and is
  split by fixed source responsibility when needed.
- No legacy fallback, fixture hardcoding, source-tree dependency, or new crate
  or dependency is allowed.
- Both committed fixture files remain byte-for-byte unchanged.

## Approval, documentation, and release gates

1. Freeze this spec and obtain literal `VERDICT: APPROVE` from fresh
   independent review sessions, including OpenCode. Any spec edit invalidates
   all spec approvals.
2. Write the detailed Superpowers Subagent-Driven TDD plan and obtain literal
   `VERDICT: APPROVE` from fresh independent review sessions, including
   OpenCode. No production or test implementation begins before the plan is
   approved.
3. Execute the bounded implementation slices with subagent implementers and
   independent reviewers. Verify every claimed RED/GREEN result in the shared
   workspace.
4. Freeze the implementation manifest. Obtain literal `VERDICT: APPROVE` from
   an independent whole-spec compliance reviewer, a separate code-quality
   reviewer, and OpenCode. Any production/test edit invalidates all final
   implementation approvals.
5. Only after implementation approval, update
   `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, and the ignored
   SDD progress ledger. Freeze and independently approve the docs-only diff.
6. Run focused and adjacent tests, full workspace Nextest, rustfmt,
   warning-denying Clippy, native/WASM checks, release WASM, `wasm-bindgen`
   browser tests, dynamic-value audit, fixture hashes, no-hardcoding and
   source-pinning scans, per-file LOC checks, and frozen-manifest equality.
7. Stage only the frozen manifest, use an approved Conventional Commits
   message, push the current branch, and require all five Tier 1 jobs green for
   the exact pushed SHA before declaring Task 19B.3 complete.

Task 19B.3 completion will not complete the persistent `ksr_fdmtest_v4`
slicing goal; Task 19C and later source-cited geometry/G-code slices remain.
