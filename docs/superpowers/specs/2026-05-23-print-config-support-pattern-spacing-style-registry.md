# M122 Spec: PrintConfig support pattern spacing, speed, expansion, and style registry slice

## Goal
Port the adjacent support base-pattern spacing, normal support expansion, support speed, and support style option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:972`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6178-6185`: `support_base_pattern_spacing` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:973`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6187-6193`: `support_expansion` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:974`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6195-6202`: `support_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:179-181`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:975`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:322-331`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6204-6230`: `support_style` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/enum UI values/modes beyond the current registry metadata boundary.
- Support base-pattern spacing behavior, support expansion behavior, support speed assignment behavior, support style selection behavior, style interactions with tree/normal support, support geometry, and support path generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6232+`: `independent_support_layer_height` and following support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: add four covered support definitions in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all four definitions.
- `docs/roadmap.md` and `docs/milestones/m122-print-config-support-pattern-spacing-style-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_base_pattern_spacing` (`coFloat`, default `2.5`, field at `PrintConfig.hpp:972`, definition lines 6178-6185, Ares kind `Float`)
- `support_expansion` (`coFloat`, default `0`, field at `PrintConfig.hpp:973`, definition lines 6187-6193, Ares kind `Float`)
- `support_speed` (`coFloat`, default `80`, field at `PrintConfig.hpp:974`, definition lines 6195-6202, Ares kind `Float`)
- `support_style` (`coEnum`, default `default`, enum at `PrintConfig.hpp:179-181`, enum map lines 322-331, field at `PrintConfig.hpp:975`, definition lines 6204-6230, Ares kind `Enum`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, support spacing behavior, support expansion behavior, support speed behavior, support style behavior, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `independent_support_layer_height` or following options from `PrintConfig.cpp:6232+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6232+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
