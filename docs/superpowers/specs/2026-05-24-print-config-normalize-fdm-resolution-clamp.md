# M188 Spec: PrintConfig normalize_fdm resolution clamp

## Goal
Port OrcaSlicer's optional G-code `resolution` lower-bound clamp from `DynamicPrintConfig::normalize_fdm(int used_filaments)` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8372-8374`: optional `resolution` lookup and lower-bound clamp to `0.001`.

Context anchors:

- Existing Ares option metadata for `resolution` remains the source-cited option registry boundary already ported in earlier milestones.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8376+`: prime tower, independent support layer height, filament-count behavior, and all later `normalize_fdm` branches.
- Object arrangement, variant expansion, silent-mode behavior, typed option accessors beyond this API, option parsing changes outside this API, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, UI runtime behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/fdm_normalization.rs`: extend `SliceOptions::normalize_fdm(&mut self, used_filaments: usize) -> Result<(), SliceError>` with the resolution clamp and private numeric helper if needed.
- `crates/ares-core/src/options/tests/fdm_normalization.rs`: extend source-behavior tests.
- `docs/roadmap.md` and `docs/milestones/m188-print-config-normalize-fdm-resolution-clamp.md`: milestone sequencing docs.

## Functional requirements

1. Keep M186 and M187 `normalize_fdm` behavior unchanged.
2. If `resolution` is absent, do not insert it.
3. If `resolution` is present and less than `0.001`, set it to numeric JSON `0.001`.
4. If `resolution` is present and equal to or greater than `0.001`, preserve its numeric value.
5. Accept `resolution` as a JSON number or numeric string at this public input boundary.
6. Reject non-finite, negative, non-numeric, or structurally invalid `resolution` values with `SliceError::InvalidInput`.
7. Do not add automatic deserialization normalization; callers must explicitly invoke `normalize_fdm`.
8. Do not add `PrintConfig.cpp:8376+` behavior, slicing, extrusion, G-code behavior, UI runtime behavior, new crates, or dependencies.
9. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove absent `resolution` remains absent after `normalize_fdm`.
- Tests prove low numeric and numeric-string `resolution` values clamp to `0.001`.
- Tests prove `resolution == 0.001` and larger values are preserved.
- Tests prove invalid `resolution` values return `SliceError::InvalidInput` and do not panic.
- Tests prove M186/M187 behavior still happens when `resolution` is present.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8376+` runtime normalization branches.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
