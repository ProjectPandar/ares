# Transcendental determinism in slicing and G-code

## Contract and scope

Use the same evaluation on native and WebAssembly wherever a math result can
change a slice or emitted bytes. The upstream behavioral reference is OrcaSlicer
2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`); the numerical comparison
below uses scalar x86-64 glibc 2.44 as the native math reference. These are
separate claims: a microprobe using that libm is not a slice against OrcaSlicer,
and other native releases and CPU dispatch variants need not match it.

The [call-site inventory](transcendental-determinism-sites.md) covers the direct
Rust calls in `ares-core` and the adapter's `ares-vgcode` consumers at Ares
`3722423bea65ea040b2f50f6ac5ab395bf8f30e4`. It follows `ares-wasm`'s public
slice and processed-slice calls into core. The adapter has no direct calls
in the surveyed math families. Placement-only, validation-only and view-only
sites are included but distinguished from emitted toolpaths. Inline Rust
matches and source-owned C++ counterparts are listed separately so a port
does not accidentally standardize a Rust placeholder instead of upstream.

## Priority by downstream effect

| Order | Port boundary | Input domain and point of sensitivity | Observed result |
|---|---|---|---|
| Existing focused slice | `WipeTower.cpp:246–259` and `WipeTower2.cpp:171–184`, `rounding_polygon` fillet sampling → `geometry/rounding.rs::append_arc` | Wrapped float angle in `[0, 2π]`. A one-ULP sine/cosine difference can cross the integer sample boundary and propagate to the Type1 fitter. | The companion `wipe-tower-fillet-sampling.md` specification records an actual changed I word and verifies its proposed sine/cosine kernels over the bounded input interval. That replacement is limited to the two sampling calls. |
| 1 | `PrintObject.cpp:209–232`, `create_polyholes` → `hole_to_polyhole/polyhole.rs::create_polyholes` | Integer radius/center, discrete edge count, at most five twist variants; float sample angles in `[0, 2π)`. Addition/multiplication is float before rounding to integer coordinates. Verify the pinned edge-count and radius expressions first. | Changed integer vertices in both the current-width and upstream-width denominator probes. This is not yet a changed public slice. |
| 2 | `GCode/SeamPlacer.cpp:1367–1410`, `pick_random_seam_point` → `seam_placement/alignment/random.rs::select` | Float dot of positions in mm. Unlike wrapped angle sampling, phase can span many periods. Multiplication by ~43,759 followed by integer-part removal magnifies a sine difference; event selection can change both XY and the beginning of an extrusion loop. | Changed float fractions, including a wrap across an integer. Verify upstream seed/candidate choice before attributing any end-to-end mismatch to the sine. |
| 3 | `WipeTower.cpp:205–245`, `WipeTower2.cpp:128–169` corner construction, and `Circle.cpp:110` `polar` helper → `geometry/rounding.rs::round_polygon` / `polar_angle` | Accepted normalized dot in approximately `(-cos(π/6), cos(π/6))`; half-angle about `[π/12, 5π/12]`; `atan2` receives scaled integer deltas across all quadrants. Center/radius casts and final float sampling-angle narrowing are the decisive boundaries. | Corner distances and polar angles differ. The first observed fillet corner's one-ULP polar difference did **not** change its sampling angle. No changed center/radius or G-code attributed to the remaining calls. |
| 4 | `GCodeWriter.cpp:952,1025–1050,1236–1269`, lift and spiral write → `motion/travel/lift.rs` / `spiral.rs` | Positive rise, run and configured slope; slope threshold selects branch, radius determines bed fit and emitted IJ/XYZ. Ordinary format rounding will absorb most one-ULP differences; target decimal half-boundaries and fit thresholds. | Changed radius in the isolated computation. No decimal difference established. |
| 5 | `Support/TreeSupportCommon.hpp:285–305`, derived organic settings → `tree/organic/settings.rs::TreeSettings::new` | Float settings widened into double radians and multiplied by scaled layer height. The result is truncated for move caps, or reused for later influence radii. Cover normal, slow and diameter angles separately; the upstream near-π/2 sentinel is not a tangent evaluation. | Tangent differs; sampled instantaneous integer move caps agree. This does not establish agreement for all layers, radii or configured angles. |

Keep the polyhole radius width correction distinct from sample-kernel
replacement, and keep seed selection distinct from random-seam sine
replacement. In corner construction, compare `acosf` and the downstream
half-angle sine as a pair before testing the center/radius integer casts.
Treat polar `atan2` as its own `Circle.cpp` slice only if a difference reaches
the rounded sampling angle. For travel lift, test the three branch/radius
calls with the writer rather than changing other consumers of the same math
functions. Keep organic tree movement separate from branch mesh tessellation.

Next investigate geometry that multiplies trig by scaled lengths or uses
angles as a topology test: Ripple (`Feature/FuzzySkin/FuzzySkin.cpp:218,286`),
Clipper2 round offsets (`clipper.offset.cpp:277–294,370`), bridge normal bins
and polygon rotation (`BridgeDetector.cpp` and `Line.hpp:179`), and periodic
infill (`FillGyroid.cpp`, `FillTpmsD.cpp`, `FillTpmsFK.cpp`). For lengths on the
writer path, confirm `hypot` versus explicit norm first. For example,
`GCode.cpp:10227` computes an Eigen double `norm()` where the Rust variable
path computes `hypot`; exchanging both target libms for a common `hypot`
would not restore that source expression. Likewise, the significant-digit
formatter's `log10` is a Rust choice; a follow-up needs the upstream decimal
format contract, not a new generalized logarithm API.

## Numerical comparison

The probe builds one Rust source optimized (`-C opt-level=3`) for
`x86_64-unknown-linux-gnu` and as a `wasm32-unknown-unknown` `cdylib`, with
Rust 1.99.0 (`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, LLVM 23.1.1).
Native explicitly resolves scalar `libm.so.6` symbols at runtime using
`dlopen`/`dlsym` rather than compiling C/Rust constants. The other build uses
actual WASM `std` float methods. Node 24.21.0 instantiates the WASM with an
asserted empty import list, so no JS `Math` substitutes for the target
implementation. The native process writes little-endian (operation, three
input bit words, result bit word) records; Node replays them and counts
bitwise differences, with signed-zero bits retained. The samples below are
synthetic, not values collected from production slicing.

| Probe | Samples | Different | Maximum ULP (raw numeric rows only) |
|---|---:|---:|---:|
| Corner `acosf`: evenly spaced and binary32 bit strata in accepted dot range | 262,144 | 116,746 | 1 |
| Corner half-angle `sinf`: uniform on approximately `[π/12,5π/12]` | 262,144 | 3,528 | 1 |
| Corner `distance / sinf(acosf(dot)/2)` with finite synthetic distances | 262,144 | 10,579 | 2 |
| Polar `atan2`: scaled integer differences in `[-10⁷,10⁷]` and near-axis pairs | 262,144 | 44,618 | 1 |
| Polyhole `sinf`: all vertices of all five twists for edge counts 3–256 | 164,465 | 2,014 | 1 |
| Polyhole `cosf`, same inputs | 164,465 | 2,072 | 1 |
| Polyhole integer `(X,Y)`, current Rust float denominator | 164,465 | 771 | — |
| Polyhole integer `(X,Y)`, upstream double denominator narrowed to float | 164,465 | 771 | — |
| Random seam `sinf`: dot products on a synthetic 350 × 350 × 400 mm grid sampled at 0.001 mm input granularity | 262,144 | 3,397 | 1 |
| Random seam fraction after multiplication and removal of integer part | 262,144 | 2,980 | — |
| Organic `tan`: evenly spaced branches from 0° through 89.9° | 262,144 | 9,660 | 1 |
| Organic integer move cap with synthetic scaled layer heights | 262,144 | 0 | — |
| Spiral `atan`: evenly spaced positive slopes from 0.001 through π/2 radians | 262,144 | 11,027 | 1 |
| Spiral radius with those slopes and 0.001–2 mm of rise | 262,144 | 8,759 | 3 |

At `PrintObject.cpp:226`, `PI` is double (`libslic3r.h:81`), so the radius
uses `float(std::cos(PI / nb_edges))`, not `cosf(PI_f32 / nb_edges)`. The
integer witness uses the upstream-width expression and is separate from that
existing expression difference. At `:229` the sample calls receive float
angles. The selected edge-count range is exhaustive only within the probe;
it is not a claim about reachable user settings. For the random seam, a
phase of `18999.603515625` yields native/WASM sine `bf2e48dd`/`bf2e48dc` and
fractions `3f4a0000`/`3f490000` (binary32 bits). The probe uses the
binary32-rounded hash multiplier. For the corner, the uneven binary32 bit
stratum includes very small magnitudes; the aggregate is not an estimate
of how often production fillets differ. For support, sample spacing alone
is unlikely to find a near-integer tie; choose cases by inverting the
movement threshold as well as using real presets before declaring parity.

Source SHA-256: Rust probe
`e2ce647a5c4cc021f977ede31f1bbe4df7c8141fb5839dff2b7b5c1ff8f2d57c`;
replay/comparator
`bf274940b0230af8a63b9a180bf9af8698716a890e14c457e3028aa857402ecd`.
WASM module SHA-256:
`e238f6b763b7e30db5648aab9260ca97b796778ae97c51f7ca2e6256cfe78472`.

### Library provenance

Rust's [pinned compiler-builtins tree](https://github.com/rust-lang/rust/tree/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/compiler-builtins)
contains `libm` **0.2.16**. The software functions descend from musl and
FreeBSD/fdlibm kernels, with changes in Rust; consult the pinned sources, not
an arbitrary crates.io `libm` build with host acceleration enabled. The
native reference is the dynamic binding of [glibc 2.44](https://ftp.gnu.org/gnu/libc/glibc-2.44.tar.xz)
on x86-64; release tarball SHA-256
`37f600f2bef3c5e8300147059568b2a2e40a7ad6ccc65ce942556d49429cc667`.

| Family | Rust `library/compiler-builtins/libm/src/math/` | glibc 2.44 scalar source | Why agreement is not guaranteed |
|---|---|---|---|
| `sinf`, `cosf` | `sinf.rs`, `cosf.rs`, `k_sinf.rs`, `k_cosf.rs`, `rem_pio2f.rs` | `sysdeps/ieee754/flt-32/s_sinf.c`, `s_cosf.c`, `s_sincosf.h`, x86 `fpu/sincosf_poly.h`, x86-64 multiarch `s_sinf.c`/`s_cosf.c` | Different range reduction and polynomials; x86 can select FMA3 for the scalar routines. |
| `acosf` | `acosf.rs`: fdlibm-derived float rational approximation with split constants and square-root cases | `sysdeps/ieee754/flt-32/e_acosf.c`: CORE-MATH implementation | Different approximation and rounding; near zero a split-constant result can disagree with the rounded π/2 value. |
| `atan2`, `atan` | `atan2.rs`, `atan.rs`: quadrant cases, ratio, piecewise reduced argument | `sysdeps/ieee754/dbl-64/e_atan2.c`, `s_atan.c`; x86-64 `fpu/multiarch/` variants | Different treatment of ratios/reduced arguments and polynomial/table evaluation (IBM Accurate Mathematical Library on the glibc side). |
| `tan` | `tan.rs`, `k_tan.rs`, `rem_pio2.rs` | `sysdeps/ieee754/dbl-64/s_tan.c`, x86-64 `fpu/multiarch/s_tan.c` | Different range reduction/table and polynomial evaluation (IBM on the glibc side); a binary64 difference may or may not survive a scaled integer cast. |
| Other trig, including `sin_cos` | `sin.rs`, `cos.rs`, `asin.rs`, `acos.rs` and width-specific partners, plus reduction kernels | `sysdeps/ieee754/{flt-32,dbl-64}` `s_sin.c`, `s_*f.c`, `e_asin.c`, `e_acos.c` or width-specific kernels and x86 variants | Check both legs of a `sin_cos`; a pair of correct-looking individual call results does not establish source width or compiler lowering. Not measured here. |
| `exp`, `ln`, `log10`, `powf` | `exp*`, `log*`, `pow*` kernels at the pinned version | matching `e_exp*`, `e_log*`, `e_pow*` and x86-64 multiarch variants | Approximation and special cases are library-specific; integer-valued `powf` exponents do not make these IEEE primitives. Not measured here. |
| `hypot` | `hypot.rs`, `hypotf.rs` | `sysdeps/ieee754/{flt-32,dbl-64}/e_hypot*.c`, x86 dispatch where applicable | Scaling, compensation and narrowing differ from each other and from a direct `sqrt(x*x+y*y)` expression. Not measured here. |

### Arithmetic that does not need a transcendental replacement

For finite operands and matching widths, `+`, `-`, `*`, `/`, `sqrt`, explicit
`mul_add`, integer conversions and `round`/`floor`/`ceil`/`trunc` are not
unconstrained transcendental approximations. They can still expose a prior
ULP difference. Do not change intermediate precision, add FMA where upstream
uses separate operations, or substitute a different norm/reduction order.
NaN payloads and upstream undefined cases are not part of the contract.
`powi` is separate: it does not call `powf`, but Rust does not promise one
multiplication order for general exponents. Audit fixed squares as multiply
forms, and retain explicit width/rounding checks for format scaling, signed
exponents, and subnormal paths. See the inventory for the `powi` sites.

## Exit criteria for a follow-up slice

Use one upstream owner per slice. First pin the exact operand widths and
source expression, then show the error crossing the boundary through a
public slicing/processed-slicing fixture. Compare native and WASM with the
same options and fixture, and compare with pinned OrcaSlicer. For bounded
discrete domains, exhaust reachable values; otherwise state the stratification
and record misses as well as differences. Preserve exact byte comparisons,
including any changed geometry that later rounds into G-code. Do not import
an entire library, change unrelated algorithms, or add a per-target fallback.
