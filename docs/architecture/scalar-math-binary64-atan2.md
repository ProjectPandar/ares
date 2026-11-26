# Deterministic binary64 `atan2` for Clipper2 round offsets and bridge direction binning

## Source boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) evaluates binary64 `std::atan2` at two geometry boundaries whose rounding drives discrete downstream branching:

1. `deps_src/clipper2/Clipper2Lib/src/clipper.offset.cpp:370` (`Clipper2Lib::ClipperOffset::OffsetPoint`) calls `DoRound(path, j, k, std::atan2(sin_a, cos_a))`, and `clipper.offset.cpp:302` (`ClipperOffset::DoRound`) converts that angle into an integer arc step count via `std::max(static_cast<int>(std::round(steps_per_rad_ * std::abs(angle))), 1)`.
2. `src/libslic3r/BridgeDetector.hpp:98` (`Slic3r::detect_bridging_direction`) quantizes each floating overhang edge normal via `double quantized_angle = std::ceil(std::atan2(normal.y(), normal.x()) * 1000.0);` before inserting into `std::unordered_map<double, Vec2d>` (`emplace` keeps the first normal per quantized bin).

In Ares, the corresponding Rust call sites are:

- `crates/ares-core/src/geometry/clipper2/offset/do_square.rs:188` (`ClipperOffset::offset_point`)
- `crates/ares-core/src/geometry/bridge_direction.rs:130` (`detect_bridging_direction`)

Both currently delegate to `f64::atan2`, which is a temporary target-dependent shell: on x86-64 Linux it dispatches to glibc `__ieee754_atan2_fma` (`libm.so.6`), whereas on `wasm32-unknown-unknown` it dispatches to `compiler_builtins::libm` (`0.2.16`), which can differ by 1 ULP on binary64 inputs near midpoint or quantization boundaries.

## Rust destination and scope

Port the MIT-licensed CORE-MATH correctly-rounded binary64 `atan2` kernel (`src/binary64/atan2/atan2.c` and `src/binary64/atan2/tint.h`, revision `3270006242ceef2885767918533ac5446807f2cf`) into pure safe Rust under `crates/ares-core/src/scalar_math/`:

- `crates/ares-core/src/scalar_math.rs`: minimal export `mod atan2_f64; pub(crate) use atan2_f64::atan2_f64;`.
- `crates/ares-core/src/scalar_math/atan2_f64.rs`: `pub(crate) fn atan2_f64(y: f64, x: f64) -> f64`, special-value dispatch, fast path, medium path, accurate path, and exact tiny-ratio boundary resolution (`x > 0`, `|y / x| < 2^-30`).
- `crates/ares-core/src/scalar_math/atan2_f64/tables.rs`: lookup tables (`T2`, `F2`, `O`), polynomial coefficients (`B`, `B2`), and 192-bit rational approximation coefficients (`P`, `Q`), preserving complete MIT copyright attribution.
- `crates/ares-core/src/scalar_math/atan2_f64/tint.rs`: 192-bit floating-point arithmetic (`Tint`) using safe `u128`/`u64` operations with zero `unsafe`.
- Wire `crate::scalar_math::atan2_f64` at `do_square.rs:188` and `bridge_direction.rs:130`.

### Included behavior

- IEEE-754 binary64 round-to-nearest-even (`<= 0.5` ULP) across all four quadrants, signed zeros, infinities, NaNs, subnormals, extreme exponent ratios, and tiny-ratio cancellation cases (`|y / x| < 2^-30`, `x > 0`, including 151-bit cancellation pairs such as `y = 0x1.8000000000003p-51`, `x = 0x1.0000000000003p+0`).
- Zero `unsafe`, zero FFI, and zero JavaScript `Math` imports on `wasm32-unknown-unknown`.
- Exact agreement with glibc `__ieee754_atan2_fma` on all correctly-rounded binary64 outputs (note: glibc `e_atan2.c` documents a ~0.524–0.56 ULP worst-case bound on rare random midpoint inputs where IEEE-754 correct rounding can differ by 1 ULP from glibc).

### Deferred behavior

- Dynamic IEEE-754 rounding modes (`FE_TOWARDZERO`, `FE_UPWARD`, `FE_DOWNWARD`), floating-point exception flags, and `errno`.
- Other binary64 transcendental kernels (`sin`, `cos`, `acos`).
- Other `f64::atan2` call sites outside these two proved boundaries.

## Failure modes

1. **Clipper2 round-join skirt arc vertex drop (`clipper.offset.cpp:370,302`)**:
   On a 4-vertex prism with centered edge `(-14_675_676, +20_006_491)` following `(+30_000_000, 0)` and a single skirt loop (`skirt_distance = 4.8214603125648496`, giving `abs_delta = 5_000_000`, `arc_tolerance = 100_000`), `OffsetPoint` calls `atan2(y, x)` with `y = 0x3fe9cd668f6f79c9` (`+0.8063233186219857`) and `x = 0xbfe2ed5d08641301` (`-0.59147502554751`).
   - Correctly-rounded binary64 and glibc `__ieee754_atan2_fma` return `0x4001a124b12262dd`, giving `steps_per_rad * |angle| = 5.5` exact -> `round = 6` steps (5 intermediate arc vertices, emitting `G1 X129.539 Y102.096 E.0579` followed by `G1 X129.032 Y102.957 E.0291`).
   - WebAssembly `libm` returns `0x4001a124b12262dc` (1 ULP below true value), giving `5.499999999999998` -> `round = 5` steps, dropping vertex `(129.539, 102.096)` and emitting a single chord `G1 X129.032 Y102.957 E.08613`.

2. **Bridge detector normal-bin collision (`BridgeDetector.hpp:98`)**:
   On a C-supported bridge overhang (`wall_loops = 0`) whose floating overhang edge has exact scaled delta `(+2_563_503, -18_513_382)` followed by a longer ~25.5 mm edge at normal angle `-3.00309 rad` (bin `-3003`), `detect_bridging_direction` calls `atan2(y, x)` with `y = 0xbfc18e6c692dd410` and `x = 0xbfefb293eea5172e`.
   - Correctly-rounded binary64 and glibc `__ieee754_atan2_fma` return `0xc008083126e978d5` (`-3.00400000000000000355...`), so `ceil(angle * 1000.0) = -3004.0`; the two edges enter distinct bins and the longer edge wins minimum cost (`G1 X113.053 Y101.504 Z1.6`).
   - WebAssembly `libm` returns `0xc008083126e978d4` (`-3.00399999999999955946...`), so `ceil(angle * 1000.0) = -3003.0`; the shorter edge occupies bin `-3003` first, `emplace` drops the longer edge, and bridge infill direction flips across `;TYPE:Bridge` moves.

## Verification

1. **Bit-level kernel tests (`crates/ares-core/src/scalar_math/atan2_f64/tests.rs`)**:
   Verify exact IEEE-754 round-to-nearest-even bit patterns across all special values (`±0.0`, `±∞`, `NaN`), all four quadrants, subnormals, extreme exponent ratios, the two slicing witness operand pairs (`0x4001a124b12262dd` and `0xc008083126e978d5`), and CORE-MATH worst-case vectors including tiny-ratio 151-bit cancellation pairs.
2. **Whole-file slicing parity (`crates/ares-cli/tests/orca_parity/option_coverage/initialized_bridge_direction/atan2_determinism.rs`)**:
   Synthetically construct both 3MF witnesses from `tests/parity/outer-only-brim/rectangle.3mf`, assert runtime OrcaSlicer emits the expected skirt arc vertex and bridge geometry lines, and require generator-only whole-file byte equality for Ares CLI and public API in both official and seeded modes.
3. **Cross-target WebAssembly determinism**:
   Compile `ares-wasm` to `wasm32-unknown-unknown`, confirm zero JavaScript `Math` imports in `ares_wasm_bg.wasm`, and verify byte-identical G-code (generator line excluded) across official OrcaSlicer, seeded OrcaSlicer, native Ares CLI, and WebAssembly `sliceProjectPlateWithCatalog` on both witnesses.
4. **Focused regression and target checks**:
   Run focused official and seeded parity across clipper, bridge, skirt, brim, and counterbore tests, `cargo nextest run -p ares-wasm`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo check --target wasm32-unknown-unknown -p ares-core -p ares-wasm`.
