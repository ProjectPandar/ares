# Task 22O.56 — lower-cluster bridge subtraction

## Status

Implemented and verified. Independent source/specification review approved the boundary before RED.

## Goal and source boundary

Port pinned `PrintObject.cpp:3160-3179`, which removes bridge expansion already filled by earlier jobs in the same O54 cluster from the current deep sparse area. The reviewed window continues through 3187, but line 3181 expansion and lines 3183-3187 current-layer gathering are deferred. Direct closure is `libslic3r.h:52::EPSILON`, flat `diff` in `ClipperUtils.hpp:42-154,430-432`, `ClipperUtils.cpp:304-335,671-679`, `Polygon.hpp:274-281`, bundled `clipper.hpp:96-99,332-381`, and `clipper.cpp:603-615,756-809,1072-1086,2779-2798`.

Destination: ordinary private `prepare_infill/bridge_over_infill/lower_cluster_subtraction.rs` and ordinary test children.

## Interface

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

History contains only earlier jobs in the current cluster, oldest to newest, after each earlier job's `3304-3308` candidate replacement; pre-expansion O43/O55 candidates are not valid production history. The caller supplies `f64::from(current_o55_front_candidate_region_o48_flow.height * 0.9_f32)`. Region/Flow selection, f32 multiplication, same-cluster/current-job exclusion, and postprocessed history construction are transaction-composer responsibilities deferred from O56.

## Behavior

1. Compute `(current_print_z - target_flow_height) - 1e-4_f64` without reassociation.
2. Traverse history newest to oldest.
3. Include equality and every layer above the boundary. At the first strict-below layer, break immediately.
4. Flatten included candidates newest-layer first, preserving each layer's O55 candidate order and each candidate's polygon order.
5. Invoke one flat NonZero difference with the borrowed deep area as subject and the flattened list as clip, even when either list is empty. Return its ordered owned Paths result.
6. Propagate the first `CoordinateOutOfRange`; open-path errors are unreachable. Leave every borrowed field and allocation unchanged.

## Trusted domain and deferrals

Z values are finite and source ordered; target height is finite positive f64 with the documented promoted f32-product provenance; subtraction is representable; polygons are valid closed source geometry. No public validation is added. Deliberate Clipper range tests cross only the external engine boundary.

Deferred: O47/O48 caller composition, expansion by `spacing * 1.5`, current-layer expansion/fill/top/lightning gathering, anchor intersection, O46/O49/O51/O53 composition, collision rerun, postprocessing, candidate commit, TBB/time-limit/debug adapters, prepared successor/lifecycle activation, surface rewrite, extrusion, motion, G-code, and CLI parity.

## Acceptance

Start with compiling RED. Build a removed temporary pinned-source Clipper driver using ARD-0024's audited MSVC STL 14.44 compatibility ordering, and freeze exact flat outputs, ordering, and boundary decisions as literals. Record command/source/object/binary/output hashes; tests have no runtime source dependency.

Tests must discriminate:

- empty deep area, empty history, and history with no candidate polygons while still executing difference normalization;
- newest-to-oldest layer traversal, candidate order, polygon order, and one-call flattening;
- exact bottom-Z equality/one-ULP below/above, exact promoted f32-product literal bits supplied as f64, and epsilon subtraction order;
- newest-to-oldest inclusion and structural break-on-first-below; under ascending trusted history, `break` versus `continue` is explicitly output/error equivalent;
- synthetic history contract snapshots without claiming this helper can prove same-cluster/current-job exclusion;
- holes/components and ordered flat difference output;
- first natural coordinate-range error, complete input/allocation nonmutation, and independent repeatability.

Run reversible mutations for early clone/return, comparison strictness, traversal direction, epsilon reassociation, candidate/polygon reorder, repeated per-layer difference, subject/clip reversal, and ExPolygon/hierarchical replacement; restore byte-exact. Audit `break` structurally. Defer region selection, f32 multiplication, same-cluster ownership, and current-job exclusion mutations to the transaction composer.

Private module registration and ordinary test children are required; every file contains at most 399 lines. `include!`, `include_bytes!`, and `include_str!` are prohibited for source splitting. Final gates: focused O56, O43-O56/Clipper/Flow dependency, workspace, rustfmt, strict Clippy, wasm32, Windows/macOS, diff/LOC/static/clean-Orca/no-staged gates, and independent six-axis repair/re-review until unconditional approval.

## Verification record

The removed pinned-source/fixed-MSVC-order driver and output hashes are `0bc6b7a1b1d14585f1bae64a0d4f134b40f2dd5381d6d5b9471ba6787f5662fe` and `7d1c0bc23214e8ba0b30442f9caa4a6978bdc1679e16cd9db2817b17db750a49`; Rust exact Paths and arithmetic literals match all records.

Focused 10/10, dependency 683/683, workspace 6,343/6,343, strict Clippy, wasm32, four Windows/macOS checks, and formatting/static/repository gates pass. Early return, strict boundary, traversal, epsilon reassociation, candidate reversal, polygon reversal, repeated difference, role reversal, and hierarchy mutations fail; structural break mutation is rejected. Restoration SHA-256 is `706aacabc22d70977ee51c4988614b1696309bc6660b64b6dc7d7d23836d099f`.
