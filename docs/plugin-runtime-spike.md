# Plugin runtime spike measurements

[Plugin system](plugin-system.md#runtime-spike)

## Results

Reproduce with the [prototype and instructions](https://github.com/ProjectPandar/ares/tree/44327f0e329fc84caed08dd4fbe9fd9f15189ccd/spikes/plugin-runtime) at `44327f0e329fc84caed08dd4fbe9fd9f15189ccd` (runtime code measured at `9cfecead950b6117624661bdbb73f07c25ff7010`; only reporting changed). This is a synthetic benchmark, not a slicer or 3MF integration.

**Method.** Linux x86-64 (AMD EPYC 7D12) and Chrome 156.0.8078.79 on the same machine, with fixed processor affinity and one hook stream. Each completed case is the median of five release runs over 300 layers × 2,000 paths × 200 points. Hosted timing starts with a serialized layer; it includes dispatch and buffer transfers, but excludes initial serialization and hashing. Native compiled Rust times the typed hook alone; Chrome compiled Rust includes binding calls and byte projection. Load excludes file/network I/O and browser host initialization. Reload includes rerunning the first layer, with verification outside the timer; compiled Rust cannot reload.

**Timing.** Each cell is ms/layer / total ms for 300 layers / load ms / reload ms. Direct means compiled Rust or the JS engine linked into the host, not a separate guest. The table uses no explicit Wasm fuel or memory caps. Both Chrome Wasmi+Boa cases ended with guest exit 71; a staggered rerun reproduced it on layer index 166 of the first trial. The cause was not isolated.

| Placement | Plugin | Linux staggered | Linux subdivision | Chrome staggered | Chrome subdivision |
| --- | --- | ---: | ---: | ---: | ---: |
| Direct | Rust | 0.009 / 2.657 / — / — | 0.007 / 2.084 / — / — | 0.634 / 190.250 / — / — | 0.659 / 197.735 / — / — |
| Direct | QuickJS | 3.719 / 1115.703 / 0.838 / 4.751 | 4.021 / 1206.266 / 0.790 / 4.860 | 5.416 / 1624.790 / 1.560 / 7.110 | 5.657 / 1697.195 / 1.845 / 7.500 |
| Direct | Boa | 9.405 / 2821.588 / 6.469 / 15.259 | 9.852 / 2955.713 / 6.200 / 17.144 | 20.105 / 6031.390 / 6.065 / 25.540 | 20.818 / 6245.515 / 5.895 / 26.680 |
| Wasmi guest | Rust | 0.782 / 234.526 / 0.499 / 2.490 | 0.839 / 251.627 / 0.530 / 2.649 | 4.393 / 1317.755 / 1.100 / 8.060 | 4.730 / 1419.035 / 1.090 / 7.960 |
| Wasmi guest | QuickJS | 43.025 / 12907.491 / 40.303 / 57.680 | 44.106 / 13231.783 / 42.619 / 59.720 | 174.598 / 52379.300 / 89.385 / 217.790 | 183.975 / 55192.650 / 90.105 / 226.810 |
| Wasmi guest | Boa | 251.638 / 75491.352 / 132.903 / 280.696 | 266.327 / 79898.165 / 127.991 / 295.905 | WASI exit 71 | WASI exit 71 |
| Wasmtime / browser WebAssembly | Rust | 0.505 / 151.503 / 51.999 / 59.189 | 0.556 / 166.718 / 49.587 / 57.802 | 0.604 / 181.210 / 0.345 / 6.410 | 0.604 / 181.135 / 0.425 / 5.815 |
| Wasmtime / browser WebAssembly | QuickJS | 6.108 / 1832.270 / 2003.979 / 12.341 | 5.978 / 1793.315 / 1959.986 / 12.771 | 5.470 / 1641.095 / 6.375 / 12.050 | 4.584 / 1375.205 / 5.175 / 9.770 |
| Wasmtime / browser WebAssembly | Boa | 24.384 / 7315.230 / 8936.503 / 29.292 | 25.862 / 7758.671 / 8934.465 / 28.900 | 24.071 / 7221.405 / 23.515 / 28.055 | 18.434 / 5530.185 / 21.890 / 28.495 |

**Size and portability.** Against a 13,426,499-byte release `ares-wasm` baseline (raw bindgen Wasm, not compressed), the separate variants added 45,557 B (compiled Rust), 1,272,397 B (Wasmi 2.0.3), 1,330,200 B (Wasmi with virtual files), 1,086,158 B (direct QuickJS via `rquickjs` 0.14.0), or 5,202,580 B (direct Boa 0.22.0). Browser WebAssembly needs no Ares runtime linkage. Guest binaries were 26,505/26,681 B (Rust), 1,216,950 B (QuickJS), and 5,941,814 B (Boa). Linux and `wasm32-unknown-unknown` built and ran in their respective environments. Windows MSVC and macOS arm64 `cargo check` passed for Wasmi and direct Boa; Wasmtime 51.0.0 checked on Windows only (macOS C compiler/SDK unavailable). QuickJS requires a target C toolchain (Clang for Wasm here); Windows/macOS QuickJS checks stalled at that prerequisite. No Windows/macOS execution was tested.

**Determinism.** All 34 completed rows (18 Linux, 16 Chrome) matched edit SHA-256 by recipe: staggered `c4b2f0c5c8fcf166ea5cd11e7ca46d3c13282c085bd417c8a757b42ec096688b`; subdivision `1523d50735292602dd257ac2fc008eec8ea61b7512c89d3abfca043aa096fd90`. G-code SHA-256 matched within Rust (`d4b35d7f4f3287b4d97c552b23bb17dbb52d12553570ba1c6254c1071988a985`) and across both JS engines with pinned `libm` and canonical NaNs (`dc626b844d70b4676d7bcecec5b3ea54e618a417be2795b38cfce5f189e5daf4`). Unmodified math matched across Wasmtime and browser WebAssembly for the same guest, but not for direct JS: `tan(7)` differed by one bit in both engines; direct QuickJS also differed on `hypot(0.17, 0.93)` and the NaN sign for `sin(Infinity)`. This is a numeric sample, not a guarantee for arbitrary JS.

**Limits.** With 1,000 fuel, Wasmi trapped after 50 counter increments on Linux and Chrome, Wasmtime after 72; browser WebAssembly had no fuel API and completed 100,000. All three rejected a two-page growth above the two-page cap. Metering plus a memory cap moved staggered Rust from 234.5→268.3 ms (Linux Wasmi), 151.5→169.2 ms (Wasmtime), and 1317.8→1419.2 ms (Chrome Wasmi). Direct QuickJS interrupted at 15,000 increments and rejected a test buffer above its allocator limit; Boa stopped at 1,001 with a loop limit, not comprehensive metering or a heap cap. Boa guests needed a fixed-epoch WASI shim; the QuickJS core guest needed a reduced engine stack bound. Neither provides a real clock.

**Files.** Wasmi exercised both a minimal WASI p1 subset supporting Rust `std::fs` and two custom imports. Median µs per read+write over 10,000 pairs: Linux 5.591 (WASI) vs 0.867 (custom); Chrome 35.604 vs 6.119. IndexedDB snapshot/commit, outside the hook, was 0.430/1.540 ms for WASI and 0.370/0.715 ms for custom. The fixture read a package file and checked staged writes, slice-start visibility, and rejection of absolute/traversal paths. Prefer the narrow WASI subset if Rust source compatibility matters; custom calls are leaner in this test. The fixture does not prove package hashing, plugin identity, or production isolation.

**Components.** Wasmtime loaded and replaced a 307-byte list-of-bytes component. Chrome rejected it as a raw module. Jco 1.42.0 translated and replaced it in the worker without a build-time transpile step: 163 ms tool import, 672 ms first translation/load/call, 245 ms reload, 6.32 MB raw tool resources. The plain core module avoids shipping this translator; for the staggered Rust guest, Chrome WebAssembly load/reload was 0.345/6.410 ms and Chrome Wasmi was 1.100/8.060 ms.

**Initial recommendation.** Keep the typed hook in core and the reloadable core-module byte ABI in the host. Use compiled Rust for built-ins and test Wasmi with Rust guests first for hosted plugins: its staggered hook total was 235 ms on Linux and 1,318 ms in Chrome, versus 12.9 s and 52.4 s for QuickJS in the same interpreter. Defer JS as a default for per-layer hooks. If JS is needed, evaluate QuickJS inside metered Wasm before Boa-in-Wasm; the tighter runtime and smaller guest both favor QuickJS. Direct QuickJS is only for trusted local use; direct Boa still needs an effective operation budget and heap cap. Treat unmetered browser WebAssembly and engine-specific native fuel as unsuitable for a shared untrusted budget without additional instrumentation.

### Metering follow-up

The [portable fuel prototype](https://github.com/ProjectPandar/ares/tree/11e0482657d324862be35bd0e6f7e89684cae132/spikes/plugin-runtime) uses `wasmparser` and `wasm-encoder` 0.263.0 to add one debit per straight-line segment and set the linear memory maximum. Wasmtime 51.0.0 on Linux and Chrome WebAssembly produced byte-identical rewritten counter modules: with 1,000 units both trapped after 71 increments, allowed growth from one page to two, and returned `-1` for further growth. All 80 timing trials matched the earlier edit and G-code SHA-256 values. Bulk memory (`memory.copy` and `memory.fill`), table operations, and host imports receive only static charges. This bounds guest instruction count, not wall time. The linear memory cap does not cover tables, stacks, or host allocations.

**Timing.** Same method and original guest bytes as above. Each cell is ms/layer / load ms / reload ms. Rust reload replaces the module and reruns the first layer. QuickJS reload replaces the source in the existing runner and reruns the first layer; the fourth number is replacement using a fresh runner. The current uninstrumented runs were slower than the earlier rows, so use the paired columns to estimate the metering cost. Rust hook times stayed within 11% of the paired uninstrumented runs; QuickJS hook times rose 26–35%. Fresh runner load and replacement cost more, especially for QuickJS.

| Engine | Guest | Recipe | Uninstrumented | Instrumented |
| --- | --- | --- | ---: | ---: |
| Wasmtime | Rust | Staggered | 0.801 / 63.150 / 78.847 | 0.820 / 99.958 / 122.082 |
| Wasmtime | Rust | Subdivision | 0.804 / 64.981 / 77.899 | 0.747 / 101.689 / 116.156 |
| Wasmtime | QuickJS | Staggered | 7.171 / 2507.628 / 21.807 / 2512.908 | 9.713 / 4685.580 / 24.407 / 4804.196 |
| Wasmtime | QuickJS | Subdivision | 7.495 / 2455.673 / 21.304 / 2481.428 | 9.446 / 4425.881 / 24.219 / 4592.416 |
| Chrome | Rust | Staggered | 1.021 / 0.595 / 11.210 | 1.041 / 3.525 / 15.855 |
| Chrome | Rust | Subdivision | 1.050 / 1.575 / 12.925 | 1.159 / 4.295 / 17.160 |
| Chrome | QuickJS | Staggered | 6.701 / 10.850 / 20.095 / 51.960 | 9.012 / 101.320 / 24.110 / 141.750 |
| Chrome | QuickJS | Subdivision | 7.732 / 11.265 / 22.710 / 49.325 | 10.202 / 107.910 / 26.410 / 163.380 |

**Size.** The parser, encoder and binding added 498,683 B to the raw `ares-wasm` binary (13,426,499 → 13,925,182 B). The guest debit count is the number of inserted sites, not how often the sites run.

| Guest module | Inserted debits | Code section bytes | Total module bytes |
| --- | ---: | ---: | ---: |
| Rust staggered | 1,272 | 23,578 → 43,945 | 26,505 → 46,893 |
| Rust staggered replacement | 1,272 | 23,578 → 43,945 | 26,505 → 46,893 |
| Rust subdivision | 1,277 | 23,750 → 44,199 | 26,681 → 47,151 |
| Rust subdivision replacement | 1,277 | 23,760 → 44,209 | 26,691 → 47,161 |
| QuickJS runner | 64,052 | 1,080,470 → 2,105,656 | 1,216,950 → 2,242,158 |

**Choice.** `wasm-instrument` 0.4.0 metered the counter but could not parse the recipe guests (`UnknownOpcode(252)`); enabling `parity-wasm` bulk memory support failed to compile because that release did not handle the bulk instructions in a match. The shared parser/encoder path removes the need for Wasmi in the measured core-module and QuickJS paths: neither build selected it. This does not establish a production sandbox or replace the virtual file and component experiments. Boa was not pursued in this follow-up.
