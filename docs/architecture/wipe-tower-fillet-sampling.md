# Deterministic wipe tower fillet sampling

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/GCode/WipeTower.cpp::rounding_polygon:246–259`, constructs a
fillet arc, computes each sampling angle as `float`, wraps it, and uses
`std::cos(cur_angle)` and `std::sin(cur_angle)` to form an integer `Point`.
`WipeTower2.cpp::rounding_polygon:171–184` uses the corresponding sampling loop
for Type2. Both are represented by `append_arc` in
`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/geometry/rounding.rs`.

The current `f32::sin` and `f32::cos` calls follow the source types but delegate
the evaluation to the target math library. Replace those two calls only. Keep
angle iteration, wrapping, sign and precision, radius multiplication, rounding,
point insertion, and all `round_polygon` callers unchanged. In particular, the
support-wall and bottom/rib consumers keep their existing geometry and control
flow. This does not change the arc-fitting algorithm or I/J formatting.

## Observed numerical contract

The input is the synthetic two-cube, two-slot X1C fixture
`tests/parity/type1-compacted-exit/x1c-two-slots.3mf`, introduced in
`5760bf7dea5c2ecf41dcce281856790f79351abb`. To construct the reference project,
replace only `Metadata/project_settings.config` with the resolved configuration
from the pinned Orca preset-selection probe. All other archive entries are
byte-identical to the fixture.
A `printf` build of the pinned Orca source sliced that synthetic reference
project and emitted this G-code record:

```text
G2  X47.583 Y162.751   I4.179 J-1.242 E0.0937
```

The upstream instrumentation captured the input and both outputs of the calls
at `WipeTower.cpp:258`. All floating-point entries below are binary32 bits;
corner and sample indices start at zero.

| Corner/sample | Angle | Orca sine | Orca cosine | Orca scaled sample point |
|---|---|---|---|---|
| 2/19 | `4018a7ef` | `3f2faf21` | `bf3a33a7` | `(24708682,18581052)` |
| 6/8 | `408ce188` | `bf73cef7` | `be9c1ef6` | `(6210094,23227203)` |

Native Ares produces those bits. WASM produces sine `3f2faf20` in the first
row and cosine `be9c1ef5` in the second. In the first row, multiplying by the
scaled radius places Y on opposite sides of an integer rounding boundary.
Gap construction and six brim offsets carry the changed sample to the Type1
fitter. Both targets select the same fitting span and candidate index; the
changed point produces a different center and WASM emits `I4.178`.

The instrumented Orca build links glibc 2.42; the native Ares host links glibc
2.44. The Orca trace establishes agreement on the two sample calls, not that
arbitrary libm versions agree over the entire input interval.

## Reference implementation and provenance

The comparison reference for the port is the **FMA3 path of scalar `sinf` and
`cosf` in the host's x86-64 glibc 2.44**. The public symbols resolve through
`sysdeps/x86_64/fpu/multiarch/s_sinf.c` and `s_cosf.c`.
`ifunc-fma.h` selects `_fma` when FMA and AVX2 are both usable, otherwise `_sse2`.
`s_sinf-fma.c` and `s_cosf-fma.c` select the
`sysdeps/ieee754/flt-32/s_sinf.c` and `s_cosf.c` bodies under `-mfma -mavx2`;
see the same directory's `Makefile`. Fast reduction is in
`sysdeps/ieee754/flt-32/s_sincosf.h`; the x86 polynomial and coefficient layout
are in `sysdeps/x86/fpu/sincosf_poly.h` and `s_sincosf_data.c`. These names refer
to the [glibc 2.44 release source](https://ftp.gnu.org/gnu/libc/glibc-2.44.tar.xz).
On the host, dynamic lookup resolves both functions to bodies with fused
`vfmadd*sd` and `vfnmadd*sd` instructions. Disabling FMA and AVX2 changes both
resolved bodies to separate multiplications and additions. Both variants happen
to agree at the sample angles above; the variant selection was verified by
lookup and instructions, not inferred from those values.

Translate the algorithm from [Arm optimized-routines, revision
`20859d3fd9e797dde592c489c8ef7d383f778620`](https://github.com/ARM-software/optimized-routines/tree/20859d3fd9e797dde592c489c8ef7d383f778620/math):
`math/sinf.c`, `math/cosf.c`, `math/sincosf.h`, and `math/sincosf_data.c`.
These files declare `MIT OR Apache-2.0 WITH LLVM-exception`; use the MIT option
and retain Arm's attribution and license notice with the translated source.
Use glibc and the resolved machine code only to establish reference behavior
and fusion points, not as source to copy. Do not copy the LGPL x86 coefficient
layout or implementation.

## Rust destination

Use a private, pure-Rust binary32 sine/cosine implementation under the existing
rounding module, called on every target only from `rounding.rs::append_arc`.
There is no target dispatch or libm dependency in the production path. The
input domain is the wrapped binary32 fillet angle, bounded by positive zero
and the binary32 rounding of `2π` (`40c90fdb`); include negative zero for its
observable sine sign. General large-angle reduction, nonfinite values, errno,
and floating-point exceptions are not part of this boundary.

Keep the Arm algorithm's small-angle cases, quadrant and sign selection, fast
range reduction, and polynomial coefficients. Use the reference quadrant
conversion and bit thresholds, the same binary64 intermediate order, and
narrow to binary32 only where the reference does. Spell fused operations in
fast reduction and polynomial evaluation with `f64::mul_add`, preserving
separate operations where the reference does not fuse. `mul_add` supplies one
rounding even on targets that implement it in software. Transcribe the sine
and cosine paths as used by the two scalar functions, not a platform `sincos`
substitution with a potentially different evaluation order. Do not use wider
standard-library trigonometry as a shortcut: on the first sample, native
`sinl` narrowed to binary32 yields `3f2faf20`, not the Orca value.

## Acceptance criteria

1. **Exhaustive native scalar comparison.** Use a reproducible program outside
   the repository, not a committed test. Compile the exact candidate source in
   that program without adding any test hook, wrapper, or visibility change to
   the production module. Before changing the sampling call site, compare to
   the dynamically resolved host glibc 2.44 FMA3 `sinf` and `cosf` in an
   optimized native build. Enumerate every encoding from positive zero through
   `40c90fdb`, inclusive, plus negative zero: **1,086,918,621 inputs per
   function**. Compare `to_bits` exactly and do not constant-fold the reference
   calls. Record the source revision and SHA-256 of the compiled port and
   comparison program, build command, host glibc version and resolved variant,
   and compared and mismatch counts per function. Rerun for every submitted
   revision that touches the implementation. Required: **zero mismatches for
   sine and zero for cosine**. If the Arm translation cannot meet this
   requirement, stop and report the mismatches. Do not tune constants or
   evaluation order to fit the oracle or copy glibc's x86 `sincosf_poly` layout.
2. **Synthetic fixture replay.** Regenerate the resolved reference project from
   `tests/parity/type1-compacted-exit/x1c-two-slots.3mf` with the same Orca
   preset selection described above. Slice with the same options through the
   native and WASM public APIs. Require byte-identical emitted G-code,
   including the complete affected tower path and the Orca record above. Check
   both direct-project and PlaterSession WASM entry points. Do not mask,
   loosen, or refit any comparator. If an unrelated existing mismatch prevents
   byte identity, report it separately; this criterion has not passed. Do not
   commit Orca-generated G-code.
3. **Existing geometry consumers.** Exercise the existing Type1 and Type2
   rounded-wall and bottom/rib behavior through the public slicing API and
   preserve the paired Orca comparisons in both reference modes. Establish a
   synthetic public-API regression before altering production sampling. Its
   witness must cover a sample crossing an integer boundary and remain
   independent of the archived project.

## Separate numerical audit

The upstream `Circle.cpp::get_polar_radians:108–113`, called in
`ArcSegment::update_angle_and_length:267–277`, corresponds to the binary64
`atan2` in `rounding.rs::polar_angle`. It differs by one ULP on the first
corner, but all 20 wrapped binary32 sampling angles there are identical. Leave
this call alone. Audit it separately along with corner construction `acosf`
and half-angle `sinf`, and any other target-dependent transcendental calls;
none is changed here. Gap construction, polygon offsets, simplification,
and fitting remain downstream consumers of the sampled vertices.
