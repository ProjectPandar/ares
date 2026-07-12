# M189 Spec: PrintConfig normalize_fdm prime tower normalization

## Goal
Port OrcaSlicer's prime-tower branch from `DynamicPrintConfig::normalize_fdm(int used_filaments)` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8376-8401`: optional `enable_prime_tower` lookup; `used_filaments > 0` guard; `independent_support_layer_height` creation lookup; `print_sequence` and `timelapse_type` enum reads; non-smooth single-filament/by-object prime-tower disable; independent-support-height disable when prime tower remains enabled.

Context anchors:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:148-153` and `PrintConfig.cpp:293-297`: `PrintSequence` enum keys, including `"by layer"` and `"by object"`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:281-284` and `PrintConfig.cpp:431-435`: `TimelapseType` enum keys, where `"1"` is smooth timelapse.
- Existing Ares option registry metadata for `enable_prime_tower`, `independent_support_layer_height`, `print_sequence`, and `timelapse_type` remains the source-cited option-definition boundary already ported in earlier milestones.

Related upstream behavior explicitly deferred:

- Commented-out `adaptive_layer_height` handling in `PrintConfig.cpp:8380` and `PrintConfig.cpp:8394-8395`.
- Commented-out independent-support-height re-enable branch in `PrintConfig.cpp:8397-8400`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8403+`: later split `normalize_fdm_1`, duplicate normalization branches, and subsequent behavior.
- Object arrangement, variant expansion, silent-mode behavior, typed option accessors beyond this API, option parsing changes outside this API, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, UI runtime behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/fdm_normalization.rs`: extend `SliceOptions::normalize_fdm(&mut self, used_filaments: usize) -> Result<(), SliceError>` with the prime-tower branch and private string helper if needed.
- `crates/ares-core/src/options/tests/fdm_normalization_prime_tower.rs`: add source-behavior tests without growing the existing FDM normalization test file past 400 LOC.
- `crates/ares-core/src/options/tests.rs`: register the new test module.
- `docs/roadmap.md` and `docs/milestones/m189-print-config-normalize-fdm-prime-tower.md`: milestone sequencing docs.

## Functional requirements

1. Keep M186, M187, and M188 `normalize_fdm` behavior unchanged.
2. If `enable_prime_tower` is absent, do not insert it or `independent_support_layer_height`.
3. If `used_filaments == 0`, do not apply the prime-tower branch even when `enable_prime_tower` is present.
4. If `used_filaments > 0`, `enable_prime_tower` is present, timelapse is not smooth, and `used_filaments == 1`, set `enable_prime_tower` to `false`.
5. If `used_filaments > 0`, `enable_prime_tower` is present, timelapse is not smooth, and `print_sequence == "by object"`, set `enable_prime_tower` to `false`.
6. If `timelapse_type == "1"`, treat it as smooth timelapse and do not disable `enable_prime_tower` because of single-filament or by-object conditions.
7. Entering the branch creates `independent_support_layer_height` with Orca's default `true` when absent; if `enable_prime_tower` remains `true` after the disable check, set it to `false`.
8. Reject structurally invalid branch inputs with `SliceError::InvalidInput` rather than panicking: non-boolean `enable_prime_tower`, non-string/unsupported `print_sequence` when read, and non-string/unsupported `timelapse_type` when present.
9. Do not add automatic deserialization normalization; callers must explicitly invoke `normalize_fdm`.
10. Do not add commented-out Orca behavior, later `normalize_fdm_1` behavior, slicing, extrusion, G-code behavior, UI runtime behavior, new crates, or dependencies.
11. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove missing `enable_prime_tower` and `used_filaments == 0` do not apply prime-tower side effects.
- Tests prove non-smooth single-filament and by-object conditions disable prime tower.
- Tests prove smooth timelapse preserves prime tower and disables independent support layer height.
- Tests prove branch entry creates absent independent support layer height with default `true`, and enabled prime tower with multi-filament by-layer settings then disables it.
- Tests prove invalid prime-tower branch values return `SliceError::InvalidInput` and do not panic.
- Tests prove M186/M187/M188 behavior still happens alongside the prime-tower branch.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8403+` runtime normalization branches.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
