# Preserve the binary32 wipe-tower corner angle

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) constructs rounded Type1 wipe-tower corners in `src/libslic3r/GCode/WipeTower.cpp:205–245`. At line 235 the binary32 dot product goes through `std::acos(cosangle) / 2.f` before the tangent center and radius are converted to scaled integers. `WipeTower2.cpp:128–169` uses the same expression at line 159. Both Ares writers share `crates/ares-core/src/project_slice/gcode_emit/wipe_tower/geometry/rounding.rs::round_polygon`.

On a rounded rib tower, equal dot-product bits can produce adjacent half-angle values on native and WebAssembly targets. The center and radius differ by a scaled unit before the polar calculation, and the altered sampled vertices enter the arc fitter. Matching the corner angle at its source preserves the resulting arc without changing the fitter or output formatting.

## Rust destination and scope

Translate `cr_acosf` from CORE-MATH `src/binary32/acos/acosf.c` at `f786e13fb0595adee545d7b29931d283f658ba0a` into `crates/ares-core/src/scalar_math/acosf.rs`, retaining its copyright and full MIT notice. Use the shared module only at the `cosine.acos()` call in `round_polygon`; the target-provided call is the temporary implementation being replaced. Retain the exact coefficients and evaluation order. The output contract is round-to-nearest binary32 bits, including quiet NaN results and payloads matching the pinned glibc 2.42 `acosf@GLIBC_2.2.5` symbol. Other floating-point environments, exception flags and `errno` are out of scope. Leave the half-angle sine, polar angle, arc-sample sine/cosine, casts, fitter and G-code writer unchanged.

## Public verification

Slice a synthetic two-block project through Chromium and OrcaSlicer with an enabled rounded rib tower of width `24.123`, rib width `4.375`, and position `(110.0005, 81.013)`. Require complete exports and compare the exact ordered G0–G3 instructions within the wipe-tower spans in both Native reference modes. Retain the existing rounded-tower sampling comparison as a control and the random-seam comparison for the shared scalar module. For Bambu Lab models, `WipeTowerEstimate.cpp:62–73` selects Type1 even when the declared type is Type2; these inputs do not exercise the Type2 writer.

Compare the final translated routine against the pinned symbol under round-to-nearest on every binary32 bit pattern. No output golden or host-specific dependency is needed by the slicer or public test.
