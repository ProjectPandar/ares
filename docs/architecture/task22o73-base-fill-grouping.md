# Task 22O.73 architecture decision record

## Status

Accepted and verified. The implementation is complete at the exact-tree
checkpoint and remains crate-private and lifecycle-inactive.
Decision date: 2026-08-13.

## Upstream boundary

Port the base grouping portion of pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/Fill.cpp:216-334` — `SurfaceFillParams`, its
  strict-weak ordering, and its distinct, unused-by-grouping equality;
- `src/libslic3r/Fill/Fill.cpp:336-346` — the source-shaped `SurfaceFill`
  result;
- `src/libslic3r/Fill/Fill.cpp:829-1067` — region/surface parameter
  projection, `InternalVoid` observation, LockedZag sidecars, comparator-
  equivalent interning, source-order coalescing, no-overlap ownership, and
  priority union/difference; and
- `src/libslic3r/Fill/Fill.cpp:1213-1224,1377-1397` — the two real callers,
  `Layer::make_fills` and sparse-infill anchoring.

The directly reached source dependencies are:

- `Fill/Fill.cpp:52-59,208-213` for empty-template angle conversion and the
  adjacent nonempty-template behavior that remains gated;
- `Fill/FillBase.hpp:39-46,158-163` and `Fill/FillBase.cpp:91-102` for
  `LockRegionParam` and pattern bridge-flow selection;
- `Surface.hpp:9-33,35-114` and `Layer.hpp:33-80,123-196` for surface
  classification, metadata, Layer/LayerRegion order, fill surfaces, and
  no-overlap geometry;
- `PrintRegion.cpp:6-53` and `LayerRegion.cpp:21-58` for one-based filament
  selection, role Flow, first-layer width selection, and standard/thick bridge
  Flow;
- `Flow.hpp:13-119` and `Flow.cpp:20-36,129-143,146-229` for Flow fields,
  equality and ordering distinctions, automatic widths, spacing, bridge
  threads, flow-ratio adjustment, and `mm3_per_mm`;
- `PrintConfig.hpp:87-98,1074-1204`, `ExtrusionEntity.hpp:19-43`, and
  `Config.hpp:1178,1284` for exact pattern/role ranks and reached typed option
  semantics;
- `ExPolygon.hpp:300-315` for contour-before-holes flattening;
- `ClipperUtils.hpp:364-370` with `ClipperUtils.cpp:393-410` for safety union;
- `ClipperUtils.hpp:442-455` with `ClipperUtils.cpp:741-747` for safety
  difference; and
- `ClipperUtils.hpp:548-553` with `ClipperUtils.cpp:815-824` for plain
  no-overlap union.

This is an upstream rewrite slice, not an Ares-owned fill scheduler.

## Rust destination seam

Add one graph-native, crate-private module at `project_slice::group_fills`:

```rust
pub(in crate::project_slice) fn group_fills_base(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<BaseGroupedFills, SliceError>;
```

`PreparedPostExternalSurfaces` is the smallest common prepared graph owned by
both source-shaped call paths. O46 already holds it, while post-O72 can borrow
it through the owned predecessor chain. Accepting
`PreparedPostInfillCombination` would prevent the earlier O46 caller from
reusing the implementation; accepting the horizontal-shell record owner would
permit pre-external calls and expose a storage detail.

The interface is graph-native deliberately. A caller-built view would expose
and duplicate traversal alignment, resolved object/region options, nozzle and
initial-width selection, transform rotation, planned-layer history, scale,
surface records, and no-overlap ownership. All dependencies are in-process, so
no trait, port, adapter, callback, global state, or test-only parameter seam is
added.

`group_fills_base` borrows the complete graph and returns one owned result. It
does not mutate the graph, create `PreparedPostGroupFills`, consume O72, or
advance the public lifecycle.

## Result interface

`BaseGroupedFills` owns:

- ordered `surface_fills: Vec<SurfaceFill>`;
- the four ordered LockedZag maps represented as
  `lock_region_param: LockRegionParam`; and
- `has_internal_voids: bool`, the source local needed by the deferred
  continuation.

Each `SurfaceFill` owns `region_id`, complete `SurfaceFillParams`, ordered
authoritative `expolygons`, ordered-unique `region_id_group`, and
`no_overlap_expolygons`. Its representative surface is metadata-only:
kind, thickness, thickness-layer count, bridge angle, and extra-perimeter
count. It must not expose an empty or stale representative ExPolygon after the
source move; `SurfaceFill::expolygons` is the only authoritative grouped
geometry.

`SurfaceFillPattern` is slice-private and has
`Configured(ProcessInfillPattern)` plus `ConcentricInternal`. The latter is not
emitted by O73, but it is the immediately required source value for O74's
narrow-solid continuation. It must not be added to the user-configurable
process enum. The incomplete legacy public `InfillPattern` is not a fallback,
and source-only `ipSupportBase` and sentinel `ipCount` are not admitted result
values.

`SurfaceFillParams` exposes the source fields except implementation-only
`idx`. It must not derive `Eq`, `Ord`, or `Hash`. A private key implements the
source comparator because comparator equivalence, not
`SurfaceFillParams::operator==`, is the grouping identity.

## Ordering decision

The private comparator uses ordinary source `<` and `>` comparisons, never
`f32::total_cmp`, enum discriminants, derived ordering, or hashing. Thus
`-0.0` and `+0.0` remain equivalent. Under the already validated finite-option
domain, its exact order is:

1. `bridge_angle` descending;
2. one-based `extruder`;
3. explicit pinned `InfillPattern` rank;
4. spacing, overlap, angle, and fixed-angle flag;
5. density, multiline, anchor length, and maximum anchor length;
6. Flow width, height, and nozzle diameter;
7. `params.bridge` and explicit pinned `ExtrusionRole` rank;
8. role speed and both lateral lattice angles;
9. symmetric-Y flag, lock depth, skin depth, overhang angle, and Gyroid flag.

The comparator excludes `idx`, Flow spacing, Flow bridge state, and
`mm3_per_mm`. In particular, `params.bridge` and `flow.bridge` are independent
fields. The source equality operator additionally differs by comparing the
complete Flow and omitting symmetric-Y; it must not be used for interning.

Projection preserves source region then surface order and source cast points.
The single `params` value is reused for the entire layer: lock depth, skin
depth, and symmetric-Y retain the source's conditional, sticky assignment.
The misleading `SurfaceFillParams` comment does not change runtime behavior:
`PrintRegion::extruder` returns a one-based selector, and subtraction occurs
only for nozzle lookup.
Sparse anchors retain the source f32 cast, percent multiplication, and clamp
order, including `std::min` first-operand identity for comparator-equivalent
signed zeros.

Two phases preserve source error and operation order. Projection and key
interning complete first. Groups are then materialized in comparator order and
the input is rescanned so equivalent geometry remains in source order. The
first equivalent surface supplies representative metadata; its bridge angle
is the projected f32 value promoted to f64.

## Geometry decision

For the currently admitted single-region graph, `region_id` and
`region_id_group` remain source-shaped and no-overlap polygons are copied from
that region. The generic multi-region graph representation, ordered region
joining, and per-new-region no-overlap union remain deferred behind the
existing public `UnsupportedProjectFeature("multi_region_layer_slices")`
gate. O73 must neither duplicate that public check nor silently substitute
region zero.

Priority processing flattens each ExPolygon as contour followed by holes. The
first group with multiple subjects uses a safety union. Every later nonempty
group is safety-differenced against all preceding raw polygons. The raw
subjects, never clipped output, are appended to the predecessor accumulator.
A first singleton is untouched, group slots clipped to empty are retained, and
production geometry is never canonicalized or sorted after Clipper.

LockedZag density maps use ordinary ascending f32 comparison. LockedZag Flow
maps use the source `Flow::operator<`, which compares only `mm3_per_mm`; the
first comparator-equivalent Flow is retained. Their raw source ExPolygons
append in traversal order before priority clipping.

The implementation is `O(S log G)` for `S` projected surfaces and `G` unique
keys, plus Clipper work. It owns only result and working geometry, retains raw
priority subjects, and is deterministic, parallel-call safe, and Tier-1/WASM
compatible.

## Errors and atomicity

Invalid object/layer indices and broken prepared-array alignment are internal
programmer errors, not user-input variants. An aligned absent layer returns an
empty result. The existing public option/materialization boundaries own input
validation.

Nonempty sparse or solid rotation templates remain explicit
`UnsupportedProjectFeature` results until their complete pinned grammar and
PRNG behavior at `Fill.cpp:25-214` is ported. O73 must not call the older
simple-list parser or add a host RNG fallback. Flow resolution retains its
existing `SliceError` categories. A reachable Clipper coordinate failure maps
exactly to:

```text
fill-grouping polygon coordinate is outside the supported Clipper range
```

Closed-path grouping makes open-path Clipper errors unreachable. Error order is
projection/LockedZag Flow, then coalescing geometry, then priority geometry.
Any error drops local work, returns no partial result, and leaves the borrowed
graph unchanged.

## Included and deferred

Included: complete admitted-domain parameter projection; explicit pattern and
role ranks; exact comparator identity and output order; source-order
coalescing; metadata-only representative ownership; InternalVoid observation;
LockedZag sidecars; single-region no-overlap state; and base priority clipping
through `Fill.cpp:1067`.

Deferred to O74: `Fill.cpp:349-827,1069-1186`, including InternalVoid repair,
narrow-solid detection and splitting, and assignment of
`ipConcentricInternal`. KSR has no reached InternalVoid repair, but enables
`detect_narrow_internal_solid_infill`, so O74 is mandatory before production
activation.

Also deferred: nonempty rotation-template grammar/PRNG; multi-region graph and
coalescing; active infill combination; fill-generator dispatch and extrusion
entities after `make_fills`; adaptive/Lightning generator ownership; motion,
G-code, CLI, and complete golden parity.

O73 does not replace O46. The current O46 private reduced grouping remains a
temporary compatibility shell until O74 completes `group_fills`; only then may
both `Layer::make_fills` and the anchoring caller move to the shared module and
the reduced O46 grouping be deleted. No legacy scaffold may be called as a
fallback.

## Final verification evidence

Production-behavior tests must cross `group_fills_base`, not private
comparator, projection, or geometry helpers. Test-only encoder grammar checks
may exercise the encoder directly. The real-KSR pre-narrow acceptance
checkpoint must preserve all 460 ordered layer slots, including empty layers
260-459, and match:

```text
477 groups
1,882 fill ExPolygons
174 fill holes
2,056 fill contour-plus-hole paths
107,540 fill points
2,547 no-overlap ExPolygons
metadata SHA-256 a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900
canonical geometry SHA-256 062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af
layer table SHA-256 ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721
```

The fill ExPolygon/hole/path/point totals exclude the no-overlap section. The
canonical geometry digest includes both fill geometry and all 2,547
no-overlap ExPolygons; the layer table and fill totals cover fill geometry
only.

These normative hashes replay O38's audited MSVC STL 14.44 bridge-direction
order. A Linux libstdc++ source run instead produced metadata
`25a9ddd67028354ff44607a59c04a065ffa74a99b9f1a05bdc7a1adb9c15dce7`,
canonical geometry
`136cca449aebb9d155fd51552f51a7bb3b2f5acb42702bd84b2d2920e265d1dc`,
and layer table
`f45a91b4f62dabae2f2320f936b8c903ee5d8e7d8db07fb9251418c82e832bf6`;
those are nonnormative platform variants, not authorization to normalize
bridge angles or production order.
The canonical geometry digest is test evidence only. Production must retain
the source result and must not canonicalize to satisfy a hash.

The post-narrow O74 comparison is deliberately not an O73 success value: 536
groups, 2,218 fill ExPolygons, 152 fill holes, 2,370 fill paths, 110,610 fill
points, metadata
SHA-256 `cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387`,
canonical geometry SHA-256
`c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c`,
and layer-table SHA-256
`8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2`.
The Linux libstdc++ post-narrow provenance is metadata
`36aecdaf4d3bfb8dadcaf63a0d0d39f3a12ad9b0b0e1aad0c5a9ceab19ef2eff`,
canonical geometry
`13d36da11e01e99840b1cf058003ad18c26c29bd8d6bb0d33af23c1b2ce4534c`,
and layer table
`15dd3f792d2a9176630e30c2170487c872a9b94eb637fdb6eb6a2841667ece5a`;
it is nonnormative for the same predecessor tie-order reason as the O73 PRE
variant. The oracle contains explicit `assert_ne!` witnesses for the O74
aggregate totals and each of its metadata, canonical-geometry, and layer-table
hashes, so the distinct post-narrow result cannot be accepted as O73.

Final exact-tree verification passed:

- the focused `task22o73` Nextest band passed 19/19 with 6,451 skipped;
- the prepare-infill dependency band passed 277/277 with 26 slow and 6,193
  skipped;
- workspace Nextest passed 6,508/6,508 with 27 slow and two configured skips;
- strict workspace all-target/all-feature Clippy with `-D warnings`,
  `cargo fmt --all -- --check`, and `git diff --check` passed;
- all six Tier-1 checks passed: core and adapter WASM, core Windows x86_64 and
  aarch64, and core macOS x86_64 and aarch64;
- the staged path count was zero, neither Cargo manifest nor `Cargo.lock`
  changed, and the forbidden-production and lifecycle/static scans were clean;
- pinned OrcaSlicer was clean at exact commit
  `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
- every changed or new Rust file stayed below 400 LOC: the maximum changed file
  was `project_slice.rs` at 381 LOC and the maximum new production shard was
  `group_fills/params/projection.rs` at 369 LOC; and
- independent source/specification and standards rereviews closed
  unconditionally.

Thirty-one compiling behavioral mutations were killed and byte-exactly
restored. One additional compiling contour/hole insertion-order mutation
survived because it is behaviorally equivalent for normalized valid
ExPolygons; it is recorded as an equivalent survivor, not as a kill. The
restored production SHA-256 values are:

| production file | SHA-256 |
|---|---|
| `project_slice/group_fills.rs` | `1e0c8bb628a7e587fc5a8adbb81313083db49af5c33c9c075e7bef018683f5d3` |
| `project_slice/group_fills/coalesce.rs` | `71b16ca2b2d4024cd597bd8c48964bf55e9fd8b86d49d43c82fd0fa18d1491ae` |
| `project_slice/group_fills/params.rs` | `7a3b73dd1d12a0df6dbaa53f32d04c20ebe2388f4ffa7cff79031c57d9282088` |
| `project_slice/group_fills/params/locked.rs` | `dbba0d22889347f61b11024bdcda9345cbe7340d3054bc89d5b0f287007bf020` |
| `project_slice/group_fills/params/projection.rs` | `9fac547764b34d70434db46a854ef46a2cc796d6d1aa60c967adb1a2fbf00638` |
| `project_slice/group_fills/priority.rs` | `83df27b3d976b4b5701d8a061f16a03447f6d7d3cbff8b19d99dfd82937eb4dd` |
| `project_slice/group_fills/types.rs` | `2916cc6bdd2f02175c14ca4fafc1265866b65f675a4b0bbf47edd81b160e7eb3` |
| `project_slice/perimeters/flow.rs` | `7d5138ef9c369f2872ad184e89ebd21e18eaf2867a730e1ed99bce1fe566ace3` |
| `project_slice/perimeters/flow/fill.rs` | `43837d725862a580d325cb2c53eb9ceb37fe1ca37121dfd67832066b3763ca6c` |

Every new or changed Rust source file must remain below 400 LOC. The module is
implemented as a small facade plus `types`, `params`,
`params/{projection,locked}`, `coalesce`, and `priority` shards; fill-role Flow
behavior belongs in the dedicated
`perimeters/flow/fill.rs` shard rather than growing `perimeters/flow.rs` past
the repository limit.

## Next upstream owner

O74 owns pinned `Fill/Fill.cpp:349-827,1069-1186`: InternalVoid repair and the
KSR-active narrow-internal-solid continuation. Only the completed base plus
that tail may become the shared full `group_fills` used by O46 replacement and
future `Layer::make_fills` lifecycle work.
