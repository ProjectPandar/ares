# M155 Spec: PrintConfig SLA relative correction registry slice

## Goal
Port `relative_correction` from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1837`: `SLAPrinterConfig` `relative_correction` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7312-7318`: `relative_correction` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min metadata beyond the current registry metadata boundary.
- SLA relative correction/scaling runtime behavior.
- Typed accessors or behavior changes for the newly registered key.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7320+`: axis-specific correction and later SLA settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_raft.rs`: add `relative_correction` after `reduce_infill_retraction` and before `required_nozzle_HRC`.
- `crates/ares-core/src/options/registry/tests/keys.rs`, `keys/second.rs`, and a new `keys/third.rs`: split expected key fixtures so no Rust test file reaches 400 LOC, with `relative_correction` in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_relative_correction.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_relative_correction.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add a fixture value for `relative_correction`.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 1.
- `docs/roadmap.md` and `docs/milestones/m155-print-config-sla-relative-correction-registry.md`: milestone sequencing docs.

## Included option definitions

- `relative_correction` (`coFloats`, default `{1., 1.}`, field at `PrintConfig.hpp:1837`, definition lines 7312-7318, Ares kind `Floats`, Ares default string `1`)

## Functional requirements

1. Add the missing option using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA relative correction behavior, slicing behavior, extrusion behavior, or G-code behavior for this option in this milestone.
6. Do not add `relative_correction_x` or later SLA settings from `PrintConfig.cpp:7320+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; split expected-key fixtures as needed.

## Acceptance checks

- Registry tests prove the covered key has expected kind, default value, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists for the covered definition.
- Plan/spec explicitly account for deferred SLA runtime behavior and `PrintConfig.cpp:7320+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
