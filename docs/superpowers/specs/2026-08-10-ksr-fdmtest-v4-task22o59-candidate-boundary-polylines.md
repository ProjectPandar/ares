# Task 22O.59 — candidate boundary polylines

## Status

Completed after independent source/specification review and final six-axis validation.

## Goal and source boundary

Port pinned `PrintObject.cpp:3226-3233`: apply the empty `area_to_be_bridge` continue, expand total-fill and limiting areas with their distinct source arithmetic, convert each temporary polygon vector to closed polylines, and append total-fill boundaries before limiting boundaries. The destination is private ordinary module `prepare_infill/bridge_over_infill/candidate_boundary_polylines.rs` and ordinary test children.

Direct closure is `Flow.hpp:64-69`, `Polygon.hpp:220-246`, `ClipperUtils.hpp:19,27,331-379`, `ClipperUtils.cpp:267-414`, and the accepted Ares closed offset kernel. O58 supplies `CandidateBridgeArea`; the future composer supplies exact O48-derived spacing values.

## Interface

```rust
pub(in crate::project_slice) fn prepare_candidate_boundary_polylines(
    candidate_area: &CandidateBridgeArea,
    total_fill_area: &[Polygon],
    scaled_spacing: Coord,
    spacing: f32,
) -> Result<Option<Vec<Polyline>>, ClipperError>;
```

## Behavior

1. Empty `candidate_area.area_to_be_bridge` returns `Ok(None)` before scalar validation/calculation, geometry inspection, or either offset call. Arbitrary scalar bits and invalid later geometry are ignored.
2. Compute `(1.3_f64 * scaled_spacing as f64) as f32`; expand all `total_fill_area` paths once with Miter/3.
3. Consume expanded paths in order. Move each path's point vector, append its original first point once, and emit one closed polyline per polygon.
4. Compute `(0.3_f64 * f64::from(spacing)) as f32`; expand all `candidate_area.limiting_area` paths once with Miter/3.
5. Consume and append limiting polyline values after every total-fill polyline. Preserve engine output and point order. Although source iterator insertion copies local limiting polylines, Rust may move them into the final vector; final allocation identity is non-observable and excluded from parity.
6. Return the first offset error and preserve the complete borrowed O58 result and total-fill input.

No raw option lookup, inferred Flow, alternate gate, validation, fallback, saturation, host sort, canonicalization, or lifecycle wiring is allowed. Trusted nonempty-branch inputs are the source-valid O58/O48 values described by the task ADR: positive spacing, finite positive products representable as positive f32, valid flat polygons, and nonempty offset-result polygons. The empty branch deliberately has no scalar or later-geometry precondition.

## Deferrals

Deferred: candidate loop and Flow provenance at 3213-3214; debug-only drawing; anchor/non-anchor O51 dispatch; O49 override; anchor append; lightning clipping; O53 construction; collision rerun; postprocessing; expansion mutation; candidate commit; surface rewrite; successor/lifecycle; extrusion, motion, G-code, and CLI parity.

## Acceptance

Begin with compiling RED. Build a removed actual-source driver against the audited fixed-MSVC-order archive and freeze exact ordered closed-polyline literals for:

- the empty gate proving zero scalar validation/calculation and zero engine calls with invalid later geometry, `scaled_spacing <= 0`, nonfinite spacing, and hostile finite scalar values;
- exact total delta bits from f64 `1.3 * coord_t`, including a discriminator against f32 multiplication and against cast-before-multiply;
- exact limiting delta bits from f64 `0.3 * promoted f32 spacing`, including a discriminator against f32 multiplication and against scaling the mm spacing;
- default Miter/3 expansion roles and first-error order;
- temporary-rvalue polygon-to-polyline consumption: one closure duplicate, polygon order, point order, holes/components, and total-before-limiting append order; final limiting allocation identity is explicitly not asserted;
- empty total or limiting geometry after a nonempty candidate;
- first total-offset and second limiting-offset natural range errors;
- complete input/allocation nonmutation and repeatability.

Use a private injected operation-order seam to freeze offset call count, roles, exact delta bits, consumed temporary results, conversion timing, append order, and competing-error precedence when commutative geometry would hide mutations. Kill reversible mutations for empty-gate omission/reversal, pre-gate validation/calculation, wrong emptiness field, f32 or reordered arithmetic, role swaps, offset omission/repetition/reversal, cloning offset polygons during conversion, missing/double closure points, limiting-before-total append, output sorting, and error-order reversal; restore source byte-exact.

Private ordinary registration is required; every file contains at most 399 lines. `include!`, `include_bytes!`, and `include_str!` are prohibited for source splitting. Final gates: focused O59, O43-O59/Clipper/Flow dependency, Linux workspace, rustfmt, strict Clippy, wasm32, x86_64/aarch64 Windows, x86_64/aarch64 macOS, diff/LOC/static/clean-Orca/no-staged checks, and independent six-axis repair/re-review until unconditional approval.

## Completion evidence

The repeatable removed source oracle freezes exact empty, ordered, acute-Miter, and arithmetic literals. Archive/driver/object/binary/output/link-command SHA-256 values are `b643964e681e9435680b78fdd743dcb12c9c07cd16ef164e353d370add8132a1`, `577738d4c8f00879276ac7815afe6d9b0ca80e834fe67899df46cfe30ccbd532`, `a3b74f48e04fb41b61f6694b95b510571bdc78c978ed9d1bfbe79358690b7106`, `9762c5531a462913e3f0476ce42fb2a0ef4d2f8d7e94353b21ed60aa6abb1296`, `9fa3600b73ebe60ac821a2f9811b606aae85ad6f6a1ee797cc4c5ffd62bda320`, and `98f57cf5675fca7419790c1a5b16b5d4aa97ae208cbd6b76692bdc42c262333b`.

All nineteen reversible mutations were killed, including ascending output sorting over a non-monotonic interleaved fixture; audit SHA-256 is `acaea31285ee8e548af408ae65f3b08d9c08966d181140e67283bd7dfd4555d1`, and source restored at `f803b04ba8db10fc611954883f34eef7cf11674871e9138ec5d2e016d0b4855a`. Fresh gates pass focused 10/10, dependency 718/718, workspace 6,378/6,378 with two skipped, strict Clippy, wasm32, four Windows/macOS cross checks, rustfmt, diff/LOC/static, clean Orca, and no staged files.
