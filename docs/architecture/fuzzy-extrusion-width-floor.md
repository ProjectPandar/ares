# Fuzzy extrusion-width floor

## Source boundary

This is a Rust rewrite slice of OrcaSlicer `libslic3r` at
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

`src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp::fuzzy_extrusion_line`
accepts the actual perimeter-generator `layer_height` at line 358.
Lines 372–374 define a fractional scaled-double minimum width:

```cpp
scaled<double>(layer_height * (1. - 0.25 * M_PI) * 1.05)
```

The Extrusion and Combined branches add that floor to the destination
junction width and noise radius before taking the maximum with the floor
(lines 400–405). Combined derives displacement from the resulting double;
conversion to the integer junction width happens afterward.

The whole-coverage and clipped-segment callers forward the same actual
height at lines 732, 743, 805 and 818. This is not the sampling Z coordinate
and is not a nominal profile height or a reconstructed flow width.

## Rust destination

All paths below are relative to `crates/ares-core/src/project_slice/`:

| Destination | Responsibility |
|---|---|
| `perimeters/arachne/materialize.rs` | Forward `PerimeterInputRecord::layer_height` into traversal. |
| `perimeters/arachne/traverse.rs` | Carry actual height into fuzzy `Layer`. |
| `perimeters/arachne/fuzzy.rs::Layer` | Preserve the operand through whole and clipped calls. |
| `perimeters/arachne/fuzzy/junctions.rs::fuzzify` | Compute and consume the source-defined floor. |

The existing `.01` scaled-width expression is a temporary incompatible
shell for the cited upstream minimum and is replaced, not retained as a
fallback. Existing `CoordinateScale::scaled_delta` supplies division by
the active coordinate scale without integer quantization.

## Invariants and included behavior

- Preserve the source multiplication order in `f64`, then scale as a double.
- Use the same double floor for both addition and the minimum comparison.
- Keep destination-width addition, noise radius and maximum evaluation order.
- Keep Combined displacement fractional until the existing coordinate casts.
- Keep the existing final truncating cast to integer junction width.
- Carry the actual height unchanged, including non-default layer heights.
- Preserve RNG calls, draw order, point sampling, junction order and padding.
- Displacement widths and Ripple's independent early-return path are unchanged.
- Core logic remains platform-neutral for browser WASM and native platforms.

## Deferred adjacent behavior

Noise kernels, RNG implementation, Ripple, closure, clipping, effect merging,
supported-region masks, geometry algorithms, downstream flow conversion,
seam placement and travel routing are outside this source slice.

## Failure modes

An unscaled or prematurely quantized floor changes emitted widths. Using
slice Z or nominal height changes the source operand. Applying the floor
to Displacement or Ripple changes independent modes. Additional path,
extrusion, metadata or configuration differences after widths agree are
separate causes; this boundary does not authorize compensating arithmetic
or a replacement pipeline.
