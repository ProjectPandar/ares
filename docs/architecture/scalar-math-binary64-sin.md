# Deterministic binary64 sine for ripple fuzzy skin

## Source boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) evaluates ripple
fuzzy-skin displacement in binary64 (`double`) precision at two call sites in
`src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp`:

- Classic walls (`fuzzy_polyline_ripple`, `FuzzySkin.cpp:172-231`): line 218
  computes `const double disp = std::sin(arc_phase(s)) * amplitude;` and line
  220 projects `(seg_perp * scale_(disp)).cast<coord_t>()`.
- Arachne walls (`fuzzy_extrusion_line_ripple`, `FuzzySkin.cpp:236-304`): line
  286 computes `const double disp = std::sin(arc_phase(s)) * amplitude;` and
  line 288 projects `(seg_perp * scale_(disp)).cast<coord_t>()`.

In both pinned x86-64 OrcaSlicer 2.5.0 binaries (glibc 2.42 and glibc 2.44),
`std::sin(double)` resolves to IBM's `s_sin.c` implementation (`__sin_fma`,
`__sin_avx`, and `__sin_sse2`), which produces correctly rounded IEEE-754
round-to-nearest-even binary64 results (`<= 0.5` ULP) across the witness
inputs. By contrast, `wasm32-unknown-unknown` lowers `f64::sin` to
`compiler_builtins::libm` (`sin.rs`, fdlibm/musl-derived), whose error reaches
`~0.56` ULP and differs by 1 ULP from correctly rounded `sin` on roughly `2.95%`
of sampled ripple phases.

To eliminate target-libm divergence while keeping zero `unsafe`, zero FFI, and
zero JavaScript `Math` imports, Ares ports CORE-MATH's MIT-licensed correctly
rounded binary64 sine routine (`cr_sin`) from upstream CORE-MATH revision
`3270006242ceef2885767918533ac5446807f2cf`, `src/binary64/sin/sin.c`
(Copyright (c) 2026 Paul Zimmermann and Alexei Sibidanov).

## Rust destination boundary

- `crates/ares-core/src/scalar_math.rs`: minimal module declaration and
  crate-internal export (`mod sin_f64; pub(crate) use sin_f64::sin_f64;`).
- `crates/ares-core/src/scalar_math/sin_f64.rs`: entry dispatch (`sin_f64`),
  moderate-range Cody-Waite fast path (`2^-26 <= |x| < 2^31`), large-range
  Payne-Hanek fast path (`|x| >= 2^31`), and double-double helpers.
- `crates/ares-core/src/scalar_math/sin_f64/tables_fast.rs`: Payne-Hanek
  `1/(2*pi)` table (`T`), quarter-wave `U1` sine table (`U1_SIN`, 65 pairs,
  reconstructing all 128 `U1` entries bit-identically via symmetry), and `U2`
  table (`128 x 4`).
- `crates/ares-core/src/scalar_math/sin_f64/tables_accurate.rs`: 128-bit
  fixed-point accurate-path polynomials (`PS`, `PC`) and trig tables
  (`S1U`, `S2U`, `C2U`).
- `crates/ares-core/src/scalar_math/sin_f64/accurate.rs`: cold accurate paths
  (`sin_small_accurate` for `|x| < 2^-16`, `sin_large_accurate` for
  `|x| >= 2^-16`), 128-bit Payne-Hanek reduction (`reduce_large_acc`), and
  rounding (`u128_to_f64`).
- `crates/ares-core/src/perimeters/fuzzy_skin_noise.rs:62`
  (`ripple_scaled_closed_polyline`): replace the temporary target-libm
  `phase.sin()` shell with `crate::scalar_math::sin_f64(phase)`.
- `crates/ares-core/src/project_slice/perimeters/arachne/fuzzy/ripple.rs:52`
  (`fuzzify`): replace the temporary target-libm `phase.sin()` shell with
  `crate::scalar_math::sin_f64(phase)`.

## Scope and deferred behavior

Included in this slice:

- Pure-Rust, zero-`unsafe`, correctly rounded binary64 `sin_f64(x: f64) -> f64`
  covering normal values, subnormals, tiny values (`|x| < 2^-26`), moderate
  arguments (`2^-26 <= |x| < 2^31`), large arguments (`|x| >= 2^31` up to
  `f64::MAX`), signed zeros (`+0.0` -> `+0.0`, `-0.0` -> `-0.0`), infinities
  and NaN (canonical quiet NaN `0x7ff8_0000_0000_0000`).
- Wiring `sin_f64` at both upstream binary64 ripple fuzzy-skin call sites
  (`FuzzySkin.cpp:218` and `FuzzySkin.cpp:286`).

Unchanged and deferred:

- `cos_f64` and `sincos_f64`: CORE-MATH's binary64 `cos.c` and `sincos.c`
  share the exact same `T`, `U1_SIN`, `U2`, `PS`, `PC`, `S1U`, `S2U`, and
  `C2U` tables; exporting `cos_f64` / `sincos_f64` is deferred until a proved
  upstream binary64 cosine divergence requires it so no dead code or unneeded
  WASM bytes are introduced.
- Other binary64 trigonometric calls outside `FuzzySkin.cpp`, non-ripple fuzzy
  skin noise generators (`classic`, `perlin`, `billow`, `ridgedmulti`,
  `voronoi`), and ripple phase/anchor arithmetic.

## Failure modes

When `sin(phase) * thickness / 1e-6` lands within `~10^-10` of an integer
nanometer boundary, a 1-ULP error in `f64::sin` changes the truncated `i64`
displacement by `1` scaled unit (`10^-6 mm`). If the displaced world coordinate
lies on a `0.0005 mm` three-decimal G-code rounding midpoint, `wasm32` emits a
different `X`/`Y` coordinate and downstream `E` value than native Ares and
OrcaSlicer 2.5.0:

1. **36 mm cube witness (`tests/parity/fuzzy_skin_supported_area/classic_ripple_sin_witness.3mf`)**:
   `36.0 mm` cube translated to `(110.0, 110.000776, 0.3) mm`, 3 layers at
   `0.2 mm`, Classic single/double wall (`outer_wall_line_width = 0.5`,
   `initial_layer_line_width = 0.5`), `fuzzy_skin = external`,
   `fuzzy_skin_first_layer = 1`, `fuzzy_skin_noise_type = ripple`,
   `fuzzy_skin_ripples_per_layer = 34`, `fuzzy_skin_thickness = 1.736865`,
   `fuzzy_skin_point_distance = 0.5`, `fuzzy_skin_ripple_offset = 0%`.
   - At sample `s = 27.5` on the top edge (`phase = 0xc0403a55ff8950f3`),
     correctly rounded `sin` is `0xbfeb9814bafacecd` (`0.468` ULP, matching
     glibc `__sin_fma`/`__sin_avx`/`__sin_sse2`), whereas `wasm32` `libm`
     returns `0xbfeb9814bafacecc` (`0.532` ULP).
   - OrcaSlicer and native Ares emit `G1 X100.25 Y129.249 E.02001` on layer 0
     and `G1 X100.25 Y129.249 E.02` on layers 1-2; unpatched WASM emits
     `Y129.248` and diverges across 5 G-code lines.
2. **40.05 mm cube witness** (`N = 27`, `thickness = 1.72165`,
   `ty = 110.000517`): at `s = 10.5`, `phase = 0xc047098522719b51`, correctly
   rounded `sin` is `0xbfebbff361954bd2` (`0.446` ULP) vs `wasm32` `libm`
   `0xbfebbff361954bd3` (`0.554` ULP), shifting `Y131.268` to `Y131.269`
   across 5 G-code lines.
3. **36 mm cube positive-phase witness** (`N = 26`, `thickness = 1.736865`,
   `tx = 110.000776`, `ty = 110.000776`): at `s = 76.0`,
   `phase = 0x40403a55ff8950f3`, correctly rounded `sin` is
   `0x3feb9814bafacecd` vs `wasm32` `libm` `0x3feb9814bafacecc`, shifting
   `Y93.749` to `Y93.748` across 5 G-code lines.

Additional arithmetic failure modes guarded against by the port:

- Unsigned 128-bit wrapping multiplication and addition in Payne-Hanek
  reduction (`reduce_large` and `reduce_large_acc`) must use explicit
  `wrapping_mul` / `wrapping_add` to avoid debug-mode overflow panics.
- Exponent indexing in `reduce_large_acc` for `2^-16 <= |x| < 2^-12`
  (`1007 <= e <= 1010`) must handle the virtual `T[-1] = 0` word cleanly.
- Signed zero (`-0.0`) must be preserved and subnormal inputs must not
  underflow to zero.

## Verification

1. **Standalone bit comparison**: compare `sin_f64` with CORE-MATH's
   `cr_sin` on hard-to-round `sin.wc` vectors and stratified random binary64
   inputs, without exposing the private kernel through a public API.
2. **Native parity (`crates/ares-cli/tests/orca_parity/fuzzy_skin_supported_area.rs`)**:
   slice `classic_ripple_sin_witness.3mf` against live Official and Seeded
   OrcaSlicer 2.5.0 with whole-file ordered byte equality (generator header
   excluded), plus the existing fuzzy and ripple parity suite.
3. **WASM entry-point and runtime verification**:
   verify `crates/ares-wasm/tests/ripple_sin_determinism.rs` against
   `ares-core`, and run the compiled `wasm32-unknown-unknown` module in Node
   (confirming zero JS math imports) for 4-way byte-identical G-code
   (Official Orca 2.5.0 == Seeded Orca 2.5.0 == Native CLI == WASM) across all
   three witnesses.
4. **Static and build checks**: `cargo fmt --all -- --check`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo check --target wasm32-unknown-unknown -p ares-core -p ares-wasm`,
   and `hk run pre-push`.
