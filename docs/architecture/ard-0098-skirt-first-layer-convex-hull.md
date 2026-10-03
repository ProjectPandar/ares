# ARD-0098: Preserve the upstream skirt occupied hull for first-layer bounds

The owning upstream boundary is `Print.cpp::_make_skirt` and its
`append_skirt_loops_for_hull` lambda (lines2720–2775), which separately
offsets the occupied hull at the final skirt distance plus half spacing.
`Print::finalize_first_layer_convex_hull` (lines3154–3163) includes that hull;
`GCode.cpp` (lines2926–2939) exposes its bounds as first-layer template values.
`Print::has_skirt` (lines583–586) uses height alone: zero loops still produce
the occupied hull, while the extrusion collection stays empty.

Rust destination: `ares-core::project_slice::gcode_emit::skirt::SkirtPlan`
and its first-layer footprint consumer. Ares's simplified emitted centerline
loops are not an equivalent source for this hull: offsetting the original
occupied hull and padding a simplified centerline bounding box need not agree.
Keep the upstream geometric source and numeric operation order, without
changing emitted paths or independently designing a footprint algorithm.
The existing traversal/footprint assembly is a temporary transport shell
around the upstream print geometry and placeholder concepts.

Included: combined and currently supported single-object skirt occupied
bounds, final spacing offset including minimum skirt length, and unchanged
stock Kobra 3 Max zero-loop complete output. Replace the bounding-box distance
fallback with the same occupied geometry, without emitting zero-loop paths.
Include zero-loop per-object collision groups from `_make_skirt`
(lines2785–2905) so the new occupied path does not newly reject multi-object
inputs; retain object hull identity until the collision merge reaches its
fixed point. First-layer islands remain first-layer-only; skirt's own occupied
collector owns finite/infinite skirt height. Deferred, not exempted: positive-
loop multi-object per-object extrusion groups, adjacent brim/support/tower
footprint gaps and all
other printer/option matrix residuals. No comparator or fixture relaxation.
