# Task 22O.56 architecture decision record

## Status

Accepted, implemented, gate-verified, and independently reviewed.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `PrintObject.cpp:3160-3179`, the complete same-cluster lower-bridge subtraction block inside the reviewed `3160-3187` window of `bridge_over_infill`. Line 3181's expansion and lines 3183-3187's current-layer gathering are adjacent deferred behavior. Direct geometry closure is flat `diff(Polygons, Polygons)` at `ClipperUtils.hpp:42-154,430-432`, `ClipperUtils.cpp:304-335,671-679`, `Polygon.hpp:274-281`, and bundled Clipper provider/range/result execution at `deps_src/clipper/clipper.hpp:96-99,332-381` and `clipper.cpp:603-615,756-809,1072-1086,2779-2798`. `EPSILON` is pinned at `libslic3r.h:52`.

The Rust destination is private `bridge_over_infill/lower_cluster_subtraction.rs`:

```rust
pub(in crate::project_slice) struct ClusterBridgeHistoryLayer<'a> {
    pub print_z: f64,
    pub candidates: &'a [CandidateSurface],
}

pub(in crate::project_slice) fn subtract_filled_lower_cluster_bridges(
    deep_infill_area: &[Polygon],
    previous_cluster_layers: &[ClusterBridgeHistoryLayer<'_>],
    current_print_z: f64,
    target_flow_height: f64,
) -> Result<Vec<Polygon>, ClipperError>;
```

The caller supplies only earlier jobs from the current O54 cluster, in oldest-to-newest order. Those candidates are the surviving O55-order subsequence after each earlier job has replaced its geometry at `PrintObject.cpp:3304-3308`; current pre-expansion O43/O55 inventory is not valid production history. `target_flow_height` is the already-promoted source value `f64::from(current_o55_front_candidate_region_o48_flow.height * 0.9_f32)`. Selecting the current O55-front candidate's region, resolving its O48 Flow, multiplying in f32, and constructing postprocessed history remain deferred to the transaction composer, so O56 is an unwired synthetic-input seam.

## Required semantics

Compute `bottom_z = (current_print_z - target_flow_height) - 1e-4_f64` in source order. Walk previous cluster layers in reverse. Include a layer while `print_z >= bottom_z`; append every candidate's `new_polygons` in retained O55 candidate order and each candidate's polygon order. At the first layer strictly below `bottom_z`, break; do not inspect any older layer. Because trusted history is ascending in Z, replacing `break` with `continue` is output/error equivalent and is retained as a structural source invariant rather than a behavioral mutation claim.

Execute exactly one flat nonzero `difference_polygons_paths(deep_infill_area, filled_lower_polygons)` after gathering. This call is unconditional even when history or the flattened clip list is empty; do not substitute a clone/early return because Clipper normalization and ordered output remain observable. Return the first closed-path coordinate error atomically. Inputs are borrowed and unchanged; output is owned.

The trusted domain is finite source-ordered Z values, finite positive f64 target height produced by the documented f32 multiplication/promotion, source-valid closed polygons, representable Z arithmetic, and signed coordinates valid for Ares input adaptation except deliberate boundary-error tests. No validation, fallback, saturation, host sort, filesystem access, or platform branch is added.

## Consequences

O56 supplies only the current-cluster history exclusion dependency. O47 gathering, O48 candidate-region height composition, the following 1.5-spacing expansion, current-layer area/anchor gathering, O46/O49/O51/O53 composition, collision rerun, postprocessing, candidate commit, TBB/time-limit/debug adapters, prepared successor, public lifecycle, surface rewrite, extrusion, motion, G-code, and CLI parity remain deferred.

Register private `mod lower_cluster_subtraction;` with ordinary test children. Every production/test source must contain at most 399 lines. `include!`, `include_bytes!`, and `include_str!` are prohibited for source splitting.

## Completion evidence

A removed actual-source driver linked the fixed-MSVC Clipper-order Debug archive `ec47c8b945656e0d52f7223234b80ec66068b4a9c671e71daa5030e049e2a41b` through command graph `ce98084159ce6e6dfb6c10de3514ce9578e0b186c5d6550157db0412a7cd3d90`. It froze empty-clip normalization, candidate/polygon order, one-call flattening, hole/component flat order, and target/bottom-Z bits. Driver/object/binary/output SHA-256 values were `0bc6b7a1b1d14585f1bae64a0d4f134b40f2dd5381d6d5b9471ba6787f5662fe`, `4ce836cbe50340762c590bf1a1b8385cae3e46b9a9f804292ab3ae67b93261f2`, `26657038af1cbfe5944450745d07cac3e0a28cc095039d9d4af2f0bf39057126`, and `7d1c0bc23214e8ba0b30442f9caa4a6978bdc1679e16cd9db2817b17db750a49`.

Final gates pass focused 10/10, O43-O56/Clipper/Flow dependency 683/683, workspace 6,343/6,343, warning-denying Clippy, core/browser wasm32, both Windows and both macOS targets, rustfmt, diff, LOC, static, clean-Orca, and no-staged checks. Nine behavioral mutations plus the structural break mutation are killed; production restores byte-exact to SHA-256 `706aacabc22d70977ee51c4988614b1696309bc6660b64b6dc7d7d23836d099f`.
