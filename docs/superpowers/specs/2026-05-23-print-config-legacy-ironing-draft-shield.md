# M176 Spec: PrintConfig legacy ironing and draft-shield slice

## Goal
Port the `ironing_direction`, negative `ironing_angle`, `counterbole_hole_bridging`, and `draft_shield` legacy branches from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8005-8012`: migrate legacy ironing-direction alias, negative ironing-angle value, counterbore spelling alias, and draft-shield limited value.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8013+`: pattern migrations, `filament_map_mode`, `filament_type`, prime-tower rib migrations, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M176 branches.
- `crates/ares-core/src/options/tests/legacy_ironing_draft_shield.rs`: add focused M176 tests proving covered key renames, value migrations, non-matching preservation, non-string preservation, and unknown-key preservation without growing existing test modules past the 400 LOC limit.
- `crates/ares-core/src/options/tests.rs`: register the M176 test module.
- `docs/roadmap.md` and `docs/milestones/m176-print-config-legacy-ironing-draft-shield.md`: milestone sequencing docs.

## Included legacy rewrites

`ironing_direction` (`PrintConfig.cpp:8005-8006`):

- key becomes `ironing_angle`
- value is preserved unchanged, including non-string JSON values

`ironing_angle` (`PrintConfig.cpp:8007-8008`):

- string values beginning with `-` become string `0`
- non-negative strings remain unchanged
- non-string values remain unchanged because this Rust ingestion layer only applies string-prefix comparisons to JSON strings

`counterbole_hole_bridging` (`PrintConfig.cpp:8009-8010`):

- key becomes `counterbore_hole_bridging`
- value is preserved unchanged, including non-string JSON values

`draft_shield` (`PrintConfig.cpp:8011-8012`):

- string value `limited` becomes string `disabled`
- other strings remain unchanged
- non-string values remain unchanged

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Preserve values unchanged for simple aliases.
3. Preserve non-string values for simple aliases under their modern keys.
4. Apply negative `ironing_angle` migration only to string values whose first character is `-`.
5. Apply `draft_shield` migration only to exact string value `limited`.
6. Preserve non-legacy unknown options exactly as today.
7. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are stored under modern keys/values according to the source-cited branch.
8. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
9. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:8013+` in this milestone.

## Acceptance checks

- Tests prove `ironing_direction` renames to `ironing_angle` and preserves string and non-string values.
- Tests prove negative string `ironing_angle` values become `0`.
- Tests prove non-negative string and non-string `ironing_angle` values remain unchanged.
- Tests prove `counterbole_hole_bridging` renames to `counterbore_hole_bridging` and preserves string and non-string values.
- Tests prove `draft_shield: "limited"` becomes `disabled` while other strings and non-string values remain unchanged.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8013+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
