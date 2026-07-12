# M186 Spec: PrintConfig normalize_fdm extruder role propagation

## Goal
Port the first branch of `DynamicPrintConfig::normalize_fdm(int used_filaments)` into `ares-core` as the initial explicit `SliceOptions::normalize_fdm(used_filaments)` API behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8332-8353`: `DynamicPrintConfig::normalize_fdm(int used_filaments)` start, `extruder` erasure/role propagation, and `solid_infill_filament` fallback from `sparse_infill_filament`.

Related upstream behavior explicitly deferred:

- Commented-out support propagation at `PrintConfig.cpp:8342-8348`; this milestone must not set `support_filament` or `support_interface_filament` from `extruder`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8355+`: `spiral_mode`, resolution clamp, prime tower, independent support layer height, filament-count behavior, and all later `normalize_fdm` branches.
- Object arrangement, variant expansion, silent-mode behavior, typed option accessors beyond this API, option parsing changes outside this API, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, UI runtime behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/fdm_normalization.rs`: implement `SliceOptions::normalize_fdm(&mut self, used_filaments: usize) -> Result<(), SliceError>` and private integer helpers.
- `crates/ares-core/src/options.rs`: add only the module declaration; do not grow this near-400 LOC file with implementation logic.
- `crates/ares-core/src/options/tests/fdm_normalization.rs`: add source-behavior tests.
- `crates/ares-core/src/options/tests.rs`: add the test module.
- `docs/roadmap.md` and `docs/milestones/m186-print-config-normalize-fdm-extruder-roles.md`: milestone sequencing docs.

## Functional requirements

1. Add `pub fn SliceOptions::normalize_fdm(&mut self, used_filaments: usize) -> Result<(), SliceError>`.
2. `used_filaments` is accepted for API compatibility with upstream and later milestones but is not used by this M186 slice.
3. If `extruder` is present, parse it as an integer and remove the `extruder` key.
4. If parsed `extruder != 0`, set `sparse_infill_filament` to that integer only when missing.
5. If parsed `extruder != 0`, set `wall_filament` to that integer only when missing.
6. If parsed `extruder == 0`, only erase `extruder`; do not populate role filament keys from it.
7. Do not propagate `extruder` to `support_filament` or `support_interface_filament`.
8. After `extruder` handling, if `solid_infill_filament` is missing and `sparse_infill_filament` exists, parse `sparse_infill_filament` as an integer and set `solid_infill_filament` to that value.
9. Preserve existing `sparse_infill_filament`, `wall_filament`, and `solid_infill_filament` values when upstream `has(...)` checks would preserve them.
10. Accept integer JSON numbers and integer strings at this public input boundary.
11. Reject fractional, non-finite, non-numeric, negative, or structurally invalid integer values needed by this API with `SliceError::InvalidInput`.
12. Do not add automatic deserialization normalization; callers must explicitly invoke `normalize_fdm`.
13. Do not add `PrintConfig.cpp:8355+` behavior, slicing, extrusion, G-code behavior, UI runtime behavior, new crates, or dependencies.
14. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove non-zero `extruder` is erased and populates missing `sparse_infill_filament` and `wall_filament`.
- Tests prove existing `sparse_infill_filament` and `wall_filament` are not overwritten by `extruder`.
- Tests prove `extruder == 0` is erased without propagation.
- Tests prove `solid_infill_filament` is copied from existing or newly propagated `sparse_infill_filament` when missing.
- Tests prove existing `solid_infill_filament` is preserved.
- Tests prove support-filament keys are not populated from `extruder`.
- Tests prove invalid integer boundary values return `SliceError::InvalidInput` and do not panic.
- Tests prove deserializing `SliceOptions` alone does not run `normalize_fdm` automatically.
- `options.rs` remains below 400 LOC and contains only the module declaration for this implementation.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8355+` runtime normalization branches.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
