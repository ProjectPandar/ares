# M121 Spec: PrintConfig support bottom interface spacing, interface speed, and patterns registry slice

## Goal
Port the adjacent support bottom-interface spacing, support-interface speed, support base pattern, and support interface pattern option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1019`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6114-6122`: `support_bottom_interface_spacing` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:968`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6124-6131`: `support_interface_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:172-177`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:969`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:312-320`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6133-6156`: `support_base_pattern` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:190-192`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:970`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:333-340`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6158-6176`: `support_interface_pattern` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/enum UI values/modes beyond the current registry metadata boundary.
- Support bottom-interface spacing behavior, support interface speed behavior, support base/interface pattern selection behavior, support geometry, support style interaction, and support path generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6178+`: `support_base_pattern_spacing`, `support_expansion`, `support_speed`, `support_style`, and following support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: add four covered support definitions in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all four definitions.
- `docs/roadmap.md` and `docs/milestones/m121-print-config-support-interface-pattern-speed-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_bottom_interface_spacing` (`coFloat`, default `0.5`, field at `PrintConfig.hpp:1019`, definition lines 6114-6122, Ares kind `Float`)
- `support_interface_speed` (`coFloat`, default `80`, field at `PrintConfig.hpp:968`, definition lines 6124-6131, Ares kind `Float`)
- `support_base_pattern` (`coEnum`, default `default`, enum at `PrintConfig.hpp:172-177`, enum map lines 312-320, field at `PrintConfig.hpp:969`, definition lines 6133-6156, Ares kind `Enum`)
- `support_interface_pattern` (`coEnum`, default `auto`, enum at `PrintConfig.hpp:190-192`, enum map lines 333-340, field at `PrintConfig.hpp:970`, definition lines 6158-6176, Ares kind `Enum`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, support bottom-interface spacing behavior, support interface speed behavior, support pattern selection behavior, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_base_pattern_spacing`, `support_expansion`, `support_speed`, `support_style`, or following options from `PrintConfig.cpp:6178+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6178+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
