# M184 Spec: PrintConfig variant option key sets

## Goal
Port OrcaSlicer's global variant-related option key sets into `ares-core` as read-only option registry API data for downstream UI/API consumers and future source-cited variant-expansion milestones.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8155-8185`: `print_options_with_variant` active keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8187-8227`: `filament_options_with_variant` active keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8229-8238`: `printer_extruder_options` keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8240-8266`: `printer_options_with_variant_1` keys, including upstream duplicate `nozzle_volume` initializer.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8268-8285`: `printer_options_with_variant_2` keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8287`: `empty_options` placeholder.

Related upstream behavior explicitly deferred:

- Commented-out candidate keys in `print_options_with_variant`.
- Variant expansion or lookup behavior that consumes these sets.
- Silent-mode behavior implied by the `printer_options_with_variant_2` comment.
- Extruder-count expansion, filament override/following behavior, typed option accessors, option parsing changes, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, UI runtime behavior, and any Ares-owned pipeline changes.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8289+`: `DynamicPrintConfig::full_print_config()` and later normalization/runtime behavior.

Rust destination boundary:

- `crates/ares-core/src/options/registry/key_lists.rs`: add static key lists and public functions.
- `crates/ares-core/src/options/registry.rs`: re-export the key-list functions from the registry module.
- `crates/ares-core/src/lib.rs`: re-export the key-list functions for UI/API consumers.
- `crates/ares-core/src/options/registry/tests/key_lists.rs`: add sorted set contents and registry-coverage assertions.
- `crates/ares-core/src/options/tests/registry_lookup_variant_option_sets.rs`: add public API assertions.
- `crates/ares-core/src/options/tests.rs`: add the public lookup test module.
- `docs/roadmap.md` and `docs/milestones/m184-print-config-variant-option-sets.md`: milestone sequencing docs.

## Included API data

Expose these functions returning `&'static [&'static str]`:

- `print_options_with_variant()`
- `filament_options_with_variant()`
- `printer_extruder_options()`
- `printer_options_with_variant_1()`
- `printer_options_with_variant_2()`

The upstream containers are `std::set<std::string>`, so the Rust lists must preserve sorted unique set semantics rather than initializer order. `printer_options_with_variant_1()` must contain one `nozzle_volume` entry even though the upstream initializer repeats it.

`empty_options` is a source-cited empty set placeholder and is intentionally not exposed in this milestone because it has no data and no covered consumer.

## Functional requirements

1. Add read-only functions returning `&'static [&'static str]` for the five non-empty variant option sets.
2. Preserve sorted unique `std::set` semantics for every exposed list.
3. Verify every exposed list is sorted and contains no duplicates.
4. Verify every exposed key resolves through `option_definition()`.
5. Verify public crate-level exports return the expected lists for UI/API consumers.
6. Preserve existing public API function signatures and existing M153 key-list functions.
7. Do not add variant expansion, silent-mode resolution, extruder-count expansion, filament override resolution, typed parsing/accessors, option parsing behavior, slicing behavior, extrusion behavior, G-code behavior, UI runtime behavior, new crates, or dependencies.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all five lists match expected sorted unique upstream set contents.
- Registry tests prove every variant option set is sorted and duplicate-free.
- Registry tests prove every list key resolves through `option_definition()`.
- Public API tests prove crate-level exports are wired for all five lists.
- Plan/spec explicitly account for deferred variant runtime behavior and `PrintConfig.cpp:8289+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
