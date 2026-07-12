# M95 Spec: PrintConfig resonance avoidance and input shaping registry slice

## Goal
Port the adjacent resonance avoidance and input shaping option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1277`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4516-4523`: `resonance_avoidance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1278`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4525-4531`: `min_resonance_avoidance_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1279`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4533-4539`: `max_resonance_avoidance_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1282`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4541-4546`: `input_shaping_emit` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:365-379,544,1283`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:503-518,4548-4555`: `InputShaperType` enum map and `input_shaping_type` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1284`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4557-4564`: `input_shaping_freq_x` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1285`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4566-4573`: `input_shaping_freq_y` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1286`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4575-4581`: `input_shaping_damp_x` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1287`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4583-4589`: `input_shaping_damp_y` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/mode/readonly and enum-label metadata beyond the current registry boundary.
- Resonance avoidance speed planning and ringing mitigation behavior.
- Input-shaping G-code emission, firmware override/disable behavior, and printer-firmware integration.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4591+`: fan max speed, layer-height, extrusion-rate smoothing, nozzle, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for the six `input_shaping_*` keys.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add sorted definitions for `max_resonance_avoidance_speed` and `min_resonance_avoidance_speed`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definition for `resonance_avoidance`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the nine new expected registry keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod resonance_input_shaping;`.
- `crates/ares-core/src/options/registry/tests/metadata/resonance_input_shaping.rs`: source metadata assertions for all nine options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_resonance_input_shaping;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_resonance_input_shaping.rs`: public lookup coverage for all nine options.
- `docs/roadmap.md` and `docs/milestones/m95-print-config-resonance-input-shaping-registry.md`: milestone sequencing docs.

## Included option definitions

- `resonance_avoidance` (`coBool`, default `false`, field at `PrintConfig.hpp:1277`, definition lines 4516-4523, Ares kind `Bool`)
- `min_resonance_avoidance_speed` (`coFloat`, default `70`, field at `PrintConfig.hpp:1278`, definition lines 4525-4531, Ares kind `Float`)
- `max_resonance_avoidance_speed` (`coFloat`, default `120`, field at `PrintConfig.hpp:1279`, definition lines 4533-4539, Ares kind `Float`)
- `input_shaping_emit` (`coBool`, default `false`, field at `PrintConfig.hpp:1282`, definition lines 4541-4546, Ares kind `Bool`)
- `input_shaping_type` (`coEnum`, default `Default`, enum declaration lines 365-379, enum map lines 503-518, field at `PrintConfig.hpp:1283`, definition lines 4548-4555, Ares kind `Enum`)
- `input_shaping_freq_x` (`coFloat`, default `0`, field at `PrintConfig.hpp:1284`, definition lines 4557-4564, Ares kind `Float`)
- `input_shaping_freq_y` (`coFloat`, default `0`, field at `PrintConfig.hpp:1285`, definition lines 4566-4573, Ares kind `Float`)
- `input_shaping_damp_x` (`coFloat`, default `0.1`, field at `PrintConfig.hpp:1286`, definition lines 4575-4581, Ares kind `Float`)
- `input_shaping_damp_y` (`coFloat`, default `0.1`, field at `PrintConfig.hpp:1287`, definition lines 4583-4589, Ares kind `Float`)

## Functional requirements

1. Add the included missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, resonance avoidance behavior, input-shaping emission behavior, firmware command generation, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter following fan/layer-height/extrusion-rate/nozzle options from `PrintConfig.cpp:4591+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all nine new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all nine new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:4591+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
