# Explicit outer brim width quantization

## Source and starting point

Port the ordinary `OuterOnly` contour width in OrcaSlicer 2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `src/libslic3r/Brim.cpp`,
`outer_inner_brim_area`. Start from `fccf447888957f115d91201287908e6d6c6e49bc`:
the X1C stock direct-STL comparison also needs the loaded-filament-slot fix
in `a49588003` to serialize `slow_down_min_speed` as `20`, not `20,20`.
That independent configuration difference is not normalized away.

## Arithmetic and destination

- `Print.cpp:3068–3086` supplies `Print::brim_flow()`. For the rounded
  rectangle, `Flow.cpp:207–212` computes the spacing from float width and
  height. `Flow.hpp:71` divides this float by the double scaling factor,
  then truncates to `coord_t` once. `Brim.cpp:496,498` derives **two** values
  from that lattice integer: double `flowWidth` in mm (multiply by the
  scaling factor), and float `scaled_flow_width` (cast the integer, do not
  re-scale the mm value).
- `Brim.cpp:497` divides the configured double by `flowWidth` and then by
  two, floors, multiplies by `flowWidth`, then multiplies by two, all in
  double. `scale_` (`libslic3r.h:80,106`) divides by the double scaling
  factor; the next conversion is directly to float `brim_width`, not to
  `coord_t`.
- For explicit outer-only, neither auto conditional applies, so
  `brim_width_mod` starts as this same float (`Brim.cpp:561–575`). At
  line 576 both divisions are float. The unqualified `floor` in this
  translation unit resolves to the C double overload on the pinned Linux
  producer: promote the float quotient, floor in double, multiply by the
  promoted float spacing, then by two. Assign back to float before calling
  `offset_ex` (`Brim.cpp:581–594`, `ClipperUtils.hpp:369–370`). Preserve
  the operand order without a tolerance, integer round-trip or reassociation.
- `project_slice/gcode_emit/brim.rs::BrimPlan::prepare_area` owns the inputs;
  `brims/outer.rs` consumes the resulting contour offset. The existing area,
  masks, contact handling and infill are retained; the old width conversion
  is only a temporary compatibility shell for other callers.

This is deterministic IEEE-754 rounding, not undefined behavior: the spacing
is positive and the spacing cast and offsets are well within range. At a
0.50 mm first-layer line width and 0.20 mm layer height, spacing truncates to
457079 lattice units. For a 3.6 mm request, the first quantization and scale
give double `2742473.9999999995`, which becomes float `2742474`. A spurious
integer truncation yields `2742473` instead. After the float divisions the
correct quotient is `3.0`, not `2.9999988079071045`: the latter loses two
extrusion spacings. At 0.51 mm, spacing is 467079 units and both paths give
a quotient of `3.0`.

## Adjacent paths

- `InnerOnly` does not consume the re-quantized width: the hole inset is
  `-brim_width - brim_offset` (`Brim.cpp:505,596–607`).
- `OuterAndInner` uses the same line 592 offset for its outer ring and the
  original width for its hole inset. Its combined two-ring port and an
  activated hole witness are deferred; no change to the hole input is implied.
- Auto reaches the same outer ring after a different input pipeline: the group
  calculation assigns `brim_width` at lines 549–551, the adhesion adjustment
  is at lines 563–569, and `std::min(float, float)` with the maximum lattice
  bounding-box side is at lines 572–575, before line 576. Ares still uses a
  separate mm-space approximation there. The auto group/clamp port and an
  activated clamp witness are deferred.
- Gap conversion, ear modes, support/raft/tree, and path order are unchanged.

## Public behavior

Slice a generated 14 × 12 × 3.2 mm flat plate via the public loaded-settings
STL API. Use stock 0.4 mm nozzle X1C, K1 and MK4 presets, explicit `outer_only`
and 3.6 mm brim, with 0.50 mm first-layer line width (including MK4) and
0.51 mm as a nearby control on each. Require an active brim and ordered
whole-file parity to both official and seed-0 Native, allowing only the
existing generator identity/time normalization. Compare the affected brim
inventory at the starting point and the resulting revision in both modes.
