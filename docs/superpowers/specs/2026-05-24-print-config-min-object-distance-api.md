# M185 Spec: PrintConfig min object distance API

## Goal
Port `libslic3r`'s `min_object_distance(const ConfigBase&)` helper into `ares-core` as a `SliceOptions` API for future UI/arrange consumers.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:602-603`: public helper declaration and arrange-distance comment.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8305-8329`: `min_object_distance(const ConfigBase &cfg)` implementation.

Context anchors:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:148-152`: `PrintSequence::{ByLayer, ByObject, ByDefault, Count}`.
- Existing Ares option metadata for `printer_technology`, `extruder_clearance_radius`, and `print_sequence` remains the source-cited option registry boundary already ported in earlier milestones.

Related upstream behavior explicitly deferred:

- Object arrangement / placement algorithms that consume this distance.
- `DynamicPrintConfig::normalize_fdm` and later runtime normalization from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8332+`.
- Variant expansion, silent-mode behavior, extruder-count expansion, filament override/following behavior, typed option accessors beyond this API, option parsing changes outside this API, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, UI runtime behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/object_distance.rs`: implement the `SliceOptions::min_object_distance()` API and private parsing helpers.
- `crates/ares-core/src/options.rs`: add only the module declaration; do not grow this near-400 LOC file with implementation logic.
- `crates/ares-core/src/options/tests/object_distance.rs`: add source-behavior tests.
- `crates/ares-core/src/options/tests.rs`: add the test module.
- `docs/roadmap.md` and `docs/milestones/m185-print-config-min-object-distance-api.md`: milestone sequencing docs.

## Functional requirements

1. Add `pub fn SliceOptions::min_object_distance(&self) -> Result<f64, SliceError>`.
2. If `printer_technology` is present and equals `SLA`, return `6.0`.
3. If `printer_technology` is absent or not `SLA`, follow the FFF/unknown branch.
4. In the FFF/unknown branch, return `0.0` when `extruder_clearance_radius` is absent or `print_sequence` is absent.
5. In the FFF/unknown branch, when both options are present and `print_sequence` equals `by object`, return `extruder_clearance_radius` only if it is greater than `6.0`; otherwise return `6.0`.
6. In the FFF/unknown branch, when both options are present and `print_sequence` is any non-`by object` supported value, return `6.0`.
7. Accept `extruder_clearance_radius` as a JSON number or numeric string because `SliceOptions` is a user/input boundary in Ares.
8. Reject non-finite, negative, non-numeric, or structurally invalid `extruder_clearance_radius` values with `SliceError::InvalidInput`.
9. Reject non-string `printer_technology` and non-string `print_sequence` values with `SliceError::InvalidInput`.
10. Do not add object arrangement, placement, UI runtime behavior, slicing, extrusion, G-code behavior, new crates, or dependencies.
11. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove SLA returns `6.0` without requiring clearance or print sequence.
- Tests prove missing `extruder_clearance_radius` or missing `print_sequence` returns `0.0` in the FFF/unknown branch.
- Tests prove by-object sequence returns `max(6.0, extruder_clearance_radius)`.
- Tests prove by-layer and by-default sequences return `6.0` when both options are present.
- Tests prove invalid boundary values return `SliceError::InvalidInput` and do not panic.
- `options.rs` remains below 400 LOC and contains only the module declaration for this implementation.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8332+` runtime normalization and arrange algorithms.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
