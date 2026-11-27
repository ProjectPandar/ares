# Reduce WASM slicer function code

Reduce the code shipped for the existing `ares-core` rewrite without changing slicing behavior. The browser must still be able to slice the same projects, and each existing witness must emit exactly the same G-code bytes before and after the change.

## Source boundary

OrcaSlicer v2.5.0 at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19` is the source reference. All OrcaSlicer paths below are under `src/libslic3r`; the destinations are under `crates/ares-core/src`.

| OrcaSlicer boundary | Rust destination | Behavior retained |
| --- | --- | --- |
| `PrintObject.cpp:813–838`, `PrintObject::infill` | `project_slice/fill_entities/generate.rs:23–383`, `generate_layer_with_active` | Fill generation for each layer, with the existing pattern selection and geometry. |
| `Print.cpp:3982–4017`, `Print::export_gcode`; `GCode.cpp:2850–2880`, `GCode::do_export` | `project_slice/gcode_emit/orchestration.rs:9–345`, `emit_to_sink` | The same G-code export stages, options, and output bytes. |
| `GCode.cpp:2492–2515`, `GCode::collect_layers_to_print`; `GCode.cpp:5159–5179` and `5257–5270`, both `GCode::process_layers` overloads | `project_slice/gcode_emit/layers/schedule.rs:73–356`, `build`; `project_slice/gcode_emit/layers.rs:46–316`, `append` | Layer pairing, print order, and sequential or grouped emission. |
| `PrintConfig.cpp:998–1007`, `PrintConfigDef::PrintConfigDef` | `options/registry/print_config_def.rs:1–13`, `print_config_def_json` | Config metadata remains available unchanged; this slice does not rewrite or shrink the registry. |

These are existing rewrite boundaries. The `ares-wasm` byte-oriented adapter remains the browser entry point; no core logic moves into it and no new pipeline or compatibility layer is needed. Scope is WASM code generation and, only if justified by attribution, sharing equivalent internal code in the Rust destinations. Adjacent upstream behavior, including unported algorithms, native I/O and viewer behavior, stays out of scope. Do not change public bindings, options, diagnostics, algorithms, ordering, or floating-point semantics; do not introduce a feature flag that changes slicing.

## Size baseline and attribution

At Ares `7770f9d02a6681d0242afc9e25abe409c16282b5`, built for `wasm32-unknown-unknown` with the existing release profile (`opt-level=3`, LTO, one codegen unit, stripped symbols) and processed by `wasm-bindgen 0.2.121` with the web build's flags:

| Artifact | Bytes | `gzip -n -9` bytes | Function body bytes |
| --- | ---: | ---: | ---: |
| Raw `ares_wasm.wasm` | 14,405,742 | 4,736,605 | 13,284,296 (92.22%) |
| Browser `ares_wasm_bg.wasm` | 13,345,267 | 4,777,983 | 12,517,075 (93.79%) |

`twiggy top` on the stripped raw module reports five largest function entries of 87,173, 72,140, 59,441, 49,251 and 48,914 bytes, using Twiggy's shallow-size accounting. It cannot name them. The largest non-code entries are the initial data segment (804,090 bytes) and wasm-bindgen metadata (208,257 bytes); the metadata is not function code.

An earlier symbol-bearing release build had 13,155,820 function body bytes. Grouping demangled function names by their first crate path gives `ares_core` 8,725,882 bytes (66.33%), Rust `core` 2,080,870 (15.82%), `alloc` 350,550 (2.66%), `regex_automata` 284,368 (2.16%), `serde_json` 198,528 (1.51%), `ares_wasm` 149,075 (1.13%), and 425,346 unclassified (3.23%). These are shallow counts: an inlined callee is charged to the containing function after LTO. The earlier symbols guide the investigation but are not assigned to the stripped baseline by size or index.

| Largest named bodies in the earlier build | Bytes |
| --- | ---: |
| `ares_core::project_slice::fill_entities::generate::generate_layer_with_active` | 87,172 |
| `ares_core::project_slice::gcode_emit::layers::append` | 57,523 |
| `ares_core::project_slice::gcode_emit::layers::schedule::build` | 56,943 |
| `regex_automata::meta::strategy::new` | 48,913 |
| `ares_core::project_slice::gcode_emit::orchestration::emit` | 47,554 |
| `ares_core::project_slice::gcode_emit::orchestration::state::build` | 46,228 |

Before selecting a source change, repeat the attribution at the exact baseline with Rust's WASM function names retained, keeping the release optimization inputs fixed. Compare code and data sections before mapping the names to the stripped artifact; if they differ, report the symbol-bearing results separately. Use the production build path for before/after size and timing, changing only the candidate under measurement.

## Candidates

1. **WASM-only size-oriented optimization.** Compare `opt-level=s` with `3`, retaining LTO, one codegen unit, and stripped output. Consider `z` only as a separate measurement; smaller code is not worth slower slicing or startup. A 1–3% reduction of the measured raw function body budget would save roughly 130–400 kB before gzip. This is a planning estimate, not a measured gain. Leave the native release profile unchanged.
2. **Share repeatedly instantiated internal work.** The earlier build had 722 named `core::slice::sort` bodies totaling 926,911 bytes. Find which core call sites retain those bodies and look for identical work in different instantiations; a Rust `core` symbol does not by itself prove an Ares call site or removable duplication. If 5–10% of those bytes prove redundant, expect about 45–95 kB raw savings. Share the type-independent operation, not the choice of sort, the comparator, or tie ordering. Prefer a single measured change over combining this estimate with the profile estimate.
3. **Defer broad panic, formatting, and data changes.** The WASM target already defaults to `panic="abort"`; setting it again should save nothing. The earlier standalone `core`/`alloc` formatting and `std`/`core` panic bodies account for only about 11 kB together, excluding inlined code. Preserve the panic hook and error text. Static table compression, dependency replacement and binding removal are outside this function-code slice.

## Acceptance

- Record raw and browser module bytes, deterministic gzip bytes, and function body bytes on the same toolchain and build path before and after. Require both raw and gzip browser size to improve and verify the function body reduction. Keep the measurements for each candidate separate.
- Compare web cold starts in fresh browser contexts with cache disabled: use the same production build, browser version, device and fixtures. Measure module readiness and time to first slice; also measure warmed slicing. Report medians and spread over repeated runs, and reject a reproducible regression.
- Compare base and candidate SHA-256 of the entire G-code emitted by the existing browser witnesses in both Official and Seeded runs, including the Ender3 and KSR WASM fixtures and the Arachne and tool-change witnesses. Use identical inputs and settings and retain the existing browser-to-native comparisons. Compare native base and candidate G-code for KSR, Arachne and tool-change workloads as well. Do not substitute normalized or truncated output for the byte comparison.
- Run the workspace non-parity suite, a focused OrcaSlicer parity sample in both Official and Seeded modes, and the workspace and WASM static checks, including WASM Clippy. Do not relax comparators or count skipped Native comparisons. If no candidate improves the shipped size while preserving behavior and performance, ship no code change.
