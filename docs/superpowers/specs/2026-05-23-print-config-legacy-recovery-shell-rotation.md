# M174 Spec: PrintConfig legacy recovery shell rotation slice

## Goal
Port the `enable_power_loss_recovery`, `ensure_vertical_shell_thickness`, and `rotate_solid_infill_direction` legacy normalization branches from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7971-7991`: migrate legacy power-loss recovery values, ensure-vertical-shell-thickness values, and rotate-solid-infill-direction key/value.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7992+`: infill-anchor aliases, chamber/thumbnail aliases, top-one-wall migration, initial-layer-flow alias, ironing aliases/value migration, counterbore spelling fix, draft-shield migration, pattern migrations, filament map/type migrations, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M174 branches.
- `crates/ares-core/src/options/tests/legacy_recovery_shell_rotation.rs`: add focused M174 tests proving all covered value migrations, key rename, non-matching preservation, non-string preservation, and unknown-key preservation without growing the existing legacy test module past the 400 LOC limit.
- `docs/roadmap.md` and `docs/milestones/m174-print-config-legacy-recovery-shell-rotation.md`: milestone sequencing docs.

## Included legacy rewrites

`enable_power_loss_recovery` (`PrintConfig.cpp:7971-7977`):

- string `1` -> `enable`
- string `true` -> `enable` case-insensitively
- string `0` -> `disable`
- string `false` -> `disable` case-insensitively
- other values remain unchanged

`ensure_vertical_shell_thickness` (`PrintConfig.cpp:7978-7984`):

- string `1` -> `ensure_all`
- string `0` -> `ensure_moderate`
- other values remain unchanged

`rotate_solid_infill_direction` (`PrintConfig.cpp:7985-7991`):

- key becomes `solid_infill_rotate_template`
- string `1` -> `0,90`
- string `0` -> `0`
- other values remain unchanged under the renamed key

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Preserve non-string values for covered keys unchanged except that `rotate_solid_infill_direction` still renames the key, matching the upstream branch's key assignment.
3. Preserve non-legacy unknown options exactly as today.
4. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are stored under modern keys/values according to the source-cited branch.
5. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
6. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:7992+` in this milestone.

## Acceptance checks

- Tests prove all covered `enable_power_loss_recovery` values migrate as specified, including case-insensitive boolean words.
- Tests prove unmatched and non-string `enable_power_loss_recovery` values remain preserved.
- Tests prove `ensure_vertical_shell_thickness` `1`/`0` values migrate and unmatched/non-string values remain preserved.
- Tests prove `rotate_solid_infill_direction` always deserializes under `solid_infill_rotate_template`, with `1`/`0` value migrations and unmatched/non-string values preserved under the modern key.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:7992+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
