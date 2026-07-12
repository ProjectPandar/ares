# M191 Spec: PrintConfig handle_legacy_sla correction expansion

## Goal
Port OrcaSlicer's `handle_legacy_sla(DynamicPrintConfig&)` correction-vector expansion into Ares `SliceOptions` ingestion so legacy SLA correction vectors populate the scalar correction keys used by newer configs.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8507-8527`: `handle_legacy_sla` loop over `relative_correction` and `material_correction`, conditional scalar-key creation, and source vector indexing.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:693`: public `handle_legacy_sla(DynamicPrintConfig&)` declaration.
- Call-site context: `OrcaSlicer/src/libslic3r/Preset.cpp:486` and `Model.cpp:456`, where loaded configs receive the legacy SLA normalization.

Context anchors:

- `PrintConfig.hpp:1837-1840` and `PrintConfig.cpp:7312-7342`: `relative_correction`, `relative_correction_x`, `relative_correction_y`, and `relative_correction_z` option declarations/defaults.
- `PrintConfig.hpp:1817-1820` and `PrintConfig.cpp:7479-7505`: `material_correction`, `material_correction_x`, `material_correction_y`, and `material_correction_z` option declarations/defaults.
- Existing Ares registry metadata for these keys remains the source-cited option-definition boundary already ported in earlier milestones.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:8529+` `get_parameter_size`, extruder-variant extension, validation, preset/model loading machinery, 3MF project import behavior, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.
- Changing existing scalar values when they are already present; upstream only creates missing scalar keys.

Rust destination boundary:

- `crates/ares-core/src/options/legacy/sla.rs`: add the correction-vector composite normalization helper.
- `crates/ares-core/src/options/legacy.rs`: call the helper from existing `normalize_legacy_composite_options` during `SliceOptions` deserialization.
- `crates/ares-core/src/options/tests/legacy_sla_corrections.rs`: add source-behavior tests.
- `crates/ares-core/src/options/tests.rs`: register the new test module.
- `docs/roadmap.md` and `docs/milestones/m191-print-config-handle-legacy-sla-corrections.md`: milestone sequencing docs.

## Functional requirements

1. During `SliceOptions` deserialization, if `relative_correction` is present and any of `relative_correction_x`, `relative_correction_y`, or `relative_correction_z` is absent, create only the absent scalar key(s).
2. For `relative_correction`, created `_x` and `_y` values both come from vector index `0`, and created `_z` comes from vector index `1`, matching `PrintConfig.cpp:8511-8523` exactly.
3. During `SliceOptions` deserialization, if `material_correction` is present and any of `material_correction_x`, `material_correction_y`, or `material_correction_z` is absent, create only the absent scalar key(s).
4. For `material_correction`, created `_x` and `_y` values both come from vector index `0`, and created `_z` comes from vector index `1`, matching upstream indexing exactly even though the option default has three vector entries.
5. If a correction vector key is absent, do not create any of its scalar keys.
6. If a scalar key is already present, preserve its existing value and do not overwrite it.
7. Accept JSON arrays and numeric strings split by `;` or `,` for correction vectors. Scalar numeric correction values parse as a one-element vector: they may satisfy missing `_x`/`_y` only when `_z` already exists, but must fail if a missing `_z` requires index `1`.
8. Match upstream lazy indexing: if all scalar keys for a correction prefix already exist, preserve them and do not parse or validate the vector value. If only `_x` and/or `_y` are missing, read and require only vector index `0`; invalid later entries must not fail. If `_z` is missing, read and require vector index `1`.
9. Reject malformed, too-short-for-needed-index, or non-finite correction values with `SliceError::InvalidInput` surfaced through serde deserialization rather than panicking.
10. Preserve existing legacy key migrations, thumbnail composite normalization, wiping-volume composite normalization, and M186-M190 FDM normalization behavior.
11. Do not add `get_parameter_size`, extruder-variant extension, preset/model loading machinery, UI runtime behavior, slicing, extrusion, G-code behavior, new crates, or dependencies.
12. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove `relative_correction: [1.2, 1.3]` creates missing `relative_correction_x = 1.2`, `relative_correction_y = 1.2`, and `relative_correction_z = 1.3`.
- Tests prove `material_correction: [2.1, 2.2, 2.3]` creates missing `material_correction_x = 2.1`, `material_correction_y = 2.1`, and `material_correction_z = 2.2`.
- Tests prove existing scalar correction values are preserved while missing siblings are created, and all-existing scalar siblings cause no vector parsing/validation.
- Tests prove absent correction vector keys do not create scalar correction keys.
- Tests prove numeric string vector forms are accepted, scalar numeric correction values are accepted only when the missing scalar axes require index `0` but fail when a missing `_z` requires index `1`, and invalid/non-finite unneeded later vector entries are not parsed when only index `0` is needed.
- Tests prove malformed, empty, nonnumeric, non-finite, and too-short-for-needed-index correction values fail deserialization with `InvalidInput` text instead of panicking.
- Tests prove existing legacy composite behavior, at least thumbnail or wiping-volume normalization, still works after adding the SLA helper.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8529+` behavior and preset/model loading machinery.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
