# M126 Spec: PrintConfig support ironing registry slice

## Goal
Port the adjacent support interface ironing option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:997`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406-6412`: `support_ironing` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:998`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:87-98`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:225-255`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6414-6424`: `support_ironing_pattern` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:999`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6426-6436`: `support_ironing_flow` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1000`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6438-6446`: `support_ironing_spacing` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/maximums/modes beyond the current registry metadata boundary.
- Support interface ironing behavior, pattern application, flow/spacing behavior, support interface generation, support geometry, and support path generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6448+`: `activate_chamber_temp_control`, `chamber_temperature`, and following chamber-temperature options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: add the four covered `support_ironing*` definitions after `support_interface_top_layers` and before `support_line_width` in sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the four covered expected keys after `support_interface_top_layers` and before `support_line_width`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/support_ironing.rs`: add metadata assertions for all four definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_support_ironing.rs`: add public lookup assertions for all four definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values and expected counts without touching near-limit `values.rs`.
- `docs/roadmap.md` and `docs/milestones/m126-print-config-support-ironing-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_ironing` (`coBool`, default `false`, field at `PrintConfig.hpp:997`, definition lines 6406-6412, Ares kind `Bool`)
- `support_ironing_pattern` (`coEnum` over `InfillPattern`, default `rectilinear`, field at `PrintConfig.hpp:998`, enum map lines 225-255, definition lines 6414-6424, Ares kind `Enum`)
- `support_ironing_flow` (`coPercent`, default `10`, field at `PrintConfig.hpp:999`, definition lines 6426-6436, Ares kind `Percent`)
- `support_ironing_spacing` (`coFloat`, default `0.1`, field at `PrintConfig.hpp:1000`, definition lines 6438-6446, Ares kind `Float`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, support-ironing behavior, support interface behavior, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `activate_chamber_temp_control`, `chamber_temperature`, or following options from `PrintConfig.cpp:6448+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6448+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
