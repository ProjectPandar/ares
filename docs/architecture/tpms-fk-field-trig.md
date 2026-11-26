# Deterministic TPMS-FK implicit field trigonometry

## Upstream boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/Fill/FillTpmsFK.cpp:27–30` (`fischer_koch`) and `:58`
(`marchsq::ScalarField::get_scalar`), evaluates the Fischer–Koch S implicit
surface in binary32 arithmetic:

```cpp
cosf(2 * x) * sinf(y) * cosf(z)
    + cosf(2 * y) * sinf(z) * cosf(x)
    + cosf(2 * z) * sinf(x) * cosf(y)
```

Each coordinate is scaled as `double(freq) * coord` and narrowed to binary32
before calling `sinf` and `cosf`. Marching squares compares the binary32 field
value against `isoval = 0.0f` during cell tagging (`> 0.0f`) and edge
interpolation (`< 0.0f`).

## Rust destination and scope

In `crates/ares-core/src/fill/tpms_fk.rs` (`ScalarField::get_scalar`), replace
the target-dependent `f32::sin` and `f32::cos` calls with the shared scalar
binary32 routines in `crates/ares-core/src/scalar_math/sincosf.rs`. Expose a
`cos` entry point over `sincosf::sample` so `cosf(2 * coord)` avoids redundant
sine evaluation. Keep the frequency derivation, double-to-float coordinate
scaling, product and summation order, bounding-box padding (`FillTpmsFK.cpp:152`),
marching-squares traversal and interpolation, Douglas-Peucker simplification,
binary64 fill-angle rotation (`Point::rotate`), adaptive TPMS
(`AdaptiveInfillField.cpp`), and other periodic infill owners (`FillGyroid.cpp`,
`FillTpmsD.cpp`) unchanged.

## Failure mode and verification

A one-ULP difference in `sinf` or `cosf` can flip the sign of `fischer_koch`
across `0.0f`. On a `29.044762 × 22.563962 × 2.0 mm` block centered at
`(128, 128)` with `tpmsfk` infill at `20%` density, `0.2 mm` layers, `0.45 mm`
sparse infill line width, `45°` infill direction, zero walls/shells, and zero
overlap, layer `z = 1.8 mm` evaluates `fx = -2.3137403` (`0xc0141452`). The
pinned x86-64 reference yields `cosf(0xc0141452) = 0xbf2d2c6b` and
`fischer_koch = 0xb3800000` (`-5.96e-8`), while WebAssembly `libm` yields
`cosf(0xc0141452) = 0xbf2d2c6a` and `fischer_koch = 0x00000000` (`0.0`). That
sign flip shifts the contour vertex by one `0.004 mm` raster pixel and changes
the emitted X and E coordinates in Sparse infill.

A Chromium regression in `web/e2e/tpmsFkFieldTrig.spec.ts` slices that
synthetic block through the public WebAssembly API and compares its complete
output byte-for-byte with runtime OrcaSlicer in both official and seeded modes.
Ignore only the single slicer generator line; require positive extrusion in
Sparse infill and do not filter, quantize, or reorder the remaining G-code.
