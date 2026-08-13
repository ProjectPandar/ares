# Task 22O.54 architecture decision record

## Status

Accepted and implemented. Independent pre-RED review approved the source boundary; implementation repair/re-review evidence is recorded below.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `PrintObject.cpp:2763-2818`, the complete candidate-layer clustering prepass for `bridge_over_infill`. Direct helper closure is:

- `Polygon.cpp:422-448::get_extents(Polygons)` and `MultiPoint.cpp:89-92`;
- `BoundingBox.hpp:21,27-35,95-108,208-225`, `BoundingBox.cpp:15-30,94-105,204-211`, and `Point.hpp:191-198` for construction, undefined-zero bounds, defined-box merging, rounded inflation-vector construction, and polygon point order;
- `libslic3r.h:52,60-96::EPSILON/scale_`;
- `ClipperUtils.hpp:42-154,496-498,543-546`, `ClipperUtils.cpp:304-335,671-673,702-703,722-735`, and bundled `deps_src/clipper/clipper.hpp:332-381` plus `clipper.cpp:603-615,756-809,1072-1086` for provider insertion, flat union/intersection execution/result construction, and coordinate-range failure;
- `LayerRegion::bridging_flow(frSolidInfill, true).height()`, already resolved by O48's typed Flow path.

The Rust destination is a private `bridge_over_infill/layer_clustering.rs` operation over an ordered borrowed layer view:

```rust
struct CandidateClusterLayer<'a> {
    layer_index: usize,
    print_z: f64,
    bridge_height: f32,
    candidates: &'a [CandidateSurface],
}

fn cluster_candidate_layers(
    layers: &[CandidateClusterLayer<'_>],
    scale: CoordinateScale,
) -> Result<Vec<Vec<usize>>, ClipperError>;
```

A production composition seam also accepts the `BridgeCandidateObject`, planned layers, the object's ordered borrowed region-option view, nozzles, and scale. It selects region options index zero, invokes O48 `resolve_thick_solid_infill_bridge_flow`, projects that height plus planned `print_z` into the ordered layer views, calls `cluster_candidate_layers`, and adapts Flow/Clipper failures to `SliceError`. Candidate provenance is never used to select the flow. The functions return owned layer-index clusters and do not alter O43 candidates or activate a successor stage.

## Required semantics

For each candidate layer, start with empty flat Polygons. For every candidate in retained source order, compute `get_extents(candidate.new_polys)`: empty polygon sets keep the source undefined bounding box at zero. Pass `scale_(7.0)` as `coordf_t` into `BoundingBox::inflated`; `offset` constructs `Point(double, double)`, rounding each component with `std::round` before subtracting/adding it. At both supported scales this is exactly 7,000,000 or 700,000. Emit `[min, (max.x,min.y), max, (min.x,max.y)]`, and sequentially union that rectangle into layer coverage with the source flat Paths operation.

Traverse layers in ascending source-map order. Start a new cluster when no cluster exists, or when either condition is true:

1. `previous_cluster_tail.print_z < current.print_z - (current.bridge_height * 0.9f) - EPSILON`, where multiplication is f32 before promotion to f64 and `EPSILON` is source double `1e-4`;
2. the flat intersection of only the previous cluster tail's coverage and current coverage is empty.

Otherwise append to the current cluster. Geometry from earlier non-tail cluster layers is intentionally not consulted. The strict Z comparison, candidate/layer order, and flat Clipper output/error ownership are observable. Z-before-intersection short-circuit order is retained as a shared-helper structural/static invariant because coverage construction makes a natural intersection-only fault unattainable.

## Trusted domain and consequences

Inputs come from typed project data: nonempty ordered region options, unique ascending layer indices, finite `print_z`, finite positive bridge heights, source-valid candidate polygons, and representable signed bbox/inflation arithmetic. Inflated closed paths may nevertheless cross bundled Clipper's narrower coordinate range; that boundary yields `CoordinateOutOfRange`. Each `bridge_height` is a production-seam projection of the object's region-zero O48 flow, never a candidate-region flow. This internal precondition is not a public validation surface. Ares adapts upstream exception propagation to deterministic first `ClipperError`/`SliceError` return; open-path variants are unreachable here. Borrowed candidates remain unchanged.

O54 provides only deterministic sequential-thread schedule dependency data. The source TBB coverage-build adapter, time-limit macro, and debug-only terminal reporting are deliberately omitted; O54 creates neither a scheduler nor terminal behavior. Per-layer candidate sorting, sparse/deep area gathering, anchor/boundary assembly, O46-O53 composition, collision rerun, opening/closing/post-clips, surface rewrite/commit, prepared successor, public lifecycle, extrusion, motion, G-code, and CLI parity remain deferred.

Production/tests use ordinary modules, every source remains below 400 LOC, and source-splitting include macros are prohibited.

## Completion evidence

A temporary actual-source driver compiled against pinned headers at commit `8500fcdccaa10b5099ac20d252af3a7c560046f1` and the full fixed-MSVC Clipper-order Debug archive (`ec47c8b945656e0d52f7223234b80ec66068b4a9c671e71daa5030e049e2a41b`). It exercised sequential coverage, the undefined zero box, both scaling factors, and tail-only clustering. Provenance hashes were: command graph `ce98084159ce6e6dfb6c10de3514ce9578e0b186c5d6550157db0412a7cd3d90`, driver `6a5ea622fbad7743e081d39671aaf4f0d9329c53a96891c1d0ab191d6d79de48`, object `842a9c445f641127e8d69d0cec96c06fda6cf4d2ccb6c26cb8c00c1eb55328c7`, binary `113e9df64492354cc3b951d0237b0b4f528523258c311a13724fdc6caeb235c1`, and output `9b4b1c7921dd4c6b7394af2c182bb5657d35eb158f4b357a2f11fae1cf012514`. The Rust literals match that output exactly; temporary O54 artifacts were removed.

Final verification passes focused 11/11, O43-O54/Clipper/Flow dependency 661/661, and workspace 6,321/6,321 Nextest; warning-denying workspace Clippy; core/browser wasm32; both Windows and both macOS target checks; rustfmt, diff, LOC, static, clean pinned-Orca, and no-staged-file gates. Fourteen behavioral mutations plus the eager-intersection structural mutation are killed, including independent seam bypasses for raw nozzle height, ignored bridge width, and ignored bridge-flow ratio. Production restores byte-exact to clustering SHA-256 `1f6b463a0a81009236cd8ce4631e9ac0ce54e476b97f7381f91031d60439534c` and Flow SHA-256 `592faf4596a94a92bcf192db2d696ca101e585bb29b618316cb65561fd49f968`.
