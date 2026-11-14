# Post-retraction travel planning

## Source boundary

This is a Rust rewrite of OrcaSlicer 2.5.0 commit
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/GCode.cpp::GCode::travel_to`:

- Lines 10569–10580 condition the initial perimeter planning on the current
  writer position being clear and the one-shot disable being absent.
- Line 10591 consumes the initial one-shot modifiers.
- Lines 10595–10617 compare `last_pos()` before and after retraction, replace
  the travel with the new-start direct path when it moved, and invoke
  `AvoidCrossingPerimeters::travel_to` again when perimeter avoidance and a
  layer are available. This second call does not repeat the writer-position
  or one-shot-disable conditions from the initial call. The saved external
  one-shot modifier is restored for this call and consumed afterwards.

The Rust destination is
`ares-core::project_slice::gcode_emit::motion::path::start_travel`:
`route.rs`, `approach.rs`, and `retract.rs`. The existing shared `plan_route`
call is a temporary compatibility shell: it incorrectly applies the initial
writer-position condition to the post-retraction call as well.

## Decisions

1. Keep initial planning and post-retraction planning as distinct entry points.
   Share only perimeter-boundary construction, router invocation, and endpoint
   collection. Do not set `EmitState::positioned` to bypass initial semantics.
2. Compare the scaled generator cursor across retraction, corresponding to
   upstream `last_pos()`, rather than deciding from rounded emitted coordinates.
   Rebuild the travel from the updated cursor when it changed. The existing
   `LayerGeometry` supplies the layer context for this Rust call boundary.
3. Restore the saved external one-shot modifier only around the second
   perimeter-planning call, then consume both one-shot modifiers as upstream
   does. Leave no restored modifier behind when Rust returns a routing error.
4. Preserve the original destination, scaled coordinate conversion, detour cap,
   boundary caches, external-coordinate handling, and all waypoint emission.
   Do not change external bounding boxes, padding, the safe-zone predicate,
   routing algorithms, lift scheduling, retraction amounts, or M73 processing.

## Verification

Use the existing synthetic two-object X1 Carbon project in
`tests/parity/avoid-crossing-external-frame/x1c-detour0.3mf`, unchanged SHA256
`f44f09c6b26391787d5877cfafd094bb50cb452d727298d46cd6ab5d45ffa04e`.
The historical test identity remains intact, but its source citations describe
this caller boundary rather than attributing the mismatch to external routing.

The public seam is project bytes to complete Native, Rust API, and Rust CLI
G-code streams. Run the existing exact test before the implementation and
again afterwards, against both official and seeded Native modes. Require exact
whole-stream comparison with the existing generator/vendor normalization;
retain every M73 and empty line. Native replay instability, setup errors, or
missing producers are not successful comparisons. Preserve inputs, commands,
outputs, comparison artifacts, and actual process results outside the source
tree. Exercise the existing prime-tower, MK4, printer-order, and seam-travel
cases as neighbouring whole-stream checks. Formatting and strict workspace
all-target Clippy must also pass.

## Deferred source behavior

`GCode.cpp:10618–10620` combines exclusion-volume rerouting with its earlier
result, preserving that earlier result only when `last_pos()` did not change.
The Rust motion emitter does not yet implement
`ExclusionVolumeTravelAvoidance` or `route_around_exclusion_volumes`.
The exclusion-volume router and this combination are a separate dependency;
this slice neither introduces a no-op substitute nor claims that behavior.
Its eventual source boundary includes `GCode.cpp:10440–10520`, the planned-Z
and coordinate transforms, and the owning exclusion-volume router.

External/support safe-zone parity and further perimeter-router algorithms are
also deferred. This slice does not design an independent slicing pipeline or
change libvgcode, browser adapters, filesystem behavior, or terminal behavior.
