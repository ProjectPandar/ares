# M192 Spec: PrintConfig get_parameter_size API

## Goal
Port OrcaSlicer's `DynamicPrintConfig::get_parameter_size(const std::string&, size_t)` into Ares as a `SliceOptions` API for UI/config consumers that need to size per-extruder and variant-expanded option arrays.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8529-8556`: `get_parameter_size` default lengths, variant length reads from `filament_extruder_variant`, `print_extruder_variant`, and `printer_extruder_variant`, key-set precedence, printer variant-2 doubling, and extruder-count fallback.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:633`: public declaration.
- Key-set context already ported by M184 from `PrintConfig.cpp:8154-8287` and exposed in Ares registry APIs: `print_options_with_variant`, `filament_options_with_variant`, `printer_options_with_variant_1`, and `printer_options_with_variant_2`.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:8558+` `extend_extruder_variant`, `set_num_extruders`, `set_num_filaments`, array resizing/mutation, `FullPrintConfig::defaults`, preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/parameter_size.rs`: add `SliceOptions::parameter_size(&self, param_name: &str, extruder_nums: usize) -> Result<usize, SliceError>`.
- `crates/ares-core/src/options.rs`: register the new module.
- `crates/ares-core/src/options/tests/parameter_size.rs`: add source-behavior tests.
- `crates/ares-core/src/options/tests.rs`: register the new test module.
- `docs/roadmap.md` and `docs/milestones/m192-print-config-parameter-size-api.md`: milestone sequencing docs.

## Functional requirements

1. Add a public `SliceOptions::parameter_size(param_name, extruder_nums)` API.
2. Default `filament_variant_length`, `process_variant_length`, and `machine_variant_length` to `1` when their corresponding variant option key is absent.
3. When `filament_extruder_variant` is present, use its string-vector length for filament variant options.
4. When `print_extruder_variant` is present, use its string-vector length for print/process variant options.
5. When `printer_extruder_variant` is present, use its string-vector length for printer variant options.
6. Match upstream key-set precedence exactly: `printer_options_with_variant_1` first, then `printer_options_with_variant_2`, then `filament_options_with_variant`, then `print_options_with_variant`, otherwise `extruder_nums`.
7. For keys in `printer_options_with_variant_1`, return `machine_variant_length`.
8. For keys in `printer_options_with_variant_2`, return `machine_variant_length * 2`.
9. For keys in `filament_options_with_variant`, return `filament_variant_length`.
10. For keys in `print_options_with_variant`, return `process_variant_length`.
11. For all other keys, return `extruder_nums`.
12. Treat present variant-length source keys as Orca `ConfigOptionStrings`: accept JSON arrays containing only strings and reject non-array or non-string-member values with `SliceError::InvalidInput`.
13. Preserve existing registry key-set APIs, M185 object-distance API, M186-M190 FDM APIs, and M191 legacy SLA behavior.
14. Do not add array resizing, `extend_extruder_variant`, `set_num_extruders`, `set_num_filaments`, `FullPrintConfig::defaults`, UI runtime behavior, slicing, extrusion, G-code behavior, new crates, or dependencies.
15. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove absent variant source keys default each variant length to `1`.
- Tests prove `printer_options_with_variant_1` keys return `printer_extruder_variant` length.
- Tests prove `printer_options_with_variant_2` keys return twice `printer_extruder_variant` length.
- Tests prove `filament_options_with_variant` keys return `filament_extruder_variant` length.
- Tests prove `print_options_with_variant` keys return `print_extruder_variant` length.
- Tests prove fallback keys return the provided `extruder_nums`.
- Tests prove upstream precedence by using a key that appears in `filament_options_with_variant` and another variant source only affects its owning key set.
- Tests prove invalid present variant source values return `SliceError::InvalidInput`.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8558+` resizing and extruder-variant behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
