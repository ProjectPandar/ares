# Task 22O.73 — base fill grouping

## Status

Implementation and exact-tree final verification are complete. The module
remains crate-private and lifecycle-inactive.
Specification date: 2026-08-13.

## Goal and source boundary

Port the admitted base of pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s `group_fills` from
`OrcaSlicer/src/libslic3r/Fill/Fill.cpp:216-346,829-1067` into one deep,
in-process Rust module. The slice owns complete parameter projection, exact
ordered-set identity, ordered grouping/coalescing, LockedZag sidecars, and
priority clipping before the two postpasses.

`Fill.cpp:1213-1224` and `1377-1397` prove two real callers: normal
`Layer::make_fills` and sparse-infill anchoring. O73 exposes their future shared
base but activates neither caller.

The direct source dependencies and included portions are:

- `Fill.cpp:52-59` for empty-template degrees-to-radians conversion;
- `FillBase.hpp:39-46,158-163` and `FillBase.cpp:91-102` for LockedZag maps and
  source pattern bridge-flow classification;
- `Surface.hpp:9-33,35-114`, `Layer.hpp:33-80,123-196`, and
  `ExPolygon.hpp:300-315` for surface predicates/metadata, region and surface
  order, no-overlap ownership, and contour-before-hole flattening;
- `PrintRegion.cpp:6-53`, `LayerRegion.cpp:21-58`, `Flow.hpp:13-119`, and
  `Flow.cpp:20-36,129-143,146-229` for role/extruder selection and exact Flow
  construction, ratio, spacing, equality, and ordering semantics;
- `PrintConfig.hpp:87-98,1074-1204`, `ExtrusionEntity.hpp:19-43`, and the
  reached float-or-percent conversion in `Config.hpp` for typed option values
  and explicit source enum ranks;
- `ClipperUtils.hpp:364-370` with `ClipperUtils.cpp:393-410` for safety union;
- `ClipperUtils.hpp:442-455` with `ClipperUtils.cpp:741-747` for safety
  difference; and
- `ClipperUtils.hpp:548-553` with `ClipperUtils.cpp:815-824` for plain
  no-overlap union.

## Destination interface

Add `project_slice::group_fills` with exactly one entry point:

```rust
pub(in crate::project_slice) fn group_fills_base(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<BaseGroupedFills, SliceError>;
```

`object_index` indexes aligned print-object occurrences, not source objects;
`layer_index` indexes the planned-layer and record slots. Invalid indices or
misaligned internal arrays are programmer errors. An aligned absent layer
returns an empty result. The prepared graph remains unchanged on success and
error.

The concrete result types are:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum SurfaceFillPattern {
    Configured(ProcessInfillPattern),
    ConcentricInternal,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct SurfaceFillParams {
    pub(in crate::project_slice) extruder: u32,
    pub(in crate::project_slice) pattern: SurfaceFillPattern,
    pub(in crate::project_slice) spacing: f64,
    pub(in crate::project_slice) overlap: f64,
    pub(in crate::project_slice) angle: f32,
    pub(in crate::project_slice) fixed_angle: bool,
    pub(in crate::project_slice) bridge: bool,
    pub(in crate::project_slice) bridge_angle: f32,
    pub(in crate::project_slice) density: f32,
    pub(in crate::project_slice) multiline: i32,
    pub(in crate::project_slice) anchor_length: f32,
    pub(in crate::project_slice) anchor_length_max: f32,
    pub(in crate::project_slice) flow: Flow,
    pub(in crate::project_slice) extrusion_role: crate::ExtrusionRole,
    pub(in crate::project_slice) role_speed: f32,
    pub(in crate::project_slice) lateral_lattice_angle_1: f32,
    pub(in crate::project_slice) lateral_lattice_angle_2: f32,
    pub(in crate::project_slice) infill_lock_depth: f32,
    pub(in crate::project_slice) skin_infill_depth: f32,
    pub(in crate::project_slice) symmetric_infill_y_axis: bool,
    pub(in crate::project_slice) infill_overhang_angle: f32,
    pub(in crate::project_slice) gyroid_optimized: bool,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct RepresentativeSurface {
    pub(in crate::project_slice) kind: RegionSurfaceKind,
    pub(in crate::project_slice) thickness: f64,
    pub(in crate::project_slice) thickness_layers: u16,
    pub(in crate::project_slice) bridge_angle: f64,
    pub(in crate::project_slice) extra_perimeters: u16,
}

pub(in crate::project_slice) struct SurfaceFill {
    pub(in crate::project_slice) region_id: usize,
    pub(in crate::project_slice) representative: RepresentativeSurface,
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
    pub(in crate::project_slice) params: SurfaceFillParams,
    pub(in crate::project_slice) region_id_group: Vec<usize>,
    pub(in crate::project_slice) no_overlap_expolygons: Vec<ExPolygon>,
}

pub(in crate::project_slice) struct LockDensityParam {
    pub(in crate::project_slice) density: f32,
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
}

pub(in crate::project_slice) struct LockFlowParam {
    pub(in crate::project_slice) flow: Flow,
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
}

pub(in crate::project_slice) struct LockRegionParam {
    pub(in crate::project_slice) skin_density_params: Vec<LockDensityParam>,
    pub(in crate::project_slice) skeleton_density_params: Vec<LockDensityParam>,
    pub(in crate::project_slice) skin_flow_params: Vec<LockFlowParam>,
    pub(in crate::project_slice) skeleton_flow_params: Vec<LockFlowParam>,
}

pub(in crate::project_slice) struct BaseGroupedFills {
    pub(in crate::project_slice) surface_fills: Vec<SurfaceFill>,
    pub(in crate::project_slice) lock_region_param: LockRegionParam,
    pub(in crate::project_slice) has_internal_voids: bool,
}
```

Do not add `idx`, representative geometry, public exports, setters, a generic
caller input, a trait, or a lifecycle state. `ConcentricInternal` is reserved
for the immediately following O74 source continuation and is not emitted by
O73. `SurfaceFillParams` must not implement derived equality, ordering, or
hashing.

## Projection requirements

Traverse regions numerically and surfaces in stored order. The current public
graph admits one region; multi-region representation and its algorithm remain
deferred behind `multi_region_layer_slices`.

For each surface:

1. Observe and skip `InternalVoid`, setting `has_internal_voids`.
2. Select source Flow role: top uses `frTopSolidInfill`, other solids use
   `frSolidInfill`, and sparse internal uses `frInfill`.
3. A surface is a bridge only when `layer_id > 0 && surface.is_bridge()`.
4. Begin with sparse pattern/density and overwrite them exactly for top,
   bottom, internal solid, and other solid/bridge source branches. Skip only
   nonpositive top density and nonpositive nonsolid sparse density at the
   source points.
5. Select the source extrusion role. Override the group filament for top,
   bottom, and nonbridge internal-solid roles, but retain the originally
   selected Flow role and its nozzle. Filament IDs remain one-based; subtract
   one only for nozzle lookup.
6. Set multiline from configuration only for `InternalInfill`; all other roles
   use one. Pass Gyroid optimization only for effective Gyroid.
7. Empty rotation templates convert the configured direction to radians with
   the source f64 then f32 cast. Apply stored model rotation with source f32
   arithmetic when alignment is enabled. `fixed_angle` remains false.
8. Keep `params.bridge` independent from `Flow.bridge`. Build standard or thick
   bridge Flow exactly, including bridge-width and bridge-flow-ratio behavior;
   otherwise resolve the actual role Flow at effective surface thickness.
9. Resolve role speed exactly, including internal-bridge percent against bridge
   speed. Solid and bridging fills use actual Flow spacing and 1000 mm anchors;
   sparse fill uses nominal object-layer-height `frInfill` spacing and source
   f32 anchor percent/clamp order, including `std::min` first-operand identity
   for comparator-equivalent signed zeros.
10. For LockedZag, append raw source geometry to all four density/Flow sidecars
    before grouping or priority clipping.

The single `SurfaceFillParams` working value persists across the whole layer.
Only LockedZag assigns lock and skin depths; only CrossZag, LockedZag, and
ZigZag assign symmetric-Y. Later patterns intentionally inherit those values,
matching the source.

`overlap` remains the source default zero. Fields whose reachable values are
correlated through Flow/options must still occupy their exact comparator slots;
do not add a test-only raw-params entry point to vary impossible combinations.

Nonempty `sparse_infill_rotate_template` or
`solid_infill_rotate_template` returns the existing option-key
`UnsupportedProjectFeature`. The source grammar and `rand()`/PRNG behavior at
`Fill.cpp:25-214` remain deferred; the old simple-list parser and host RNG are
not fallbacks.

## Ordered-set and coalescing requirements

Use a private key with a manual source comparator. Compare finite floats with
ordinary `<` and `>` so signed zero is equivalent. Use explicit matches for
the pinned pattern and extrusion-role ranks. The order is:

```text
bridge_angle descending;
extruder; pattern; spacing; overlap; angle; fixed_angle; density; multiline;
anchor_length; anchor_length_max;
flow.width; flow.height; flow.nozzle_diameter;
params.bridge; extrusion_role; role_speed;
lateral_lattice_angle_1; lateral_lattice_angle_2;
symmetric_infill_y_axis; infill_lock_depth; skin_infill_depth;
infill_overhang_angle; gyroid_optimized
```

Exclude Flow spacing, Flow bridge, Flow `mm3_per_mm`, and source `idx`. Two
params are equivalent only when neither precedes the other. Never use
`SurfaceFillParams::operator==`, Rust struct equality, `total_cmp`, a host enum
cast, or a HashMap.

Project and intern every surface before any coalescing union so a later
projection error precedes geometry errors. Materialize groups in comparator
order, then rescan source surfaces. Equivalent geometry appends in source
order. The first source member supplies representative metadata; replace its
bridge angle with `f64::from(params.bridge_angle)`. The representative contains
no ExPolygon, and `expolygons` is authoritative.

In the admitted single region, `region_id` is zero, `region_id_group` is
`[0]`, and no-overlap geometry is copied in source order. Do not implement a
region-zero fallback for a multi-region graph. Ordered first-seen region IDs,
per-new-region no-overlap union, and the graph representation needed to supply
them are a future source slice after the public gate is removed.

LockedZag density sidecars materialize in ascending source f32 order. Flow
sidecars materialize in ascending `mm3_per_mm` order; comparator-equivalent
entries retain the first complete Flow and append geometry in source order.

## Priority requirements

Process groups in comparator order and preserve every group slot:

1. Flatten each original ExPolygon as contour then holes.
2. If the first nonempty group has multiple subjects, call the exact safety
   union.
3. For every later nonempty group, safety-difference its raw polygons against
   all accumulated preceding raw polygons.
4. Append the current raw subjects, not clipped/unioned output, to the
   accumulator.
5. Leave the first singleton untouched; if a later group exists, still append
   its raw paths before processing that group.
6. Preserve empty results and Clipper output order. Never canonicalize
   production geometry.

## Error and ownership contract

Return `SliceError` directly and no partial `BaseGroupedFills`. The error phase
order is projection/LockedZag Flow, coalescing geometry, then priority geometry.
Map a reachable Clipper coordinate error to exactly:

```text
InvalidInput("fill-grouping polygon coordinate is outside the supported Clipper range")
```

Open-path errors are unreachable for closed grouped geometry. No catch,
continue, empty fallback, legacy infill scaffold, native-only implementation,
filesystem, terminal, UI, OpenGL, or unsafe code is permitted.

## Tests and oracle contract

All production-behavior tests must call `group_fills_base` through a prepared
graph. Test-only encoder grammar and manifest checks may exercise the encoder
directly. Do not expose private projection, comparator, context, or priority
helpers for tests.

Focused cases must cover reachable kind/role projection, exact first-layer
bridge behavior, top/sparse density skips, one-based filament versus nozzle
lookup, top automatic width, standard and thick bridges, independent
`params.bridge`/`Flow.bridge`, role-speed percent resolution, f32 angle and
anchor order, sticky LockedZag fields, explicit ranks and signed zero,
source-order coalescing, representative metadata, sidecar identity/order,
singleton/multi/prior/empty priority branches, natural range error atomicity,
absent layers, input immutability, and repeatability. The existing public
multi-region gate remains covered at its owner; O73 must not claim multi-region
parity.

The real-KSR pre-narrow acceptance result across 460 ordered layer slots must
match:

| measure | contract value |
|---|---:|
| groups | 477 |
| fill ExPolygons | 1,882 |
| fill holes | 174 |
| fill contour-plus-hole paths | 2,056 |
| fill points | 107,540 |
| no-overlap ExPolygons | 2,547 |
| metadata SHA-256 | `a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900` |
| canonical geometry SHA-256 | `062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af` |
| layer table SHA-256 | `ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721` |

Layer slots 260-459 remain present and empty. The metadata witness must retain
33 groups with `params.bridge == true` but only 22 with
`flow.bridge == true`. The geometry digest is an oracle-side canonical view,
not authorization to sort production results; known build-mode order-only
permutations must remain recorded as variants rather than erased.

The fill ExPolygon/hole/path/point totals exclude the no-overlap section. The
canonical geometry digest includes both fill geometry and all 2,547
no-overlap ExPolygons; the layer table and fill totals cover fill geometry
only.

The normative hashes replay O38's audited fixed-MSVC STL 14.44 direction-map
order. The Linux libstdc++ source variant is metadata
`25a9ddd67028354ff44607a59c04a065ffa74a99b9f1a05bdc7a1adb9c15dce7`,
canonical geometry
`136cca449aebb9d155fd51552f51a7bb3b2f5acb42702bd84b2d2920e265d1dc`,
and layer table
`f45a91b4f62dabae2f2320f936b8c903ee5d8e7d8db07fb9251418c82e832bf6`;
it is retained only as provenance. In particular, do not normalize the
source's oriented bridge angle modulo PI.

The fixed-MSVC O74 post-narrow values—536 groups, 2,218 fill ExPolygons, 152
fill holes, 2,370 fill paths, 110,610 fill points, metadata
`cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387`,
geometry
`c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c`, and
table `8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2`—
are a negative boundary comparison for O73, not its acceptance target. The
Linux post-narrow provenance is metadata
`36aecdaf4d3bfb8dadcaf63a0d0d39f3a12ad9b0b0e1aad0c5a9ceab19ef2eff`,
canonical geometry
`13d36da11e01e99840b1cf058003ad18c26c29bd8d6bb0d33af23c1b2ce4534c`,
and layer table
`15dd3f792d2a9176630e30c2170487c872a9b94eb637fdb6eb6a2841667ece5a`.
The oracle contains explicit `assert_ne!` witnesses for the O74 aggregate
totals and each of its metadata, canonical-geometry, and layer-table hashes,
so the distinct post-narrow result cannot be accepted as O73.

## Files and lifecycle restriction

Production implementation is limited to:

- `crates/ares-core/src/project_slice.rs` — declare the module only;
- `crates/ares-core/src/project_slice/group_fills.rs` — entry and graph
  resolution;
- `crates/ares-core/src/project_slice/group_fills/{types,params,coalesce,priority}.rs`
  and the private `params/{projection,locked}.rs` shards;
- `crates/ares-core/src/project_slice/perimeters/flow.rs` — declare the fill
  Flow shard; and
- `crates/ares-core/src/project_slice/perimeters/flow/fill.rs` — exact fill
  role and bridge Flow construction through `FillFlowContext`,
  `FillFlowRole::{Infill, Solid, Top}`, `resolve_fill_flow`,
  `resolve_fill_bridge_flow`, and `resolve_configured_fill_flow`.

Tests belong under
`crates/ares-core/src/project_slice/tests/prepare_infill/group_fills.rs` and
`tests/prepare_infill/group_fills/`, registered by
`tests/prepare_infill.rs`. Split any planned file before it reaches 400 LOC;
fill-role Flow behavior belongs in the existing shard rather than inline
growth of `perimeters/flow.rs`.

O73 must not alter `slice_project_sync`, the O72 incomplete sink, prepared
lifecycle types, O46 production wiring, Cargo features, public exports, or
legacy `infills` modules. O46 replacement and `Layer::make_fills` activation
remain blocked until O74 ports `Fill.cpp:349-827,1069-1186` and produces the
complete full-group result.

## Final exact-tree evidence

- Focused `task22o73` Nextest passed 19/19 with 6,451 skipped.
- Prepare-infill Nextest passed 277/277 with 26 slow and 6,193 skipped.
- Workspace Nextest passed 6,508/6,508 with 27 slow and two configured skips.
- Strict workspace all-target/all-feature Clippy with `-D warnings`,
  `cargo fmt --all -- --check`, and `git diff --check` passed.
- All six Tier-1 checks passed: core and adapter WASM, core Windows x86_64 and
  aarch64, and core macOS x86_64 and aarch64.
- The staged path count was zero; neither Cargo manifest nor `Cargo.lock`
  changed; forbidden-production and lifecycle/static scans were clean.
- Pinned OrcaSlicer was clean at exact commit
  `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
- Every changed or new Rust file remained below 400 LOC. The maximum changed
  file was `project_slice.rs` at 381 LOC; the maximum new production shard was
  `group_fills/params/projection.rs` at 369 LOC.
- Independent source/specification and standards rereviews closed
  unconditionally.

Thirty-one compiling behavioral mutations were killed and byte-exactly
restored. One additional compiling contour/hole insertion-order mutation
survived because it is behaviorally equivalent for normalized valid
ExPolygons; it is recorded as an equivalent survivor, not as a kill.

| production file | restored SHA-256 |
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
