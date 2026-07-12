# M145 Spec: PrintConfig thumbnails registry slice

## Goal
Port the adjacent G-code thumbnail option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1616`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6956-6961`: `thumbnails` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:397-399`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:542-549`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6963-6978`: `thumbnails_format` enum map and option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/mode/gui metadata beyond the current registry metadata boundary.
- Thumbnail string validation/normalization in `PrintConfig.cpp:8104-8128`.
- Thumbnail image generation, thumbnail encoding, and G-code embedding behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6980+`: `use_relative_e_distances`, `wall_generator`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `thumbnails` and `thumbnails_format` after `thick_internal_bridges` and before `time_cost`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add `thumbnails` and `thumbnails_format` after `thick_internal_bridges` and before `time_cost`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/thumbnails.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_thumbnails.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for both covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by two.
- `docs/roadmap.md` and `docs/milestones/m145-print-config-thumbnails-registry.md`: milestone sequencing docs.

## Included option definitions

- `thumbnails` (`coString`, default `48x48/PNG,300x300/PNG`, field at `PrintConfig.hpp:1616`, definition lines 6956-6961, Ares kind `String`)
- `thumbnails_format` (`coEnum`, default `PNG`, enum at `PrintConfig.hpp:397-399`, enum map lines 542-549, definition lines 6963-6978, Ares kind `Enum`)

## Functional requirements

1. Add the two missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, thumbnail validation/normalization, thumbnail generation/encoding, G-code thumbnail embedding, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `use_relative_e_distances`, `wall_generator`, or following options from `PrintConfig.cpp:6980+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, thumbnail validation/generation/G-code embedding behavior, and following `PrintConfig.cpp:6980+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
