# M190 Spec: PrintConfig normalize_fdm_2 prime tower changed keys

## Goal
Port OrcaSlicer's `DynamicPrintConfig::normalize_fdm_2(int num_objects, int used_filaments)` prime-tower branch into `ares-core` as an explicit advanced normalization API that returns the option keys changed by the branch.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8452-8505`: `normalize_fdm_2` changed-key vector, optional `enable_prime_tower` lookup, `used_filaments > 0` guard, `independent_support_layer_height` creation lookup, `print_sequence`, `timelapse_type`, and `enable_wrapping_detection` reads, non-smooth/non-wrapping single-filament or multi-object by-object prime-tower disable, changed-key reporting for `enable_prime_tower`, independent-support-height disable when prime tower remains enabled, and changed-key reporting for `independent_support_layer_height`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:628-631`: public `normalize_fdm`, `normalize_fdm_1`, and changed-key-returning `normalize_fdm_2` declarations.

Context anchors:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:148-153` and `PrintConfig.cpp:293-297`: `PrintSequence` enum keys, including `"by layer"` and `"by object"`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:281-284` and `PrintConfig.cpp:431-435`: `TimelapseType` enum keys, where `"1"` is smooth timelapse.
- Existing Ares option registry metadata for `enable_prime_tower`, `independent_support_layer_height`, `print_sequence`, `timelapse_type`, and `enable_wrapping_detection` remains the source-cited option-definition boundary already ported in earlier milestones.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8404-8449` `normalize_fdm_1`, because M186-M189 already expose the combined `normalize_fdm(used_filaments)` compatibility API and this milestone only ports the advanced changed-key branch.
- Commented-out `adaptive_layer_height` handling in `PrintConfig.cpp:8458` and `PrintConfig.cpp:8483-8489`.
- Commented-out independent-support-height re-enable branch in `PrintConfig.cpp:8491-8501`.
- Automatic `Print::Apply` integration, object arrangement, variant expansion, silent-mode behavior, option parsing changes outside this API, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, UI runtime behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/fdm_normalization.rs`: add `SliceOptions::normalize_fdm_2(&mut self, num_objects: usize, used_filaments: usize) -> Result<Vec<String>, SliceError>` with the source-cited prime-tower changed-key branch, reusing existing parsing helpers where appropriate.
- `crates/ares-core/src/options/tests/fdm_normalization_prime_tower_changed_keys.rs`: add source-behavior tests without growing the existing prime-tower test file past 400 LOC.
- `crates/ares-core/src/options/tests.rs`: register the new test module.
- `docs/roadmap.md` and `docs/milestones/m190-print-config-normalize-fdm-2-prime-tower.md`: milestone sequencing docs.

## Functional requirements

1. Add a public explicit API `SliceOptions::normalize_fdm_2(num_objects, used_filaments)` returning changed option keys in upstream order.
2. If `used_filaments == 0`, do not apply the branch and return an empty changed-key list.
3. If `enable_prime_tower` is absent, do not insert `enable_prime_tower` or `independent_support_layer_height`, and return an empty changed-key list.
4. Entering the branch creates `independent_support_layer_height` with Orca's default `true` when absent; this creation is not reported as a changed key unless the branch later sets it to `false`.
5. Treat missing `print_sequence` as Orca's registry default `"by layer"`; reject unsupported present values with `SliceError::InvalidInput`.
6. Treat missing `timelapse_type` as non-smooth; treat `timelapse_type == "1"` as smooth; reject unsupported present values with `SliceError::InvalidInput`.
7. Treat missing `enable_wrapping_detection` as `false`; reject non-boolean present values with `SliceError::InvalidInput`.
8. If timelapse is not smooth, wrapping detection is disabled, and `used_filaments == 1`, disable enabled `enable_prime_tower` and return `"enable_prime_tower"`.
9. If timelapse is not smooth, wrapping detection is disabled, `print_sequence == "by object"`, and `num_objects > 1`, disable enabled `enable_prime_tower` and return `"enable_prime_tower"`.
10. If `enable_prime_tower` is already `false`, leave it false and do not return `"enable_prime_tower"`.
11. If `enable_prime_tower` remains true and `independent_support_layer_height` is true or newly defaulted true, set it to false and return `"independent_support_layer_height"`.
12. If `enable_prime_tower` remains true and `independent_support_layer_height` is already false, return no key for it.
13. Preserve M186-M189 `normalize_fdm(used_filaments)` behavior and do not make it return changed keys.
14. Do not add `normalize_fdm_1`, commented-out Orca behavior, automatic deserialization normalization, slicing, extrusion, G-code behavior, UI runtime behavior, new crates, or dependencies.
15. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove absent `enable_prime_tower` and `used_filaments == 0` return no changed keys and insert no prime-tower side effects.
- Tests prove non-smooth single-filament and multi-object by-object cases disable prime tower and return exactly `enable_prime_tower` when it was true.
- Tests prove by-object with one object does not disable prime tower and instead reports `independent_support_layer_height` if that value changes.
- Tests prove `enable_wrapping_detection = true` preserves enabled prime tower and reports only the independent-support-height change.
- Tests prove smooth timelapse preserves enabled prime tower and reports only the independent-support-height change.
- Tests prove false prime tower creates default independent support height but reports no changed keys.
- Tests prove already-false independent support height is not reported again.
- Tests prove invalid branch values return `SliceError::InvalidInput` and do not panic.
- Tests prove existing `normalize_fdm` M186-M189 behavior remains intact.
- Plan/spec explicitly account for deferred `normalize_fdm_1`, commented-out adaptive-layer-height, and automatic `Print::Apply` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
