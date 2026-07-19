# Task 22J Implementation Plan: Single-Range Volume Region Composition

## Status, fixed points, and success condition

This plan is a draft. No production or tracked test implementation is
authorized until the exact specification and plan bytes receive all
pre-implementation review approvals.

The fixed Ares baseline is commit
`eb3aa56118d75c970886d46952fdfde1f8b198b1`, tree
`35b99bc2ad16abc4a37e09dd6d62b6494cafc075`; exact-SHA Tier-1 run
`29676205957` is green on all five jobs. The fixed OrcaSlicer source is commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, with exact blobs and ranges
listed in the Task 22J specification.

Success means:

- the released post-I volume stream converts to occurrence-ID `VolumeSlices` and
  retains a complete all-layer sidecar;
- accepted single-implicit-range full-mesh bounds and source-order volume-region
  graph match the fixed upstream boundary;
- model parts, negatives, accepted modifiers, multi-parent forwarding, fast
  classification, complex clipping, occurrence ordering, and same-region closing
  match fixed-source vectors;
- every composed ExPolygon becomes a private `stInternal=4` surface with
  fixed defaults, while every planned layer remains present;
- the committed KSR project matches its exact ordered `ARES22J` checkpoint in
  native Rust and a real WASM browser;
- loaded `layer_config_ranges`, painted triangle attributes, shared-mesh/raw-ID
  identity, print centering, and all ten existing modifier usage gates remain
  fail-closed at their existing boundaries;
- structural, hardcoding, provenance, native, WASM, browser, and independent
  review gates pass while the public project API remains
  `ProjectSlicingIncomplete`;
- exact reviewed bytes are committed, pushed normally, and green in exact-SHA
  Tier-1 before the next parity slice begins.

Task 22J does not claim complete normalized G-code parity.

## Immutable behavior ledger

The implementation must preserve these non-substitutable facts:

1. Task 22J runs once after Task 22I and before top-empty-layer removal or any
   later surface processing.
2. It consumes only the loaded 3MF, resolved typed Options, selected scale,
   and released Task 22I geometry.
3. Current production input has one implicit `[0, DBL_MAX)` layer range.
4. Rejection depends on a nonempty loaded
   `ProjectObject::layer_config_ranges()` vector; absent and loaded-empty range
   documents produce identical I/J bytes.
5. Production planning always creates at least one finite layer with
   nonnegative `slice_z`; J trusts that invariant and adds no zero-layer
   fallback.
6. No Task 22J type, test hook, or helper lifts the loaded-range rejection.
7. The range-filtered upstream chain is deferred as one future cohesive slice,
   not partially materialized in dormant J structures.
8. Only model-part, negative, and parameter-modifier volumes participate.
9. Fixed Orca's runtime `ModelVolume::ObjectID` is not Ares'
   `ProjectVolume::id()`, which is the raw 3MF resource ID.
10. The released nonzero `VolumeOrdinal` becomes private per-object
    `VolumeOccurrenceId`; source index only joins source metadata and Options.
11. Occurrence identity preserves flattened breadth-first construction order
    and gaps from nonempty support volumes without claiming global numeric
    equality with Orca.
12. Occurrence-ID order owns bounds lookup and `VolumeSlices`; source volume
    order owns clipping and modifier priority.
13. Loaded raw IDs `[3,1,2]` map to occurrence IDs `[1,2,3]`; a support gap maps
    participating carriers to `[1,3]`.
14. Every post-I mode and volume kind is dropped from `VolumeSlices`; the
    occurrence ordinal is retained as identity.
15. The sidecar clones every volume layer before destructive composition; it
    is not a first-layer-only cache.
16. Empty middle and final layers remain in the sidecar and dense output.
17. The resolved first print instance transform and volume transform compose
    in `f64`; combined XY translation is then zeroed, while Z remains.
18. Matrix coefficients and referenced vertices narrow to `f32` before
    transformed bounds are accumulated.
19. Unreferenced vertices do not change bounds.
20. Positive `xy_contour_compensation` inflates XY bounds; zero and negative
    values do not shrink them.
21. Z bounds inflate by `EPSILON`; activity uses
    `plan.layers[layer_index].slice_z as f32`, never `print_z`, and all Z/XY
    bbox tests are inclusive.
22. Region IDs use first-created exact `RegionOptions` equality.
23. Model parts use the already resolved single-implicit-range candidate.
24. Negative records have neither region nor parent.
25. Modifiers scan prior printable records in reverse and extend ancestor
    bounds to the top model part.
26. Every intersecting parent whose modifier config changes receives one
    record; one no-op fallback is retained only if none changes.
27. Existing Region Option normalization and `material=None` are reused; no
    second merge implementation is created.
28. All ten current usage-affecting modifier gates remain unchanged.
29. Mesh-shared input or repeated request-wide nonempty raw resource IDs retain
    the existing `shared_mesh_centering` rejection before J.
30. Distinct/mismatched resolved print groups retain the existing
    `print_object_centering` rejection before J.
31. `clip_multipart_objects` is fixed true behavior, not a 3MF Option.
32. Zero or one non-model record emits no region geometry.
33. A multi-record non-overlap fast layer moves only its first active model
    part, even when later model parts are active.
34. Touching active bounds make the layer complex.
35. Modifier partition uses exact Intersection for the child and Difference
    for the parent without safety offset.
36. The untouched modifier source is forwarded only to the immediately next
    record for the same modifier.
37. A later model part or negative subtracts itself only from preceding
    nonempty, non-negative, XY-overlapping records.
38. A negative before a model part does not subtract from that later part.
39. The disabled fixed-source `trim_overlap` branch is not ported.
40. Difference and Intersection execute operation-to-Paths followed by one
    fresh NonZero Union-to-PolyTree, with no extra Paths union.
41. Valid complex records sort by `(region_id, volume_occurrence_id)`; empty or
    regionless records sort to the tail and are discarded.
42. Comparator-equivalent duplicate keys preserve Ares source-record order;
    no test claims an upstream total order that fixed C++ does not define.
43. Same-region geometry appends in sorted order, and closing runs only after
    at least two nonempty sources actually merge.
44. Closing delta is floating `EPSILON / scale.factor()`: Normal 100.0,
    LargeBed 10.0.
45. Every output ExPolygon becomes one Internal surface with tag 4 and defaults
    `-1,1,-1,0` in that order of fields.
46. Empty region slots are explicit; top-empty-layer trimming does not run.
47. The committed KSR takes the single-model-part fast path, so synthetic
    fixed-source vectors are mandatory release gates for complex behavior.
48. The loaded `bridge_angle=37` modifier and no-override control must match
    their complete fixed vectors twice in native Rust and Chromium.
49. Painted BBS triangle attributes retain the exact external loader error;
    they are never accepted and silently discarded.
50. Fixed BBS 3MF has no serialized Orca ModelMaterial association, so no
    unreachable material fallback is added.
51. No executable Orca source-pinning test, fixture branch, reference-G-code
    dependency, or out-of-band Option toggle returns.
52. Public `slice_project` executes through J and still returns
    `ProjectSlicingIncomplete`.

The production planner invariant is fixed by current Ares
`parameters.rs:37-52,77-98`, `layers.rs:51-56,72-84,126-162`, and
`project_slice.rs:273-287`. Existing predecessor regressions
`task22a_first_layer_height_uses_positive_value_or_regular_fallback`,
`task22a_invalid_slicing_parameter_numbers_are_keyed`,
`task22a_fixed_first_pair_is_unconditional_at_and_above_top`,
`task22a_smallest_positive_regular_height_rejects_nonprogress`, and
`task22a_layer_generation_error_precedence_is_fixed` remain release gates.

## Working protocol

Work proceeds in serial vertical TDD packages. For every package:

1. freeze its allowed paths, exact source ranges, and concrete acceptance
   vectors;
2. add only package-owned tests in separate real modules;
3. run the smallest focused nextest or browser command and record the expected
   compile or behavior RED in `.superpowers/sdd/task22j-evidence.md`;
4. implement the smallest source-cited behavior that makes that RED green;
5. run focused regressions, rustfmt, relevant Clippy, LOC, macro, hardcoding,
   and changed-path checks;
6. freeze the package path and per-file SHA-256 manifest;
7. obtain independent specification and code-quality approval before the next
   package begins.

Package 0 owns the neutral planning extraction, exact checkpoint registration,
and browser feature transition. Package A owns binary ExPolygon Booleans.
Package B owns the shared private occurrence identity, accepted full-mesh
bounds, and the single-implicit-range graph. Package C1 owns the
occurrence-keyed sidecar, dense surfaces, and fast assignment.
Package C2 owns the complex composer. Package D wires the real project stage
and complete native checkpoint. Package E promotes unchanged bytes to
WASM/browser. Package F performs docs-inclusive closure, six-axis repair loops,
and release.

Expected constants never change to accommodate Ares output. A mismatch is an
implementation defect until fixed-source evidence and independent reviewers
prove otherwise.

Use `apply_patch` for manual edits. Do not modify committed fixtures. Do not
amend, squash, force-push, or rewrite released Task 22A-I history.

## Pre-implementation exact-byte gate

Before Package 0:

1. preserve the completed read-only fixed-source, current-Ares, input
   capability, range-boundary, material/painting, manifest, and executable
   oracle audits;
2. verify both ignored probes' strict builds and all four five-run hash sets:
   synthetic, KSR, modifier-graph composition, and control-graph composition;
3. freeze specification and plan SHA-256 values;
4. dispatch an independent fixed-source/specification reviewer;
5. dispatch an independent current-Ares/implementation-plan reviewer;
6. dispatch a direct default-model reviewer with edits denied;
7. require literal approval from every reviewer on the same exact bytes.

Any document edit invalidates all document approvals. Any unresolved P0-P3
finding blocks implementation.

## Exact planned tracked manifest

No tracked path outside this 34-path list may change without a plan amendment
and fresh document approvals. The final candidate must change every listed
path; substitutions, omissions, and additions are forbidden.

### Specification, architecture, and roadmap

- `docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22j-volume-regions.md`
- `docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22j-volume-regions.md`
- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

### Core feature and export boundary

- `crates/ares-core/Cargo.toml`
- `crates/ares-core/src/lib.rs`

### Geometry implementation and tests

- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/clipper.rs`
- `crates/ares-core/src/geometry/clipper/boolean_ex.rs`
- `crates/ares-core/src/geometry/tests/clipper.rs`
- `crates/ares-core/src/geometry/tests/clipper/boolean_ex.rs`

### Project stage implementation

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/planning.rs`
- `crates/ares-core/src/project_slice/state.rs`
- `crates/ares-core/src/project_slice/closing.rs`
- `crates/ares-core/src/project_slice/volume_bounds.rs`
- `crates/ares-core/src/project_slice/volume_regions.rs`
- `crates/ares-core/src/project_slice/region_slices.rs`
- `crates/ares-core/src/project_slice/region_slices/complex.rs`
- `crates/ares-core/src/project_slice/task22j_oracle.rs`

### Project and loader tests

- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/profile_layers.rs`
- `crates/ares-core/src/project_slice/tests/volume_bounds.rs`
- `crates/ares-core/src/project_slice/tests/volume_regions.rs`
- `crates/ares-core/src/project_slice/tests/region_slices.rs`
- `crates/ares-core/src/project_slice/tests/region_slices/complex.rs`
- `crates/ares-core/src/project_slice/tests/region_fixture.rs`
- `crates/ares-core/src/project_slice/tests/region_fixture/checkpoint.rs`
- `crates/ares-core/src/project/tests/model/production.rs`

### WASM browser conformance and Tier-1

- `crates/ares-wasm/Cargo.toml`
- `crates/ares-wasm/src/lib.rs`
- `crates/ares-wasm/tests/browser/index.html`
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`
- `.github/workflows/tier1.yml`

The ignored .superpowers/sdd/task22j-oracle tree, evidence ledger, temporary
target directories, wasm-bindgen output, Playwright output, and generated 3MF
mutations are never staged.

## Module ownership and line budgets

Every changed Rust production and test file must remain below 400 physical
LOC. Target budgets leave review and formatting headroom:

- `lib.rs`: at most 285;
- `geometry.rs`: at most 80;
- `geometry/clipper.rs`: at most 210;
- `geometry/clipper/boolean_ex.rs`: at most 180;
- `geometry/tests/clipper.rs`: at most 90;
- `geometry/tests/clipper/boolean_ex.rs`: at most 320;
- `project_slice.rs`: at most 300 after the real planning extraction;
- `project_slice/planning.rs`: at most 180;
- `project_slice/state.rs`: at most 85;
- `project_slice/closing.rs`: at most 250;
- `project_slice/volume_bounds.rs`: at most 360;
- `project_slice/volume_regions.rs`: at most 360;
- `project_slice/region_slices.rs`: at most 330;
- `project_slice/region_slices/complex.rs`: at most 350;
- `project_slice/task22j_oracle.rs`: at most 220;
- `project_slice/tests.rs`: at most 45;
- `project_slice/tests/profile_layers.rs`: at most 390; only its moved import
  path may change;
- `project_slice/tests/volume_bounds.rs`: at most 360;
- `project_slice/tests/volume_regions.rs`: at most 380;
- `project_slice/tests/region_slices.rs`: at most 350;
- `project_slice/tests/region_slices/complex.rs`: at most 380;
- `project_slice/tests/region_fixture.rs`: at most 330;
- `project_slice/tests/region_fixture/checkpoint.rs`: at most 320;
- `project/tests/model/production.rs`: at most 270;
- `ares-wasm/src/lib.rs`: at most 180.

The browser HTML remains below 340 lines and the Playwright specification below
395 lines by replacing Task 22I parsing/tests, not appending a second parser.
Documentation and workflow files are not Rust LOC targets.

## Exact implementation shape

### Neutral planning extraction

Move `plan_project` and `plan_resolved_objects` from the root
`project_slice.rs` into real module `project_slice/planning.rs` without changing
signatures or behavior. Update `state.rs` and the one `profile_layers.rs` import
path. Before and after the move, run the complete Task 22I checkpoint and prove
exact byte identity.

No reformat-only cleanup, helper renaming, or adjacent refactor is authorized.
This extraction exists only to keep the root below 400 LOC when J wiring is
added.

### Exact binary ExPolygon wrappers

Add private functions in `geometry/clipper/boolean_ex.rs`:

```text
difference_ex(subject: &[ExPolygon], clip: &[ExPolygon])
intersection_ex(subject: &[ExPolygon], clip: &[ExPolygon])
```

Each function flattens contour and holes through the existing ordered geometry
conversion, adds subject/clip paths to `ClosedClipper`, executes the requested
operation to Paths with NonZero/NonZero, returns empty on empty Paths, and feeds
nonempty Paths to a fresh Union-to-PolyTree with NonZero/NonZero. Convert with
the released ordered PolyTree ownership traversal.

Do not call `union_ex`, because it would add an extra Paths union. Reuse
existing `ClosedClipper`, `ClipperOptions::default()`, `PathRole`,
`ClipOperation`, and `FillRule`; do not expose these wrappers publicly.

### Accepted full-mesh bounds

`volume_bounds.rs` defines the private nonzero `u32` `VolumeOccurrenceId`,
promotes the released post-I `VolumeOrdinal` exactly once, and owns the
source-index lookup shared by bounds, graph, and the later sidecar. It also owns
a minimal private `BoundingBox3f` with inclusive XY/Z queries. It builds one
bound per participating source volume and stores source index plus that shared
`VolumeOccurrenceId`.

Use the resolved first print instance transform corresponding to fixed Orca's
`print_instances.front().trafo` and the source volume transform in fixed order.
Compose in `f64`, zero combined XY translation, narrow matrix and referenced
vertex coordinates to `f32`, and accumulate only triangle-referenced vertices.
Inflate XY only by positive `xy_contour_compensation as f32`; inflate Z by
fixed `EPSILON`.

The module has no layer-range collection, slab clipper, config pointer,
filesystem input, or test-only production branch.

### Single implicit-range region registry and graph

`volume_regions.rs` owns:

- the first-created `Vec<RegionOptions>` registry;
- one source-order `Vec<VolumeRegion>`;
- source volume index, occurrence ID, kind, parent, optional region ID, and
  bound reference for each record.

Model-part records reuse the existing resolved candidate by source volume
index. Negative records carry no region. Modifier records call the existing
`RegionOptions::resolve` with `RegionBase::Modifier`, source volume overrides,
`material=None`, full filament Region Options, and logical filament count.

The reverse parent scan extends bounds through modifier ancestors. Add all
changed intersecting parents; add only the last intersecting model-part
fallback when none changes. Region equality is exact `PartialEq` and IDs are
vector indices.

Do not edit `effective_config/candidates.rs`, `types.rs`, `usage.rs`,
`layers.rs`, `occupancy.rs`, or `raw_intersections.rs`. Their existing input
and rejection contracts remain authoritative, including exact loaded-range,
shared-mesh/raw-ID, and print-centering gates.

### Stable sidecar, dense regions, and internal surfaces

`region_slices.rs` owns private production types equivalent to:

```text
VolumeSlices {
  volume_occurrence_id: VolumeOccurrenceId,
  layers: Vec<Vec<ExPolygon>>,
}

PostRegionPrintObject {
  plan: PlannedPrintObject,
  volume_slices: Vec<VolumeSlices>,
  regions: Vec<PostRegion>,
}

PostRegion {
  id: usize,
  options: RegionOptions,
  layers: Vec<RegionLayer>,
}

RegionLayer {
  surfaces: Vec<RegionSurface>,
}

RegionSurface {
  kind: Internal,
  expolygon: ExPolygon,
  thickness: f64,
  thickness_layers: u16,
  bridge_angle: f64,
  extra_perimeters: u16,
}
```

Names may follow local Rust style, but fields and ownership may not omit
observable future-stage state. Surface constructor defaults are exactly
`Internal/-1.0/1/-1.0/0`.

`VolumeOccurrenceId` is defined and constructed in `volume_bounds.rs` during
Package B by promoting the released post-I `VolumeOrdinal`. It is namespaced
per print object. No raw XML ID, source index, process-global counter, or
independently renumbered participating-volume sequence may construct it.

Package C1 reuses those already-promoted identities, uses source indices only
for joins, sorts by occurrence ID, clones the whole sidecar, and consumes the
other copy. Allocate every region with exactly the planned layer count before
fast/complex dispatch. Empty slots remain.

### Fast assignment

Implement one-range fast classification directly in `region_slices.rs`:

- zero records: no writes;
- one model part: move every layer's slices to its region;
- one non-model record: no writes;
- multiple records: per `plan.layers[layer_index].slice_z as f32`, locate the
  first active model part and detect later inclusive overlap with active
  predecessors; `print_z` is never used;
- noncomplex layers move only that first model part;
- collect complex layer indices for `complex.rs` in ascending order.

Use occurrence-ID binary lookup only for physical slice carriers. Graph
iteration remains source order. Do not keep mode or volume kind in the new
carrier.

### Complex composition

`region_slices/complex.rs` owns the per-layer temporary vector, modifier
partition/forward, later-part/negative subtraction, valid-record ordering,
same-region append, and closing.

Move physical slices into every graph record exactly as fixed source does. For
repeated modifier records, first move consumes the carrier and explicit forward
populates the next record. Difference and Intersection call only the Package A
wrappers. Bbox checks remain inclusive and skip preceding negative records.

Use a stable Rust sort by validity then `(region_id, volume_occurrence_id)` so
comparator-equivalent records retain source order. Do not add an invented third
tie key or claim that order as fixed C++ semantics.

Track whether at least two nonempty same-region sources actually append. Only
then call existing `offset2_ex` with outward/inward deltas
`+float(EPSILON/scale.factor())` and its negative, Miter join, and limit 3.
Map any Clipper range error through the exact J-specific private error text.

Convert final ExPolygons to default Internal surfaces without sorting or
canonicalization.

### Real project stage and lifecycle

Add a preparation function after `prepare_post_simplification` that:

1. matches each post-I object to its source and resolved object;
2. builds full-mesh bounds and the single region graph;
3. composes the object to `PostRegionPrintObject`;
4. preserves project, resolved config, config block, and coordinate scale in
   the existing prepared state shell.

Public `slice_project` destructures and exercises the post-J state before
returning `ProjectSlicingIncomplete`. It must not reimplement, call, or bridge
to old `pipeline`, `print_apply`, `print`, or `surface` code.

### Checkpoint and browser feature

`task22j_oracle.rs` encodes the exact protocol from the specification. It owns
only serialization; no algorithm or Option decision belongs in the encoder.

Replace non-default Cargo feature `task22i-browser-oracle` with
`task22j-browser-oracle`. Under it, expose exactly:

- `task22j_browser_input_oracle`: complete released post-I bytes with
  `ARES22I\0`;
- `task22j_browser_oracle`: complete post-J bytes with `ARES22J\0`.

The generated JS exports are exactly `task22jBrowserInputOracle` and
`task22jBrowserOracle`. Default bindings expose neither. Remove I browser
feature aliases and exports; native I helpers remain only where released tests
require them.

The browser parser is independently hand-written for the nested sidecar and
layer-major dense region grammar. It validates every count, signed coordinate,
empty vector, surface tag, and exact EOF. It must not call a Rust parser or
trust only a top-level hash.

## Error and invariant contract

Task 22J adds no public `SliceError`. Its project module privately maps
`ClipperError::CoordinateOutOfRange` to exactly:

```text
project region composition polygon coordinate is outside the supported Clipper range
```

Bounds and graph functions trust already validated internal project state.
They add no `None`, duplicate, finite-number, or length fallback for states the
loader/resolver cannot produce. Debug assertions may document parent-before-
child, dense region index, occurrence-ID uniqueness, and layer-count
invariants. Raw duplicate IDs are owned by the earlier explicit gate.

The existing external errors remain unchanged:

- nonempty loaded layer-range vector:
  `UnsupportedProjectFeature("layer_config_ranges")`;
- shared mesh or repeated request-wide nonempty raw ID:
  `UnsupportedProjectFeature("shared_mesh_centering")`;
- distinct/mismatched resolved print groups:
  `UnsupportedProjectFeature("print_object_centering")`;
- each of the ten modifier keys: its existing exact key;
- `paint_color` and `paint_fuzzy_skin`:
  `SliceError::InvalidInput("invalid project model XML: attribute namespace does not match its 3MF meaning")`.

No supplied XML, path, fixture label, Option document, or geometry is included
in a new error string.

## Oracle registration

Before production behavior exists, register these immutable complete values:

- project fixture SHA-256
  `698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`;
- released I: 999,721 bytes,
  `0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef`;
- target J: 2,008,706 bytes,
  `2b474697f4afae95c9a55d709d8740d382a80b2969fc5118dc89e13c1906162d`;
- synthetic J: 5,880 bytes,
  `cb681dd4761dc69482f626374079f851ace0b9ec8d02587300c4495d84e0f4aa`;
- synthetic ordered text digest
  `938c8bcb02449c0ea77617973aed9b907313a2b0e4d9bb526c73ce158ee59691`;
- modifier/control common H: 478 bytes,
  `4bc72e587c1a7061624d6a20df20d1cb4482dcad84951152ad4640d622b11f7a`;
- modifier/control common I: 478 bytes,
  `4b37ef7c7816a29076288647810bcfb6fe0b341785b5a4505f602ab72f69cb87`;
- modifier J: 1,054 bytes,
  `1b18edae9cfbb9cd405cb7d45b1bec1a26168fe12c28a16366da211a30eadc77`;
- control J: 698 bytes,
  `f2185c996e62a897b6af721f043a8ac150df647780693e828845f594524fd3d4`.

Register both modifier archive ZIP/semantic identities and both complete
ordered rendering digests from the specification. The tracked project test
registers project and I/J checkpoints only. The reference G-code SHA-256
`10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3`
is evidence-ledger and final shell-integrity input only; no tracked Task 22J
test opens that file.

The Rust modifier builder reuses `KsrArchive`, whose `BTreeMap` fixes entry
order and whose locked ZIP stack fixes the default 1980 timestamp. Chromium
sorts uncompressed entry names and supplies an explicit 1980-01-01 `mtime` to
`fflate`. Container bytes may differ across encoders, so both runtimes assert
the specification's filename-NUL-content semantic digest. Native Rust also
asserts the released H predecessor identity; native and Chromium compare their
identical complete I and J outputs.

Register the four exact sidecar record lengths/digests and four retained-layer
record lengths/digests from the specification. Parse and assert object count,
planned/sidecar/retained layer counts, occurrence ID, dense region IDs,
surface type bytes, ExPolygon/hole/point totals in both geometry copies, and
exact EOF.

Register literal ordered synthetic coordinates for:

- single-part nonempty/empty/nonempty layer retention;
- disjoint first-part-only fast output;
- occurrence IDs 90 then 10 with later-part priority;
- negative after/before ordering;
- a two-level modifier chain;
- multi-parent repeated-source forwarding;
- Normal 150/250 gap and LargeBed 15/25 gap closing controls;
- an emptied final planned layer with complete sidecar.

The geometry wrappers also receive direct ordered Difference/Intersection
root, hole, nested-island, disjoint, contained, and empty vectors. Bounds and
region-graph arithmetic vectors are source-derived and independently reviewed;
the ignored region probe does not pretend to execute upstream project loading.

Register the specification's complete two-layer modifier/control vectors,
including archive semantic framing, equal H/I, resolved child
`bridge_angle=37`, no-override region reuse, every coordinate, zero-hole and
empty vector, type-4 tag, trace count, and exact EOF. A representative subset
or whole-stream hash without the complete parsed vector is insufficient.

Tracked tests never invoke or inspect the ignored probe. No tracked test
asserts an Orca source path, line number, commit, blob, or executable hash.

## TDD package sequence

### Package 0: neutral split, checkpoint registration, and browser transition RED

Allowed paths are the planning split/imports, Cargo features, checkpoint
wrappers, new fixture modules, WASM hooks, browser files, and Tier-1 export
audit.

1. Run the released Task 22I fixture checkpoint and record its exact bytes.
2. Move planning functions into the real module, update two import sites, and
   prove the Task 22I bytes remain identical.
3. Register independent I/J parsing, project fixture integrity, target J
   counts, representative records, synthetic vectors, exact EOF, and the
   complete modifier/control archives, H/I, J vectors, and semantic framing.
4. Re-run predecessor planner proofs for positive first height, nonempty
   production layers, nonnegative `slice_z`, strict progress, and empty-pair
   rejection.
5. Rename the browser feature and expected exports without adding J behavior.
6. Run `cargo nextest run -p ares-core task22j_`; record missing-hook or
   missing-stage compile RED.
7. Add only marker-level J checkpoint plumbing after I so tests compile. The
   complete target remains a behavior RED; no expected value changes.
8. Build fresh default and J-feature bindings, run export audit and Playwright,
   and record the browser behavior RED.

Package exit: neutral split identity, checkpoint provenance, and real behavior
RED receive independent fixed-source and quality approval.

### Package A: exact binary ExPolygon Difference and Intersection

Allowed paths are the five planned geometry files.

1. Register exact ordered simple, overlapping, disjoint, hole, nested-island,
   contained, first-pass-empty, and out-of-range vectors.
2. Include one vector that would change if an extra Paths union were inserted.
3. Record missing-function compile RED.
4. Implement only operation-to-Paths then fresh Union-to-PolyTree wrappers.
5. Pass focused Boolean, full Clipper, Task 22F-I geometry/checkpoint, rustfmt,
   Clippy, LOC, and no-macro checks.

Package exit: independent fixed-source/Boolean and Rust-quality reviewers
approve call count, fill rules, ownership order, and error propagation.

### Package B: full-mesh bounds and single implicit-range region graph

Allowed paths are `volume_bounds.rs`, `volume_regions.rs`, their two test
modules, the registered fixture modules, `project/tests/model/production.rs`,
and only necessary root module declarations.

1. Register transform-order, zeroed-XY, retained-Z, `f32` narrowing,
   referenced/unreferenced vertex, compensation sign, epsilon, and touching
   bbox vectors; record missing-module RED.
2. Implement the shared occurrence newtype/promotion/source lookup and the
   smallest private full-mesh bounds; pass focused tests.
3. Register graph REDs one case at a time: model part, config reuse, negative,
   changed modifier, multi-parent modifier, ancestor bounds, no-op fallback,
   and no intersecting parent.
4. Implement the registry and graph using existing resolved candidates and
   `RegionOptions::resolve`.
5. Build the exact accepted modifier/control archives from the specification;
   prove their H/I bytes are equal, `bridge_angle=37` exists only in variant
   model settings, variant creates child region 1, and control reuses region 0.
6. Run a real loaded raw-ID `[3,1,2]` project and a support-gap vector to prove
   occurrence IDs `[1,2,3]` and `[1,3]` without raw-ID sorting or renumbering.
7. Prove absent and loaded-empty range documents have identical I bytes and
   graph/config identity, and one loaded range retains the exact range
   rejection. Final J-byte equality belongs to Package D.
8. Re-run unchanged ten modifier, shared-mesh/raw-ID, and print-centering gates.
9. Add complete archive mutations with one unprefixed `paint_color` or
   `paint_fuzzy_skin` triangle attribute; assert exact `SliceError` equality
   through `load_project` and the real project path in the existing loader
   test module.

Package exit: independent input-boundary/specification and code-quality
reviewers approve Option ownership, occurrence/source order, graph parents, and all
unchanged capability gates.

### Package C1: occurrence sidecar, dense internal surfaces, and fast assignment

Allowed paths are `region_slices.rs`, closing accessors/cfg changes, its direct
test module, and root declarations.

1. Register occurrence-ID/source/raw-ID disagreement, complete sidecar clone,
   dropped mode/type with retained occurrence identity, dense regions, internal
   defaults, empty middle/final layers, and no-top-trim REDs.
2. Register zero, one model part, one negative, one modifier, multiple disjoint
   active model parts, a boundary where `slice_z` is active but `print_z` is
   not, touching bboxes, and no-active-model fast classifiers.
3. Implement private output types, conversion, allocation, fast movement, and
   ordered complex-layer collection.
4. Prove no complex Boolean or closing behavior exists in C1.
5. Run focused stage tests and complete predecessor I bytes.

Package exit: independent ownership/order and Rust-quality reviewers approve
the sidecar copy, dense output, fixed fast quirks, defaults, and layer retention.

### Package C2: exact complex composition and closing

Allowed paths are `region_slices/complex.rs`, its separate test module,
`region_fixture.rs`, and the already declared C1 integration seam.

1. Register fixed ordered overlapping IDs 90/10 and later-part subtraction;
   record behavior RED.
2. Implement later-part/negative Difference and pass only those vectors.
3. Register negative-before/after, empty-parent, modifier chain, and repeated
   multi-parent source-forwarding REDs.
4. Implement modifier Intersection/Difference and exact forward behavior.
5. Register occurrence sort, same-region append, Normal/LargeBed close/control
   gaps, and Clipper range-error REDs.
6. Implement final ordering, merge tracking, one exact offset2 closing, and
   Internal surface conversion.
7. Encode the complete 5,880-byte synthetic object set and require its fixed
   SHA plus independently parsed ordered geometry.
8. Pass focused complex, full region, Clipper, and Task 22F-I regressions.

Package exit: the synthetic fixed-source stream must be exact without changing
any constant. Independent source-algorithm and quality reviewers approve the
whole complex closure.

### Package D: real project wiring and complete native KSR checkpoint

Allowed paths are `project_slice.rs`, `state.rs`, `task22j_oracle.rs`, the
fixture/checkpoint modules, and core export cfg.

1. Wire the post-I object, source project object, resolved object, bounds,
   graph, and composer into one post-J preparation path.
2. Switch public lifecycle state consumption to post-J while retaining exact
   `ProjectSlicingIncomplete`.
3. Run the committed KSR native checkpoint twice.
4. Require exact 2,008,706 bytes, target SHA, all counts, occurrence ID 1, eight
   representative record digests, every surface tag, and exact EOF.
5. Compare the input hook byte-for-byte with released I.
6. Run modifier and no-override control twice through the real project path;
   require their exact archive/H/I identities, complete parsed J vectors,
   resolved graph Options, output hashes, trace facts, and exact EOF.
7. Run loaded-empty/absent ranges and every external rejection through the
   real project path; require absent and loaded-empty inputs to have identical
   final J bytes.

Package exit: independent requirement and quality reviewers approve real
source/resolved ownership, no hardcoding, complete native bytes, and unchanged
public lifecycle.

### Package E: unchanged complete WASM/browser promotion

Allowed paths are the five planned WASM/browser/Tier-1 files and already
registered feature hooks.

1. Build fresh optimized default and J-feature WASM artifacts in isolated
   target directories with wasm-bindgen 0.2.121.
2. Prove default exports no Task 22 hooks and the J feature exposes exactly the
   two J hooks; G/H/I browser hooks are absent.
3. Run the independent parser/WebCrypto KAT before loading the project.
4. Run the committed project twice and require exact I/J hashes, counts,
   representative records, sidecar/region geometry totals, type tags, EOF, and
   byte repeatability.
5. Build the exact modifier/control semantic entries with fixed timestamps,
   run each twice, and require common I plus both complete parsed J vectors,
   resolved Option/control distinction, hashes, tags, empty vectors, and EOF.
6. Run Playwright twice from fresh bindings and compare native/browser KSR,
   modifier, and control bytes.
7. Run the Task 22A-J focused chain and full project/core regressions.

Package exit: independent browser/WASM and specification reviewers approve the
same constants registered in Package 0.

### Package F: candidate closure, six-axis loop, docs, and release

1. After Package E approval, update architecture and roadmap with only verified
   source, implementation, test, release, and deferral facts.
2. Freeze the exact docs-inclusive 34-path and per-file SHA-256 manifest.
3. Run the full verification and structural matrix below on those exact bytes.
4. Dispatch one independent read-only six-axis reviewer. It must assess
   requirement completeness, logical correctness, edge cases, code quality,
   test coverage, and actual execution, and return a concrete repair list.
5. The main thread fixes every finding and reruns affected plus full gates.
6. Send the new exact docs-inclusive manifest to the same reviewer for
   revalidation. Repeat until literal `SIX-AXIS VERDICT: APPROVE` with no
   unresolved P0-P3 finding.
7. Dispatch fresh specification, quality, default-model, and documentation
   reviewers on the approved candidate; repair and re-review any finding.
8. Any documentation or code repair returns to step 2, reruns the full matrix,
   and requires the same six-axis reviewer plus all fresh reviewers to approve
   the new exact bytes.
9. Commit conventionally, push normally, and verify exact-SHA Tier-1 all five
   jobs before auditing the next upstream slice.

## Focused and full verification matrix

Focused native commands include:

- `cargo nextest run -p ares-core task22j_`
- `cargo nextest run -p ares-core geometry::tests::clipper::boolean_ex`
- `cargo nextest run -p ares-core project_slice::tests::volume_bounds`
- `cargo nextest run -p ares-core project_slice::tests::volume_regions`
- `cargo nextest run -p ares-core project_slice::tests::region_slices`
- `cargo nextest run -p ares-core project_slice::tests::region_fixture`
- `cargo nextest run -p ares-core task22j_loaded_modifier_control`
- `cargo nextest run -p ares-core bbs_painted_triangle_attributes_remain_fail_closed`
- `cargo nextest run -p ares-core task22b_mesh_shared_presence_and_repeated_numeric_keys_are_rejected_request_wide`
- `cargo nextest run -p ares-core task22b_print_object_centering_gate_accepts_collapsed_xy_and_rejects_distinct_or_mismatched_groups`
- `cargo nextest run -p ares-core task22a_first_layer_height_uses_positive_value_or_regular_fallback`
- `cargo nextest run -p ares-core task22a_layer_generation_error_precedence_is_fixed`
- `cargo nextest run -p ares-core task22`

Full Rust commands are:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo check --workspace --all-targets`
- `cargo nextest run -p ares-core`
- `cargo nextest run --workspace`
- `git diff --check`

WASM commands are:

- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo check -p ares-wasm --target wasm32-unknown-unknown`
- isolated release builds for default and `task22j-browser-oracle`;
- wasm-bindgen 0.2.121 generation into separate output directories;
- exact generated-export audit;
- `npm --prefix crates/ares-wasm/tests/browser ci`;
- `npx --prefix crates/ares-wasm/tests/browser playwright install chromium`;
- `npm --prefix crates/ares-wasm/tests/browser test`, twice.

Tier-1 must run workspace nextest and Clippy on Windows, macOS, and Ubuntu,
rustfmt on Ubuntu, and wasm32 plus real Chromium conformance on Ubuntu.

## Structural, provenance, and hardcoding audits

On every package and final candidate:

1. parse the backtick paths between `Exact planned tracked manifest` and
   `Module ownership and line budgets`, compare that set in both directions
   with tracked modified plus candidate untracked paths, and reject any
   addition, omission, substitution, duplicate, or count other than 34;
2. count every changed Rust file and reject physical LOC `>= 400`;
3. search changed Rust files for `include!` or `include_bytes!` source/test
   splitting;
4. confirm tests are declared through real `mod` files;
5. search production diffs for KSR names, fixture digests, expected counts,
   expected coordinates, reference-G-code paths, raw Option overrides, stage
   bypasses, and platform-specific behavior;
6. search changed tests for Orca checkout paths, source-line/hash assertions,
   probe execution, and superseded oracle constants;
7. confirm no unsafe, FFI, filesystem/process/thread, native-only dependency,
   or second geometry engine enters core;
8. prove `effective_config/{candidates,types,usage,layers,occupancy}.rs` and
   `project_slice/raw_intersections.rs` are unchanged;
9. run exact absent/loaded-empty/loaded-nonempty range behavior, the ten
   modifier gates, shared-mesh/raw-ID gate, and print-centering gate;
10. hash both committed fixtures and prove they remain unchanged;
11. compare released Task 22I full checkpoint bytes before/after the neutral
    split and before/after J implementation;
12. prove generated default/J bindings contain exactly the approved export
    sets and no legacy G/H/I hook.

Ignored evidence is manually audited but never a build, test, or runtime input.

## Mandatory independent review loop

The final six-axis reviewer is one dedicated read-only thread. It receives the
exact fixed commit/tree, source citations, approved documents, 34-path
manifest, per-file hashes, test commands/results, browser artifacts, oracle
facts, and known deferred scope. It does not edit files.

Its report must have six explicit sections:

1. requirement completeness;
2. logical correctness;
3. boundary and edge cases;
4. code quality and structural constraints;
5. test coverage and oracle independence;
6. actual native/WASM/browser execution.

Every finding includes severity, exact path/line, evidence, required repair,
and missing regression test. The main thread owns all edits. A repair round
invalidates prior execution evidence and candidate hashes; rerun and re-freeze
before revalidation. Continue until the same reviewer approves or a concrete
external blocker is documented with the exact failing command and output.

After that approval, fresh whole-candidate reviewers independently cover
specification compliance, fixed-source/Rust quality, direct default-model
reasoning, and documentation accuracy. No reviewer approval is inferred from a
test pass.

## Documentation and release

After Package E implementation approval and before the final candidate freeze,
append `docs/architecture/option-parity-v4.md` with the fixed source boundary,
single-implicit-range specialization, occurrence/source order split, sidecar ownership,
graph rules, Boolean closure, Internal surface decision, platform constraints,
and exact deferrals.

Append `docs/roadmap.md` with exact Task 22J exit evidence and the next
source-cited audit boundary. Do not rewrite approved spec/plan prose as a status
log; execution facts belong in the ignored evidence ledger and final
architecture/roadmap appendices.

Use the `conventional-commits` skill for the release commit. The expected
subject is `feat(slicing): port volume region composition`, adjusted only if
the final diff warrants a more precise conventional scope. Stage only approved
paths, verify the staged manifest, commit once, push the current branch by
normal fast-forward, and verify local HEAD, tracking ref, direct remote ref,
and GitHub Actions all point at the exact same SHA.

Task 22J is released only when all five exact-SHA Tier-1 jobs succeed. A CI
failure reopens the repair/review loop; do not waive, amend, or force-push.

## Stop conditions

Stop implementation and return to review if:

- fixed source contradicts any normative document statement;
- the complete fixed probe cannot reproduce both synthetic and KSR constants;
- any expected constant differs without independently approved fixed-source
  evidence;
- implementing J requires lifting `layer_config_ranges` or any current
  modifier usage gate;
- accepted painted or material configuration can reach J without a typed
  representation or explicit preflight;
- source order is replaced by occurrence-ID order in clipping priority;
- a Boolean wrapper needs an extra union, safety offset, or second engine;
- a test requires an out-of-band Option or fixture-specific production branch;
- a Rust production or test file reaches 400 LOC;
- a new tracked path is required outside the approved manifest;
- native and WASM results diverge;
- any P0-P3 review finding remains unresolved;
- exact-SHA Tier-1 fails.

A genuine blocker report must include the exact source boundary, command,
output, candidate manifest, attempted repair, and why continuing would violate
the approved requirements. Difficulty or elapsed time is not a blocker.

## Gate checklist

- [ ] Exact spec and plan hashes frozen
- [ ] Fixed-source/specification review approved
- [ ] Current-Ares/plan review approved
- [ ] Direct default-model document review approved
- [ ] Package 0 neutral split and complete checkpoint RED approved
- [ ] Package A exact Difference/Intersection RED/GREEN approved
- [ ] Package B full-mesh bounds and region graph approved
- [ ] Package C1 sidecar/dense surfaces/fast path approved
- [ ] Package C2 complex composition and synthetic oracle approved
- [ ] Package D real project stage and native KSR checkpoint approved
- [ ] Package E exact WASM/browser promotion approved
- [ ] All Rust files below 400 LOC and real modules verified
- [ ] Full native/WASM/browser matrix green on frozen candidate
- [ ] Six-axis repair/revalidation loop approved
- [ ] Fresh specification/quality/default/documentation reviews approved
- [ ] Architecture and roadmap updated and reviewed
- [ ] Conventional commit and normal push verified
- [ ] Exact-SHA Tier-1 all five jobs passed
- [ ] Next source-cited continuation boundary recorded
