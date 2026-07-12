# M153 Spec: PrintConfig extruder/filament option key list slice

## Goal
Port the `PrintConfigDef` extruder and filament option-key lists into `ares-core` as read-only option registry API data for downstream UI/API consumers.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:569-574`: `extruder_option_keys()` and `extruder_retract_keys()` accessors plus retract-key comments.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:576-593`: `filament_option_keys()` and `filament_retract_keys()` accessors plus storage members.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7196`: `PrintConfigDef::init_extruder_option_keys()` list contents and sorted retract assertion.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7198-7227`: `PrintConfigDef::init_filament_option_keys()` list contents and sorted retract assertion.

Related upstream behavior explicitly deferred:

- Runtime expansion of array options by extruder count.
- Filament override/following behavior.
- Retraction, z-hop, wipe, cut-retraction, and toolpath planning behavior.
- Typed accessors, option parsing changes, or behavior changes for the listed keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7229+`: `PrintConfigDef::init_sla_params` behavior.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/key_lists.rs`: add static key lists and public functions.
- `crates/ares-core/src/options/registry.rs`: wire and re-export the key-list functions from the registry module.
- `crates/ares-core/src/lib.rs`: re-export the key-list functions for UI/API consumers.
- `crates/ares-core/src/options/registry/tests/key_lists.rs`: add source-order, sortedness, and registry-coverage assertions.
- `crates/ares-core/src/options/registry/tests.rs`: add `mod key_lists;`.
- `crates/ares-core/src/options/tests/registry_lookup_extruder_filament_key_lists.rs`: add public API assertions.
- `crates/ares-core/src/options/tests.rs`: add the public lookup test module.
- `docs/roadmap.md` and `docs/milestones/m153-print-config-extruder-filament-key-lists.md`: milestone sequencing docs.

## Included API data

- `extruder_option_keys`: exact upstream order from `PrintConfig.cpp:7167-7173`.
- `extruder_retract_keys`: exact upstream sorted order from `PrintConfig.cpp:7176-7193`.
- `filament_option_keys`: exact upstream order from `PrintConfig.cpp:7200-7205`.
- `filament_retract_keys`: exact upstream sorted order from `PrintConfig.cpp:7208-7224`.

## Functional requirements

1. Add read-only functions returning `&'static [&'static str]` for the four lists.
2. Preserve the exact upstream order of each list.
3. Verify the two retract-key lists are sorted lexicographically.
4. Verify every exposed key resolves through `option_definition()`.
5. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
6. Do not add typed parsing/accessors, extruder-count expansion, filament override resolution, retraction/z-hop/wipe behavior, slicing behavior, extrusion behavior, or G-code behavior.
7. Do not add `PrintConfigDef::init_sla_params` behavior from `PrintConfig.cpp:7229+`.
8. Do not add new crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all four lists match expected keys in upstream order.
- Registry tests prove retract lists are sorted.
- Registry tests prove every list key resolves through `option_definition()`.
- Public API tests prove the crate-level exports return the same lists.
- Plan/spec explicitly account for deferred runtime expansion/override/retraction behavior and `PrintConfig.cpp:7229+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
