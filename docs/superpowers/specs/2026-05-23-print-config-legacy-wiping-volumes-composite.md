# M182 Spec: PrintConfig legacy wiping-volumes matrix composite

## Goal
Port the wiping-volumes matrix composite conversion from `libslic3r::PrintConfigDef::handle_legacy_composite` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8132-8150`: when `wiping_volumes_matrix` exists and `wiping_volumes_use_custom_matrix` does not, infer whether the matrix is custom by comparing off-diagonal values with the pre-2.7.3 default value `140`.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8093-8096`: final `print_config_def.has(opt_key)` unknown-key validation. Ares keeps unknown non-obsolete keys until the option registry is complete enough to validate without dropping unported Orca options.
- Any code after `PrintConfig.cpp:8150`.
- Typed accessors, registry metadata, runtime purge-volume behavior, prime tower behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the composite legacy pass to call the M182 wiping-volumes helper while keeping `legacy.rs` below 400 LOC.
- `crates/ares-core/src/options/legacy/wiping_volumes.rs`: add a focused private helper for M182 matrix inference.
- `crates/ares-core/src/options/tests/legacy_wiping_volumes_composite.rs`: add focused tests for default/custom inference, existing-flag preservation, numeric input forms, invalid input rejection, and unknown-key preservation.
- `crates/ares-core/src/options/tests.rs`: register the M182 test module.
- `docs/roadmap.md` and `docs/milestones/m182-print-config-legacy-wiping-volumes-composite.md`: milestone sequencing docs.

## Included behavior

When `SliceOptions` is deserialized:

1. If `wiping_volumes_matrix` is absent, do nothing.
2. If `wiping_volumes_use_custom_matrix` is already present, preserve it unchanged and do not infer a new value.
3. Parse `wiping_volumes_matrix` as a numeric vector using Ares' existing option numeric-vector parsing behavior.
4. Compute `num_of_extruders` as `round(sqrt(matrix.len()))`, matching `int(std::sqrt(matrix.size()) + 0.5)`.
5. Iterate values in row-major order. Ignore diagonal positions where row index equals column index.
6. If any off-diagonal value is not approximately `140` using Orca `EPSILON = 1e-4`, insert `wiping_volumes_use_custom_matrix: true`.
7. Otherwise insert `wiping_volumes_use_custom_matrix: false`.
8. Reject invalid or empty matrix values during deserialization because the JSON deserializer is Ares' external boundary.

## Functional requirements

1. Apply M182 after the existing M169-M180 single-option normalization and after M181 thumbnail composite normalization.
2. Preserve `wiping_volumes_matrix` exactly as provided; only add `wiping_volumes_use_custom_matrix` when it was absent.
3. Preserve non-obsolete unknown options exactly as today.
4. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except covered wiping-volume composite insertion/rejection.
5. Keep all modified Rust files below 400 LOC.
6. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, purge-volume runtime behavior, prime tower behavior, or G-code behavior.
7. Do not implement final unknown-key validation from `PrintConfig.cpp:8093-8096` in this milestone.

## Acceptance checks

- Tests prove `[0, 140, 140, 0]` inserts `wiping_volumes_use_custom_matrix: false`.
- Tests prove an off-diagonal non-default value inserts `wiping_volumes_use_custom_matrix: true`, while a value within Orca `EPSILON = 1e-4` of `140` is treated as default.
- Tests prove diagonal non-default values still insert `false` when off-diagonal values are default, matching the upstream off-diagonal-only check.
- Tests prove an existing `wiping_volumes_use_custom_matrix` value is preserved unchanged.
- Tests prove string and scalar numeric forms are accepted through the existing numeric parser.
- Tests prove invalid and empty matrix values reject deserialization.
- Tests prove non-obsolete unknown keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8093-8096` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
