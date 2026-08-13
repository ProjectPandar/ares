# Task 22O.54 — bridge candidate-layer clustering

## Status

Implemented and verified. Independent source/specification review approved the boundary before RED.

## Goal and source boundary

Port pinned `PrintObject.cpp:2763-2818`, the layer clustering prepass that assigns potentially vertically/spatially interacting bridge candidates to one sequential worker cluster. Direct source closure is `Polygon.cpp:422-448`, `MultiPoint.cpp:89-92`, `BoundingBox.hpp:21,27-35,95-108,208-225`, `BoundingBox.cpp:15-30,94-105,204-211`, `Point.hpp:191-198`, `libslic3r.h:52,60-96`, `ClipperUtils.hpp:42-154,496-498,543-546`, `ClipperUtils.cpp:304-335,671-673,702-703,722-735`, bundled `deps_src/clipper/clipper.hpp:332-381` and `clipper.cpp:603-615,756-809,1072-1086`, and O48's source-cited thick solid bridge Flow height.

The Rust destination is ordinary `prepare_infill/bridge_over_infill/layer_clustering.rs` with ordinary test children. It stays crate-private and lifecycle-unwired.

## Interface

```rust
pub(in crate::project_slice) struct CandidateClusterLayer<'a> {
    pub layer_index: usize,
    pub print_z: f64,
    pub bridge_height: f32,
    pub candidates: &'a [CandidateSurface],
}

pub(in crate::project_slice) fn cluster_candidate_layers(
    layers: &[CandidateClusterLayer<'_>],
    scale: CoordinateScale,
) -> Result<Vec<Vec<usize>>, ClipperError>;

pub(in crate::project_slice) fn cluster_candidate_object(
    candidates: &BridgeCandidateObject,
    planned_layers: &[PlannedLayer],
    ordered_region_options: &[&RegionOptions],
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<Vec<Vec<usize>>, SliceError>;
```

The composition seam requires nonempty region options, selects only index zero, invokes O48 `resolve_thick_solid_infill_bridge_flow`, and projects its height into every candidate layer; candidate region provenance is irrelevant. It obtains each `print_z` by candidate layer index from `planned_layers`. The lower helper receives borrowed layers in ascending `surfaces_by_layer` order and does not sort or validate them. Returned cluster/layer order is exact and input is unchanged.

## Coverage geometry

1. Resolve the inflation coordinate exactly as source: pass f64 `scale_(7.0)` into `BoundingBox::inflated(coordf_t)`; `offset` constructs `Point(double,double)` and applies `std::round` to each component before integer subtraction/addition. The supported scales produce exact deltas 7,000,000 and 700,000.
2. Each layer starts with empty flat Polygons. For every candidate in order, merge all points from `new_polygons` into a source bounding box. Empty geometry leaves min/max `(0,0)` and `defined=false`; inflation still subtracts/adds the delta.
3. Emit each inflated box polygon in exact source order: `min`, `(max.x,min.y)`, `max`, `(min.x,max.y)`.
4. Sequentially apply source flat `union_(coverage, {box})`. Do not replace it with a single combined bounding box, raw rectangle list, ExPolygon hierarchy, host spatial index, or candidate reorder. Return the first Clipper error without partial externally visible output.

## Clustering

For each ordered layer:

1. Start a new cluster when this is the first layer.
2. Otherwise let `previous` be only the last layer index in the current cluster. Evaluate the strict source condition in order:
   `previous.print_z < current.print_z - f64::from(current.bridge_height * 0.9_f32) - 1e-4_f64`.
   The multiplication is f32 before promotion; no algebraic reassociation is allowed.
3. If the Z condition is false, intersect previous coverage with current coverage using source flat Paths intersection. Empty intersection starts a new cluster; nonempty intersection appends current to the existing cluster.
4. Earlier non-tail coverage is ignored. Preserve short-circuiting: a strict Z split does not run the intersection.

## Trusted domain and deferred behavior

Nonempty ordered region options, input layer order/uniqueness, finite `print_z`, finite positive region-zero bridge height, valid candidate geometry, and representable signed bbox/inflation arithmetic are internal preconditions supplied by the typed caller. Inflated closed paths may exceed bundled Clipper's narrower range and naturally return `CoordinateOutOfRange`. No defensive validation, sorting, fallback, saturation, or platform branch is added. Ares translates upstream Clipper exceptions into deterministic first lower-helper `ClipperError` and composition-seam `SliceError`; open-path errors are unreachable.

Deferred/omitted: the source TBB coverage-build adapter, its time-limit macro, and debug terminal output; `PrintObject.cpp:3114+` candidate sorting/transaction, per-layer deep sparse areas, boundary/anchor assembly, O46-O53 composition, collision rerun, postprocessing, region surface commit, successor/lifecycle activation, extrusion, motion, G-code, and CLI parity. O54 returns scheduling dependency data but does not implement a scheduler.

## Acceptance

Start with a compiling behavioral RED. Freeze literals from a temporary pinned-C++ driver using actual BoundingBox/Clipper helpers and fixed-MSVC Clipper ordering where output affects the result. Rust tests do not read/compile/run oracle artifacts, and all temporary artifacts are removed.

Tests must discriminate:

- empty layer list, one layer, multiple candidates, exact rectangle point/inflation order through a private test-visible rectangle/coverage helper, empty-candidate zero box, and both coordinate scales;
- sequential flat union versus one combined bbox/raw boxes, including disjoint candidates and flat output order where observable;
- strict Z boundary just below/equal/above, exact f32 `* 0.9f` before f64 promotion, and source `EPSILON` subtraction order;
- overlap versus edge-touch versus disjoint coverage, Clipper intersection semantics, and tail-only rather than all-cluster coverage;
- multi-region caller composition proving region-zero O48 height ownership;
- ascending input/output order, repeatability, complete nonmutation, and natural closed-path coordinate-range failure.

Z-before-intersection short-circuit ordering is frozen as a shared-helper structural/static gate; coordinate failure occurs during coverage construction before the Z decision, so it is not claimed as an intersection-only runtime witness. Run reversible mutations for 7-mm inflation/scale, empty-box behavior, bbox point order, sequential union, strict Z comparator, f32 multiplication/promotion, epsilon/order, production-seam region-zero selection/O48 ownership, tail ownership, intersection emptiness, and layer append order; restore byte-exact. Round-versus-truncate is source-documented but not claimed mutation-discriminable at the two supported exact deltas.

All production/test files remain below 400 LOC and use ordinary modules without `include!`, `include_bytes!`, or `include_str!` splitting. Final gates: focused O54, O43-O54/geometry/Flow dependency, workspace, rustfmt, warning-denying Clippy, wasm32, Windows/macOS cross-checks, diff/LOC/static/clean-Orca gates, and independent six-axis repair/re-review until unconditional approval.

## Verification record

The removed temporary pinned-source driver produced exact normal/large coverage and cluster literals. Its source/output SHA-256 values were `6a5ea622fbad7743e081d39671aaf4f0d9329c53a96891c1d0ab191d6d79de48` and `9b4b1c7921dd4c6b7394af2c182bb5657d35eb158f4b357a2f11fae1cf012514`; it linked the fixed-MSVC Clipper-order archive `ec47c8b945656e0d52f7223234b80ec66068b4a9c671e71daa5030e049e2a41b` through command graph `ce98084159ce6e6dfb6c10de3514ce9578e0b186c5d6550157db0412a7cd3d90`.

Focused 11/11, dependency 661/661, workspace 6,321/6,321, Clippy, wasm32, four Windows/macOS target checks, formatting and static gates pass. Mutations for inflation, empty bounds, point order, one-shot union, strict comparison, f64 promotion, epsilon reassociation, region selection, raw-nozzle bypass, ignored bridge width, ignored bridge-flow ratio, tail ownership, intersection polarity, append order, and eager intersection are all rejected; restoration SHA-256 values are clustering `1f6b463a0a81009236cd8ce4631e9ac0ce54e476b97f7381f91031d60439534c` and Flow `592faf4596a94a92bcf192db2d696ca101e585bb29b618316cb65561fd49f968`.
