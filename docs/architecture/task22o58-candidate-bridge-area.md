# Task 22O.58 architecture decision record

## Status

Accepted and completed. Independent source/specification and final six-axis reviews approved the implementation.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `PrintObject.cpp:3215-3224`, the complete per-candidate area preparation prefix of the bridge-over-infill expansion loop. The Rust destination is private ordinary module `prepare_infill/bridge_over_infill/candidate_bridge_area.rs`:

```rust
pub(in crate::project_slice) struct CandidateBridgeArea {
    pub area_to_be_bridge: Vec<Polygon>,
    pub limiting_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn prepare_candidate_bridge_area(
    candidate_polygons: &[Polygon],
    deep_infill_area: &[Polygon],
    internal_unsupported_area: &[Polygon],
    expansion_area: &[Polygon],
    scaled_spacing: Coord,
) -> Result<CandidateBridgeArea, ClipperError>;
```

The future composer owns loop orchestration at line 3213, resolves the candidate region's O48 Flow and `Flow::scaled_spacing()` at line 3214, then passes the current O55-ordered candidate's borrowed O43 geometry, O57 context fields, and resulting `Coord`. O58 does not search candidate provenance, resolve Flow, or perform mm-to-coordinate conversion.

## Required semantics

1. Cast strictly positive source `coord_t` spacing directly to f32 as `scaled_spacing as f32`, then expand all candidate polygons in one flat Miter/3 offset call.
2. Intersect the expanded candidates, as subject, with O57 deep area, as clip, in one flat NonZero intersection.
3. Traverse that ordered intersection result once. For each polygon, call flat intersection with a one-polygon subject and the complete internal-unsupported area as clip. Retain the polygon exactly when the result is nonempty. Preserve survivor allocations and order; return the first per-polygon engine error.
4. Build the two-argument source union input by cloning retained polygons first and appending expansion-area polygons in source order. Execute one unconditional flat NonZero union, even when the retained candidate list is empty.
5. Return owned retained and limiting paths even when the retained list is empty. The future loop composer applies source lines 3226-3227's empty `continue`; O58 must still compute the union first.
6. Preserve first-error precedence: candidate expansion, deep intersection, each survivor predicate in order, limiting union. Leave all borrowed inputs and allocations unchanged.

Direct closure is pinned `PrintObject.cpp:3215-3224` after composer-deferred `Flow.hpp:69` conversion at line 3214, flat offset defaults/implementation in `ClipperUtils.hpp:19,27,375-383` and `ClipperUtils.cpp:267-414`, flat intersections at `ClipperUtils.hpp:496-498` and `ClipperUtils.cpp:671-673,702-703`, two-vector union at `ClipperUtils.hpp:543-546` and `ClipperUtils.cpp:722-735`, contour Paths providers, and the inherited accepted Task 22F closed Boolean/PolyTree and Task 22G closed offset kernels. Source `std::remove_if` is the stable one-pass predicate traversal; no host sort or map is introduced.

The trusted domain is strictly positive source `coord_t` spacing with a finite positive direct f32 cast, source-valid closed flat polygons, and source-ordered inputs. Zero is out of domain and is not mapped to `ClipperError`. Deliberate range tests cross only the Clipper boundary.

## Consequences

O58 returns the candidate-local area and limiting union; the future composer consumes them and applies the deferred source empty gate at lines 3226-3227. Loop/Flow provenance at 3213-3214 and boundary-polyline construction 3229-3233, O49/O51 angle composition, O53 anchored construction, lightning clipping, collision rerun, postprocessing, expansion-area update, candidate replacement, surface rewrite, prepared successor/lifecycle activation, extrusion, motion, G-code, and CLI parity remain deferred.

Register private `mod candidate_bridge_area;` with ordinary test children. Every production/test source must contain at most 399 lines. `include!`, `include_bytes!`, and `include_str!` are prohibited for source splitting.

## Completion evidence

The removed actual-source driver built against fixed-MSVC-order archive SHA-256 `b643964e681e9435680b78fdd743dcb12c9c07cd16ef164e353d370add8132a1`. Driver/object/binary/output/link-command SHA-256 values are `1255ab783e35bc33e06844bf762cc7338303c56e73f53ca80175a993016f60f6`, `209f83c8d2f699827f201f880b99c744f0c5d532f9d9c9eaaa633dbb5b4393a0`, `923a769f1489997df87a9055d854f7e0bac9999e2630df280d81e61ea6594c9f`, `7701da24182d6cb4532bdaaafadf37ef5930c63f7f36a570c56de30f563eeec3`, and `a0564814087e754d1868be0022512540b5b3197001a74bd59fc699794405396b`. Two runs were byte-identical. The ordered source oracle freezes one expanded survivor and limiting union, empty-survivor union-before-continue behavior, split filtering, and spacing bits `0x1p+24`; Rust focused literals match it.

Fifteen reversible operation/role/order mutations were killed, including named repeated-union, deep-before-offset error-order, and union-before-predicate error-order variants, and production restored byte-exact at SHA-256 `7a3637253b9ae84dc50e7cabb35a4c83a555aab2259df9c36349296fcd6387f4`; the direct `scaled_spacing as f32` cast was statically confirmed with no f64 intermediate. Final gates pass: focused 10/10, dependency 708/708, workspace 6,368/6,368 (two skipped), strict Clippy, wasm32, x86_64/aarch64 Windows, x86_64/aarch64 macOS, rustfmt, diff/LOC/static checks, clean pinned Orca checkout, and no staged files.
