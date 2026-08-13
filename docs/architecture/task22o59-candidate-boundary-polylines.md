# Task 22O.59 architecture decision record

## Status

Accepted and completed after independent source/specification review; final six-axis review is recorded below.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `PrintObject.cpp:3226-3233`, as the complete empty-candidate gate and boundary-polyline construction immediately after O58. The Rust destination is private ordinary module `prepare_infill/bridge_over_infill/candidate_boundary_polylines.rs`:

```rust
pub(in crate::project_slice) fn prepare_candidate_boundary_polylines(
    candidate_area: &CandidateBridgeArea,
    total_fill_area: &[Polygon],
    scaled_spacing: Coord,
    spacing: f32,
) -> Result<Option<Vec<Polyline>>, ClipperError>;
```

`None` is the source `continue` at lines 3226-3227. `Some` contains the source-ordered boundary polylines. The future composer owns candidate iteration and supplies the O58 result plus the current candidate region's O48 Flow-derived `scaled_spacing()` and `spacing()` values; O59 does not resolve provenance or options.

## Required semantics

1. Inspect only `candidate_area.area_to_be_bridge.is_empty()`. If empty, return `Ok(None)` before validating or calculating either scalar and before inspecting either later geometry input. Arbitrary scalar bits and invalid later geometry are ignored on this branch.
2. Promote `scaled_spacing: Coord` to f64, multiply by exact f64 literal `1.3`, then cast the product to f32 for one flat Miter/3 expansion of `total_fill_area`.
3. Consume the first ordered expanded polygon vector into closed polylines in order by moving each point allocation and appending its original first point exactly once. Source-valid offset output polygons are nonempty; no fallback or validation is added.
4. Promote source `spacing: f32` to f64, multiply by exact f64 literal `0.3`, then cast the product to f32 for one flat Miter/3 expansion of `candidate_area.limiting_area`.
5. Consume the second ordered polygon vector into closed polylines and append their values after all total-fill boundary polylines. Source `PrintObject.cpp:3231-3233` copies the local limiting polylines during iterator insertion; Rust may move those local polylines into the final vector because final allocation identity is not observable task behavior. Value, polygon order, point order, and total-before-limiting order are normative; final limiting allocation identity is explicitly not frozen. Do not sort, normalize, union, deduplicate, or interleave.
6. Propagate the first offset error in source order and leave all borrowed input fields and allocations unchanged.

Direct closure is pinned `PrintObject.cpp:3226-3233`, `Flow.hpp:64-69` (`spacing()` and `scaled_spacing()` types), `Polygon.hpp:220-246` (`to_polyline` and the selected rvalue `to_polylines(Polygons&&)`), `Polyline.hpp:16-23` (source copy/move constructors), and `ClipperUtils.hpp:19,27,331-379` plus `ClipperUtils.cpp:267-414` for default Miter/3 expansion. The accepted Ares closed offset kernel, `Polygon::into_points`, and `Polyline::new` are reused.

The nonempty branch's trusted domain is a source-valid O58 result, strictly positive candidate Flow spacing, composer-supplied `scaled_spacing > 0`, finite positive f64 products representable as positive f32 deltas, valid closed flat polygon inputs, and nonempty polygons returned by the offset kernel. The empty branch intentionally imposes no scalar or later-geometry precondition because source control flow reads none of them. Deliberate range errors cross only the Clipper boundary.

## Consequences

O59 closes the source empty gate and boundary preparation only. Candidate loop/Flow provenance at lines 3213-3214, debug-only drawing, anchor/non-anchor O51 dispatch at 3242-3250, O49 override, anchor append, lightning clipping, O53 construction, collision rerun, postprocessing, expansion mutation, candidate commit, surface rewrite, prepared successor/lifecycle activation, extrusion, motion, G-code, and CLI parity remain deferred.

Register only private `mod candidate_boundary_polylines;` with ordinary test children. Every production/test source must contain at most 399 lines. `include!`, `include_bytes!`, and `include_str!` are prohibited for source splitting. Portability gates cover Linux workspace, wasm32, x86_64/aarch64 Windows, and x86_64/aarch64 macOS.

## Completion evidence

The removed actual-source driver built against fixed-MSVC-order archive SHA-256 `b643964e681e9435680b78fdd743dcb12c9c07cd16ef164e353d370add8132a1`. Driver/object/binary/output/link-command SHA-256 values are `577738d4c8f00879276ac7815afe6d9b0ca80e834fe67899df46cfe30ccbd532`, `a3b74f48e04fb41b61f6694b95b510571bdc78c978ed9d1bfbe79358690b7106`, `9762c5531a462913e3f0476ce42fb2a0ef4d2f8d7e94353b21ed60aa6abb1296`, `9fa3600b73ebe60ac821a2f9811b606aae85ad6f6a1ee797cc4c5ffd62bda320`, and `98f57cf5675fca7419790c1a5b16b5d4aa97ae208cbd6b76692bdc42c262333b`. Repeated runs were byte-identical. Exact literals freeze the empty gate, ordered total-before-limiting polylines, acute Miter/3 output, and source arithmetic `0x1.4ccccep+24`, `0x1.421f62p+2`, and `0x1.99999ap-4`.

Nineteen reversible gate/arithmetic/role/call/ownership/closure/order/error mutations were killed, including an explicit ascending output-sort mutation over a non-monotonic interleaved fixture. The final audit SHA-256 is `acaea31285ee8e548af408ae65f3b08d9c08966d181140e67283bd7dfd4555d1`; production restored byte-exact at SHA-256 `f803b04ba8db10fc611954883f34eef7cf11674871e9138ec5d2e016d0b4855a`. Final gates pass focused 10/10, dependency 718/718, workspace 6,378/6,378 with two skipped, strict Clippy, wasm32, x86_64/aarch64 Windows, x86_64/aarch64 macOS, rustfmt, diff/LOC/static checks, clean pinned Orca, and no staged files.
