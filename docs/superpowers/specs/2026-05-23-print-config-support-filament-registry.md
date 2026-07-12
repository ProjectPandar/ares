# M118 Spec: PrintConfig support filament registry slice

## Goal
Port the support/raft base filament option definition from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:959`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6027-6034`: `support_filament` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/minimums/modes beyond the current registry metadata boundary.
- Support/raft filament routing, support material selection, and raft/support generation behavior.
- Typed accessors or behavior changes for the newly registered key.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6036+`: `support_interface_not_for_body`, `support_line_width`, and following support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal.rs`: add `support_filament` in sorted order while keeping the file below 400 LOC.
- Registry key, metadata, fixture-count, and public lookup tests cover the definition.
- `docs/roadmap.md` and `docs/milestones/m118-print-config-support-filament-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_filament` (`coInt`, default `0`, field at `PrintConfig.hpp:959`, definition lines 6027-6034, Ares kind `Int`)

## Functional requirements

1. Add the missing option using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, support/raft filament routing behavior, support material selection behavior, slicing behavior, extrusion behavior, or G-code behavior for this option in this milestone.
6. Do not add `support_interface_not_for_body`, `support_line_width`, or following options from `PrintConfig.cpp:6036+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the covered key has expected kind, default value, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists for the covered definition.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6036+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
