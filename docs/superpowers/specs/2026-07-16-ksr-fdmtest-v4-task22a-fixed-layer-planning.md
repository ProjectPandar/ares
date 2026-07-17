# Task 22A: Typed Project Slicing Parameters and Fixed Layer Planning

## Status and objective

This specification is a draft. No implementation plan, test edit, or
production change may begin until these exact bytes receive independent Codex
and default-model OpenCode approvals.

Task 22A is the first post-configuration slice of the persistent
`ksr_fdmtest_v4` parity program after released commit
`4281e913b8eeaaeb6111cbefdf06f896f5c611aa` (exact-SHA Tier 1 run
`29520118127`). It consumes the already loaded project, the released bounded
effective typed configuration, and the real transformed project mesh to build
Orca-compatible one-dimensional object layer plans.

For the committed project, this task must plan 460 real layers from Z 0 through
92 mm using `initial_layer_print_height=0.2` and `layer_height=0.2`. It does not
slice a triangle at those Z values, construct XY paths, or emit successful
G-code. A valid project still returns `SliceError::ProjectSlicingIncomplete`,
but only after its private layer plan has been produced. No approximate or
placeholder G-code becomes observable.

### Why this is not Task 20A.3 profile wiring

The committed 3MF has 15 archive entries and no embedded process, machine, or
filament preset fragment. Its sole flattened full project-settings snapshot is
the 653-key `Metadata/project_settings.config`; sparse object/volume/range
overrides remain in the option-bearing model metadata documents. Profile IDs
are labels, not enough information to reconstruct an inheritance chain.
`load_project` already deserializes the full snapshot directly into
`ProjectSettings`, and the current project path already produces the exact
49,004-byte reference config block.

Task 22A therefore must not discover external presets, call
`compose_profile_fragments`, or infer profile data from IDs. Those actions would
violate the requirement that project slicing use only information present in
the 3MF and would not advance the committed fixture. Remaining legacy/dynamic
consumer cleanup may proceed in separate debt tasks, but it is not a substitute
for this fixture-observable slicing boundary.

### Pre-implementation review contract

The independent approvals at the end of this document are design reviews.
Reviewers must judge source fidelity, completeness of the declared subset,
typed ownership, TDD observability, WASM portability, and the honesty of every
deferral. Missing Task 22A types and tests in the current tree are the expected
pre-implementation state, not a review defect.

A `REVISE` verdict must identify a specification defect: an inaccurate source
claim, missing required behavior, unsafe or ambiguous ownership, an
unimplementable requirement, a hidden fallback, or acceptance criteria that
could not distinguish a wrong implementation.

## Fixed upstream rewrite boundary

All upstream citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/Slicing.hpp:25-38,44-52,66-85,98-114` defines the POD
  `SlicingParameters` fields and the object-height/first-layer identities used
  by this task.
- `src/libslic3r/Slicing.cpp:24-43` defines nozzle-derived minimum and maximum
  layer heights: configured zero minimum becomes 0.07, a configured minimum is
  clamped to at least 0.01, configured zero maximum becomes 75% of nozzle
  diameter, and maximum never falls below minimum.
- `Slicing.cpp:62-70,106-146,228-229` creates the no-support/no-raft parameter
  subset: nonpositive initial height falls back to object `layer_height`, object
  Z begins at zero, object height is recorded, object-extruder nozzle limits are
  accumulated, and regular layer height expands the resulting bounds when
  necessary.
- `Slicing.cpp:232-304::layer_height_profile_from_ranges` defines the fixed
  empty-height-range profile used here. The fixed first layer is inserted and
  the uncovered interval to object top uses regular `layer_height`; adjacent
  equal-height points are compressed with strict `is_approx` semantics.
- `Slicing.cpp:713-804::adjust_layer_series_to_align_object_height` defines the
  complete `precise_z_height` adjustment, but Task 22A deliberately gates it
  off as deferred behavior rather than partially reproducing it.
- `Slicing.cpp:807-866::generate_object_layers` produces bottom/top pairs. It
  installs the fixed first layer, samples the height profile at the next slice
  position, stops once the next midpoint reaches object top, and does not force
  the final top to equal object height when `precise_z_height` is false.
- `src/libslic3r/PrintObject.cpp:3732-3780` connects the effective print/object
  configuration, `ModelObject::max_z()`, occupied range-derived region feature
  configurations, and object printing extruders to `SlicingParameters`.
- `src/libslic3r/Model.cpp:1460-1499::ModelObject::{min_z,max_z,
  update_min_max_z}` composes the source object's first instance transform with
  each model-part volume transform and derives bounds from every mesh vertex;
  triangle indices are not consulted.
- `src/libslic3r/PrintRegion.cpp:71-109::collect_object_printing_extruders`
  defines the six feature gates, print-wide brim contribution, one-based
  selector normalization, and region-selector clamp to the first extruder.
- `src/libslic3r/PrintObject.cpp:3783-3802::object_extruders` additionally
  includes raw volume extruders and returns sorted, unique zero-based IDs.
- `src/libslic3r/PrintApply.cpp:1015-1054` creates printing regions when an
  overlapping parameter modifier changes its parent region configuration, and
  `PrintObject.cpp:3683-3686` applies those modifier options. Because Task 22A
  defers modifier geometry/region construction, it conservatively rejects a
  typed true modifier `zaa_enabled` presence instead of losing that behavior.
- `src/libslic3r/Config.hpp:624-628::ConfigOptionVector::get_at` returns the
  first vector value for every out-of-range index and asserts that the vector is
  nonempty. Together with
  `Slicing.cpp:29-42,135-143`, this fixes the intentionally retained helper
  indexing behavior described below; this path does not read `filament_map`.
  Task 22A extends bounded typed validation to enforce that nonempty precondition
  before config writing or planning.
- `PrintObject.cpp:3805-3833` chooses the range-derived layer-height profile.
  Task 22A includes only the behavior when no explicit variable/adaptive height
  profile changes the fixed first/regular heights.
- `src/libslic3r/PrintObjectSlice.cpp:24-48` computes the slice Z. With ZAA
  disabled it is exactly the midpoint of each bottom/top pair.
- `PrintObjectSlice.cpp:50-73` creates layers in pair order: IDs begin after raft
  layers, height is `hi-lo`, print Z is `hi + object_print_z_min`, and slice Z
  remains in object-local coordinates.
- `PrintObjectSlice.cpp:817-830` fixes the lifecycle order: update the layer
  profile, create layers, and only then slice volumes.
- `src/libslic3r/PrintApply.cpp:104-167,1525-1621` and the already released Ares
  transform grouping establish one print object per unique printable instance
  transform after removing XY translation. Task 22A preserves that source/group
  order and does not collapse distinct groups; as upstream does, every group
  for one source object uses the object height derived from that source
  object's first instance.
- `src/libslic3r/Format/bbs_3mf.cpp:209-216,1896-1903,2087-2095,
  2824-2881` defines the case-insensitive project archive entry for painted
  layer-height profiles and attaches decoded profiles/ranges to model objects.
  Task 22A detects these typed input presences and rejects their deferred
  variable-height behavior instead of silently replacing it with fixed layers.
- `src/libslic3r/libslic3r.h:46,48-60,300-310` fixes `coordf_t` as `double`,
  `EPSILON` as `1e-4`, linear interpolation, and strict approximate comparison.

This is a source-cited Rust rewrite of the listed `libslic3r` behavior. It is
not an extension of the existing Ares STL `planning.rs` pipeline.

## Included supported subset

Task 22A accepts a resolved print object only when all of the following are
true:

1. The archive has no case-insensitive
   `Metadata/layer_heights_profile.txt` entry.
2. No layer configuration range contains an object-owned `layer_height`
   option.
3. `raft_layers == 0`.
4. `enable_support == false` and `enforce_support_layers == 0`.
5. `precise_z_height == false`.
6. Every resolved candidate printing region has `zaa_enabled == false`, and no
   parameter-modifier volume has a typed true `zaa_enabled` override.
7. `min_layer_height` and `max_layer_height` typed vectors are both nonempty.
8. The released project validation has accepted identity XY/Z filament
   shrinkage (`100%`) and valid selector/cardinality state.
9. Layer planning therefore uses the fixed first/regular height profile; no
   decoded adaptive or painted variable-height values enter this task.

These are capability gates, not default substitutions. Before calculating any
bounds, Task 22A applies **project-wide key-major precedence**: for each key in
the fixed order `layer_height_profile`, `layer_height`, `raft_layers`,
`enable_support`, `enforce_support_layers`, `precise_z_height`, `zaa_enabled`,
evaluate the project-global profile presence once for the first key; for every
remaining key, scan every participating source object in final resolved order
and then its typed sources in stable source order. Return
`SliceError::UnsupportedProjectFeature` for the first key found anywhere in the
project. Thus object 1 `layer_height` wins over object 0 `raft_layers`; this is
not object-major traversal. It must not silently plan the project as if any
requested option were disabled.

For the final `zaa_enabled` key, typed sources are all resolved candidate
`RegionOptions` plus every parameter-modifier volume's raw typed override. Any
raw true modifier value is rejected, even if its geometry would ultimately not
overlap a model part. This conservative supported-subset restriction is an
explicit divergence needed until modifier region assignment is implemented;
raw false or absent values do not trigger it.

Occupied layer-range feature overrides already supported by the project reader
remain eligible through resolved candidate regions. A bare range `extruder`
continues to participate in the released print-wide bounded materialization
usage, but does not enter Task 22A's per-object nozzle limits when no resolved
region/volume uses it. Task 22A adds only the typed presence codec for the exact
object-owned `layer_height` key; it does not decode or apply variable-height
values. The committed fixture has neither that key nor a painted profile entry.

## Required typed state and ownership

### Stable source-object identity

`ResolvedProjectObject` must carry its original `source_object_index` from
`ProjectObjectTransformGroups`. This is necessary because source objects with
no printable instance are filtered before final resolution. Looking up a mesh
by the resolved vector position would be fixture-dependent and incorrect.

The index is crate-private, is copied in both shell and candidate resolution,
and remains unchanged through every effective-config pass. It is not serialized
or added to a public API.

### Nozzle option vector precondition

The bounded materialized-project validator must reject empty
`min_layer_height` and `max_layer_height` vectors with its existing keyed
`InvalidInput` form. Validation order becomes `nozzle_diameter`,
`min_layer_height`, `max_layer_height`, `filament_diameter`, `filament_map`, then
the existing remaining cardinality/shrinkage checks. Vectors shorter than the
selected helper index remain valid because upstream `get_at` deliberately uses
their first value; only empty vectors violate the helper contract.

This validation belongs to effective-config resolution, before Bambu config
block writing and before every Task 22A capability/planning error. Planning must
not invent a default vector member or treat emptiness as first-value fallback.

### Deferred variable-height presence

`LayerConfigRange` must retain `layer_height: Option<OrcaFloat>` separately
from its existing `RegionOptionOverrides`. The exact `layer_height` key is
parsed with the existing typed numeric codec; if the same key occurs more than
once in one range, the later value wins, while the existing duplicate-range
replacement and sort behavior remain unchanged. A crate-private accessor is
the only new exposure. Invalid numeric text remains a bounded project-reader
`InvalidInput` and is not converted into an unsupported-feature result.

`ProjectDocuments` must retain whether the archive contains any path equal to
`Metadata/layer_heights_profile.txt` under ASCII case-insensitive comparison,
with a crate-private `Project` accessor. Task 22A needs only typed presence, so
it must not parse, serialize, preserve raw bytes from, or interpret the profile
payload. Any matching entry, including an empty one or a differently cased
one, is rejected as `UnsupportedProjectFeature("layer_height_profile")` before
fixed planning. This conservative presence gate prevents deferred data from
being discarded as a fixed-height project.

### Planned print objects and layers

Task 22A introduces private typed state equivalent to:

```rust
struct PlannedLayer {
    id: usize,
    height: f64,
    print_z: f64,
    slice_z: f64,
}

struct PlannedPrintObject {
    source_object_index: usize,
    transform_index: usize,
    layers: Vec<PlannedLayer>,
}
```

Names and nesting may differ, but the four layer values and the two stable
object identities must be represented as typed fields. No layer value may be
stored in JSON, a string-key map, or a fixture-specific table.

The private project slicing state owns or borrows the loaded `Project`, the
single `BoundedResolvedProjectConfig`, the already generated config block, and
the planned print objects without rebuilding any of them from serialized
bytes. It is not returned from `slice_project` and is not exposed through CLI,
WASM, or a new public partial-success API.

## Required planning semantics

### Print-object enumeration and bounds

For each `ResolvedProjectObject` in final resolved order, enumerate every
`ResolvedPrintObjectConfig` in its existing transform order. Produce exactly
one `PlannedPrintObject` per entry. A source object with no printable transform
produces none.

Compute one source-faithful bound per participating source object and reuse it
for each of that object's planned transform groups:

1. Select the source `ProjectObject` by `source_object_index`.
2. Use the source object's first instance transform, whether or not that
   instance itself is the representative printable member of the current
   group. Do not substitute a grouped print-object transform when calculating
   `ModelObject::max_z()` semantics.
3. Consider only `ProjectVolumeType::ModelPart` volumes. Negative volumes,
   parameter modifiers, support enforcers, and support blockers do not define
   object height.
4. Compose the first source-instance transform with each model-part volume
   transform in matrix order `instance * volume`, matching the existing
   `Transform3d::then` convention.
5. Transform every vertex-array entry in `f64` and derive finite minimum and
   maximum Z samples. Triangle indices are never consulted, so an unreferenced
   vertex contributes and a model-part mesh does not need a triangle to
   contribute.
6. If there is no finite model-part vertex sample, any transformed value is
   nonfinite, or the resulting maximum Z is nonpositive, return a bounded
   `SliceError::InvalidInput` naming project-object Z bounds.
7. Use the transformed maximum Z directly as `object_height`, exactly as the
   cited `ModelObject::max_z()` handoff does. A negative or materially nonzero
   minimum Z is neither rejected nor subtracted, rounded, normalized, or used
   to translate the object in this task.

The implementation must not use the committed fixture's object ordinal,
height, transform, vertex count, filename, or expected layer count in
production.

### Object extruders and nozzle limits

Derive one sorted, deduplicated vector of zero-based object-extruder IDs per
participating source object. The complete source set is:

1. Every concrete `RegionOptions` in its resolved layer candidates contributes
   the six wall/infill/top/bottom selectors under the cited feature gates.
   `has_brim` is print-wide, as in `Print::has_brim`: if any qualifying resolved
   print object has supported brim, the outer-wall selector is considered in
   every region.
2. For every source model-part or parameter-modifier volume, an explicit
   positive volume `extruder` selector wins; otherwise use the source-object
   selector, with absent object and volume selectors defaulting to one. An
   explicit zero does not become a positive raw source.

Region feature selectors use Orca's normalization: subtract one after clamping
the one-based selector at zero, then clamp a result outside the logical
extruder count to zero. Positive raw selectors use their existing source
semantics and become zero-based by subtracting one. Sort and deduplicate only
after both source families have contributed. The committed 3MF's sparse
object-level `extruder=1` therefore participates through typed model metadata,
not through profile reconstruction.

A layer range's bare raw `extruder` is never appended directly as a third
source for these per-object nozzle limits. Its existing print-wide
bounded-usage/materialization effect remains unchanged. When the range is
occupied, existing `RegionOptions` resolution may use that generic selector as
a fallback for the six concrete feature selectors, which then participate only
under their feature gates. A nonintersecting range has no concrete candidate
region and therefore cannot affect per-object nozzle limits.

Do not read or apply `filament_map` in this calculation. The selected values
are passed as zero-based object-extruder IDs to the cited Orca nozzle helpers,
which nevertheless look up each option vector at `object_extruder_id - 1`.
Task 22A must reproduce that source behavior deliberately without unsigned
overflow: ID 0 underflows conceptually and selects the first value through
`get_at`; ID 1 selects index 0; ID 2 selects index 1; and any resulting index
outside a vector also selects its first value. Empty object-extruder input calls
the same helpers with ID 0. Each typed option vector is nonempty by the Task 22A
bounded validation precondition.

For every selected object-extruder ID under those lookup semantics:

- minimum is 0.07 when configured `min_layer_height` is zero, otherwise
  `max(0.01, configured_min)`;
- maximum is `0.75 * nozzle_diameter` when configured `max_layer_height` is
  zero, otherwise the configured maximum;
- maximum is then at least that nozzle's effective minimum.

Accumulate object minimum with `max` and object maximum with `min`, then clamp
the pair outward around regular `layer_height` exactly as upstream: final
minimum is `min(accumulated_min, layer_height)` and final maximum is
`max(accumulated_max, layer_height)`. First-value fallback is source behavior,
not fixture repair; it applies uniformly to zero, one, and out-of-range helper
indices. All values and vector cardinalities come from resolved typed
configuration.

### Slicing parameters

For the supported subset:

- `initial_layer_print_height > 0` is the first print/object layer height;
  otherwise `ObjectOptions::layer_height` is used.
- `layer_height`, the chosen first height, and object height must be finite and
  positive. The first height may exceed object height; layer generation still
  follows upstream stopping behavior rather than inventing an extra layer.
- `object_print_z_min` is zero, `object_print_z_max` is object height, and
  shrinkage Z is identity by the released validation invariant.
- raft counts/heights/gaps stay zero and the first object layer is fixed.
- the effective nozzle minimum/maximum values above are retained in the private
  parameters for generation and later mesh slicing.

### Bounded public-input generation

The source loop assumes sane desktop-slicer configuration, but `slice_project`
accepts untrusted bytes and must remain bounded on WASM and native Tier 1. Task
22A therefore defines the generic Ares resource boundary
`MAX_PLANNED_LAYERS_PER_PROJECT = 100_000`. This is neither a fixture constant
nor an Option value and is not written to G-code. It counts every materialized
`PlannedLayer` across all planned transform groups in final enumeration order.
Exactly 100,000 records are allowed; needing record 100,001 returns
`SliceError::InvalidInput` containing
`project layer count exceeds supported limit of 100000`.

Generation must also prove floating-point progress before every loop-appended
pair. After profile selection and the midpoint stop check, calculate
`next_print_z = print_z + height` once. It must be finite and strictly greater
than `print_z`; otherwise return `SliceError::InvalidInput` containing
`layer_height does not advance print_z`. Use that same computed value for the
pair and next iteration so the check does not change source accumulation. The
unconditional fixed first pair still counts against the project budget and is
not subject to this loop-progress check. Before emitting it, require one
remaining project record; an exhausted budget returns the count-limit error.

Within layer generation, error precedence is: nonfinite candidate/intermediate,
midpoint stop, non-progress, project record budget, then append. A series that
finishes at exactly the budget succeeds. The implementation must check the
budget during bounded generation rather than first allocating or iterating an
unbounded predicted count.

### Fixed height profile

Build the exact empty-range profile represented by
`layer_height_profile_from_ranges`:

1. Insert `[0, first_object_layer_height]` and
   `[first_object_layer_height, first_object_layer_height]` because the first
   layer is fixed.
2. Fill the remaining interval through object top with regular `layer_height`.
3. Apply the cited append/compression behavior using strict
   `abs(a-b) < 1e-4`; do not round coordinates to six decimal places.

For equal first and regular heights, a 92 mm object at 0.2 mm therefore has the
compressed profile `[0, 0.2, 92, 0.2]`. That example is acceptance evidence,
not a production constant.

### Object layer pairs and layer records

Generate bottom/top pairs exactly in source order:

1. Emit the fixed first pair `[0, first_object_layer_height]`.
2. Initialize `slice_z` for loop termination as
   `print_z + 0.5 * min_layer_height`.
3. Select/interpolate the profile height at each candidate slice Z using the
   cited `lerp` formula and strict upstream boundaries.
4. Recompute candidate `slice_z = print_z + 0.5 * height`; reject it if
   nonfinite, then stop when it reaches or exceeds object height.
5. Otherwise compute and validate the single `next_print_z` required by the
   bounded-generation contract, require one remaining project record, append
   `[print_z, next_print_z]`, and continue from that exact value.
6. Do not align the final top to object height because `precise_z_height` is
   outside this task.

The fixed first pair is unconditional. It remains the sole pair when its height
equals or exceeds object height. In the loop, midpoint equality with object
height stops before appending the candidate (`>=`, not `>`); neither edge may
be rewritten as an object-height clamp.

Convert each pair to one `PlannedLayer`:

- `id` is pair index because the supported raft count is zero;
- `height = hi - lo`;
- `print_z = hi` because `object_print_z_min` is zero;
- `slice_z = 0.5 * (lo + hi)` because ZAA is gated off.

Every finite intermediate and final value is derived from typed config and
transformed model data. An empty pair series is a keyed `InvalidInput`; no empty
placeholder plan is accepted.

## Project slicing lifecycle and error precedence

The public byte API keeps this exact order:

1. load and validate the archive/project documents;
2. resolve the bounded typed project configuration;
3. generate the Bambu config block when applicable;
4. apply the project-wide key-major Task 22A capability gates;
5. build transformed bounds, slicing parameters, fixed profiles, and bounded
   planned layers in final object/group order;
6. return `SliceError::ProjectSlicingIncomplete` because mesh-plane slicing and
   later stages are not implemented.

Archive, typed-option, materialization, effective-config, and config-writer
errors therefore retain precedence over layer planning. A layer-planning error
is newly observable only after those stages succeed. Non-Bambu projects still
skip config-block writing but must run the same typed layer-planning stage.
In particular, empty `min_layer_height`/`max_layer_height` errors are step-2
effective-config errors and precede both a step-3 config-writer failure and all
step-4 unsupported-feature gates.
All unsupported-feature errors precede bounds, numeric-progress, and layer-count
errors. After those gates, object/group-major planning makes an earlier planned
object's bounds or resource error observable before any later object's planning
error.

The config block bytes are frozen by earlier tasks. Task 22A may carry them in
private state but must not regenerate, reorder, normalize, or inspect them to
plan layers.

## Exact production and test scope

The implementation plan may modify only the minimum paths needed from this
set, and must freeze the final manifest before implementation:

- `crates/ares-core/src/project_slice.rs`;
- new private modules below `crates/ares-core/src/project_slice/` for layer
  parameters/planning, state, and focused tests;
- `crates/ares-core/src/project/load.rs` and focused load tests for the
  case-insensitive painted-profile presence bit;
- `crates/ares-core/src/project/domain.rs` for crate-private typed ownership and
  access to that presence;
- `crates/ares-core/src/project/layer_config_ranges.rs` and its focused tests
  for typed `layer_height` presence/value parsing;
- `crates/ares-core/src/project/effective_config/types.rs`;
- `crates/ares-core/src/project/effective_config/candidates.rs`;
- `crates/ares-core/src/project/effective_config/cardinality.rs` and focused
  tests for the two nonempty nozzle-option vector preconditions;
- existing project/effective-config test module declarations and focused test
  helpers needed to prove behavior;
- the dynamic-value baseline only if the independently reproduced RED proves
  an existing owned entry is deleted (zero change is expected);
- this spec, the later approved plan, and—only after whole implementation
  approval—the two required architecture/roadmap documents.

The implementation must not modify the committed 3MF or G-code, the CLI golden
normalizer, old STL `planning.rs`, `pipeline`, `segments`, or `contours`, public
CLI/WASM signatures, workspace membership, dependencies, or unrelated staged
`PrintApply` scaffolds.

All changed or new Rust modules must remain below 400 physical lines. Split by
upstream responsibility rather than suppressing the repository LOC gate.

## Typed and portability constraints

- The planning path consumes only `Project`, `ProjectDocuments` presence,
  `LayerConfigRange`, `BoundedResolvedProjectConfig`, `ObjectOptions`,
  `RegionOptions`, typed `RegionOptionOverrides`, `ProjectSettings`, meshes,
  transforms, and the generic project layer-record budget.
- No production path may access `tests/`, the reference G-code, fixture hashes,
  expected sizes/counts, filenames, or timestamps.
- No `serde_json::Value`, `Map<String, _>`, runtime option registry, `Any`, JSON
  round-trip, or source-parser subprocess may be added. The external XML reader
  may recognize the exact `layer_height` key and immediately produce
  `OrcaFloat`; no runtime string-key option dispatch may enter parameter or
  layer planning.
- No C++ binding, Orca executable invocation, filesystem I/O, terminal access,
  UI, OpenGL, or platform-specific implementation may enter `ares-core`.
- The implementation must compile for WASM and all Tier 1 native targets.
- Existing dynamic-audit findings may only shrink; no allowlist entry or moved
  fingerprint is permitted.
- Tests assert behavior and complete typed results. They must not execute or
  parse the pinned Orca source tree, assert upstream line numbers/symbol names,
  or become source-level pinning tests.
- There is no legacy fallback to the retained STL planner. Project slicing must
  not call `planning.rs`, `segments.rs`, `contours.rs`, `pipeline`, or existing
  approximate G-code assembly.

## TDD acceptance

### Required REDs

The independently approved plan must establish genuine failing tests before
each production package. At minimum the test matrix must include:

1. **Stable source mapping:** a synthetic project whose first source object has
   no printable instance proves the remaining resolved object maps back to its
   own mesh rather than resolved index zero.
2. **Source-first-instance all-vertex bounds:** composed first-instance/volume
   transforms produce exact finite Z bounds; a second transform-specific group
   reuses that height, an unreferenced vertex contributes, a vertex-only
   model-part contributes, and non-model-part volumes do not.
3. **Bound edges and failures:** negative and materially nonzero minimum Z are
   accepted without normalization while maximum Z remains the height; no finite
   model-part vertex, nonpositive maximum Z, and nonfinite transformed values
   fail with bounded keyed errors.
4. **Variable-height reader presence:** mixed-case
   `Metadata/layer_heights_profile.txt` presence is retained and rejected as
   `layer_height_profile`; range `layer_height` parses to `OrcaFloat`, later
   duplicate keys win, invalid text remains a reader error, and any typed range
   value is rejected as `layer_height` rather than silently fixed-planned.
5. **Nozzle-vector preconditions:** empty `min_layer_height` and
   `max_layer_height` independently return keyed effective-config errors;
   combined empty vectors prove min-before-max order, and a simultaneous
   config-writer failure proves both validations precede config writing.
6. **Option gates:** each of `raft_layers`, `enable_support`,
   `enforce_support_layers`, `precise_z_height`, and region `zaa_enabled`
   independently returns the named unsupported feature. Both overlapping and
   definitely nonintersecting parameter modifiers with typed true
   `zaa_enabled` are rejected without becoming resolved candidates;
   absent/false modifier values do not trigger the gate. A simultaneous
   config-writer failure wins, and after that writer input is repaired the ZAA
   error appears, proving this presence check remains in planning. Combined
   same-object and two-object cases prove complete project-wide key-major
   precedence, including object 1 `layer_height` before object 0 `raft_layers`.
7. **Object-extruder sources:** all six gated region selectors, print-wide
   brim, object/volume fallback, and parameter-modifier raw selectors
   contribute exactly. An intersecting range's generic `extruder` may
   contribute only through an active concrete feature-selector fallback; the
   same generic selector in a definitely nonintersecting range remains visible
   to released bounded usage but does not affect the per-object nozzle vector.
   An unoccupied range feature selector also does not contribute.
8. **Nozzle limits and indexing:** zero/configured min and max values,
   75%-of-nozzle default, multi-selector aggregation, empty-source fallback,
   and direct object-extruder IDs 0, 1, 2, and out of range distinguish the
   cited subtract-one/first-value behavior. A nonidentity `filament_map` proves
   the option is not consulted.
9. **First-height fallback:** positive `initial_layer_print_height` wins;
   zero/negative values use regular `layer_height`.
10. **Profile compression:** equal and unequal first/regular heights, object-top
   termination, duplicate append removal, and strict `1e-4` boundary behavior
   compare the complete profile vector.
11. **Layer series:** complete bottom/top pairs and complete `PlannedLayer`
    records cover a single short object, a non-divisible object height, different
    first/regular heights, first height equal to object height, first height
    greater than object height, a next candidate midpoint exactly equal to
    object height, and deterministic repeated execution.
12. **Bounded generation:** the smallest positive `f64` regular height after a
    0.2 mm first layer returns the exact non-progress error without appending a
    zero-height pair; a finite progressing input needing 100,001 records returns
    the exact count-limit error; exactly 100,000 records succeeds; and the same
    total budget spans multiple object/transform plans. Distinguishing cases
    lock nonfinite, midpoint-stop, non-progress, and budget precedence without
    introducing a test-only production limit.
13. **Lifecycle precedence:** malformed archive, effective-config failure, and
    config-writer failure remain earlier than planning; a valid project with an
    invalid planning value reaches the new keyed planning error; a fully valid
    project still reaches `ProjectSlicingIncomplete`.
14. **Real KSR fixture:** solely from committed 3MF bytes, assert one planned
     print object, 460 complete records, first
     `(id=0,height=0.2,print_z=0.2,slice_z=0.1)`, and source-accumulated final
     `print_z=92.00000000000077` with bits `0x4057000000000036`. Separately prove
     that the displayed/approximately compared top is 92 mm without rounding
     the stored value. Require strictly increasing slice Z, deterministic
     repetition, and the unchanged valid public incomplete result. Also retain
     the exact 49,004-byte config-block hash regression owned by Task 19C.

Fixture counts and expected values belong only in tests. The real-fixture RED
must fail because the typed project layer-planning seam does not yet exist, not
because a test was manually forced to fail.

### GREEN and regression gates

After every approved package and again on the frozen whole implementation:

```powershell
cargo +1.91.0 nextest run -p ares-core project_slice
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
```

The whole release gate additionally runs workspace nextest, CLI/WASM checks,
release browser bindings, the real-3MF browser test, fixture/hash guards,
forbidden-pattern scans, diff/whitespace checks, and the physical-LOC audit.
The implementation plan must spell out exact commands and expected counts from
its frozen baseline rather than relying only on the abbreviated list above.

## Explicit deferrals

Task 22A defers all of the following without authorizing a fallback:

- raft and support contributions to `SlicingParameters`;
- `precise_z_height` final-five-layer adjustment;
- ZAA non-midpoint slice Z;
- applying object-owned `layer_height` range values, parsing adaptive/editor
  profile payloads, and generating variable layer heights; their typed
  presences and unsupported gates are included and may not be deferred;
- nonidentity filament shrinkage (already rejected by the bounded project
  validator);
- Task 21A scaled `Coord`, `Point`, `Polygon`, and `ExPolygon` domain types;
- Clipper boolean/offset behavior;
- triangle-plane intersection, scaled XY points, segment chaining, loop repair,
  raw object slices, negative/modifier mesh application, and region assignment;
- applying parameter-modifier region geometry remains deferred, but typed true
  modifier `zaa_enabled` presence rejection is included and may not be deferred;
- perimeters, fills, supports, toolpaths, G-code body/header/footer assembly,
  generated-by metadata, time estimation, and post-processing;
- embedded preset extraction, external profile discovery/management,
  compatibility evaluation, CLI profile overrides, UI/runtime behavior, and
  any Ares-owned alternative slicing pipeline;
- successful normalized `ksr_fdmtest_v4` G-code parity.

The persistent full parity goal remains active after this task.

## Independent review, documentation, and release gates

1. Freeze this spec by path, byte count, and SHA-256.
2. Obtain literal `VERDICT: APPROVE` from an independent Codex reviewer and a
   default-model OpenCode reviewer for the exact frozen bytes. Any edit
   invalidates both approvals.
3. Author and freeze a Subagent-Driven TDD implementation plan only after both
   spec approvals.
4. Obtain literal plan `VERDICT: APPROVE` from a different independent Codex
   reviewer and the default-model OpenCode reviewer. Any edit invalidates both.
5. Execute the approved plan with bounded implementer subagents. Each package
   requires independent specification-compliance and code-quality `APPROVE`
   verdicts before the next dependent package begins.
6. Freeze the whole implementation and obtain independent whole
   specification-compliance, whole code-quality, and default-model OpenCode
   implementation approvals. Repeat correction and review until all three say
   `APPROVE`.
7. Only then update `docs/architecture/option-parity-v4.md` and
   `docs/roadmap.md`. Obtain an independent documentation `APPROVE`.
8. Run the complete post-documentation local release matrix, freeze exact
   tracked bytes, and create a reviewed Conventional Commit.
9. Push normally, verify local/tracking/direct-remote SHA equality, and require
   exact-SHA Tier 1 success across format, Ubuntu/Linux, WASM, macOS, and
   Windows before recording Task 22A as released.

No implementation, documentation-completion claim, commit, push, or Task 22A
release claim may bypass these gates.
