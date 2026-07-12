# Per-Object Skirt Runtime Design

## Source Boundary

This slice ports the smallest executable part of OrcaSlicer per-object skirt behavior into Ares:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:286-288`: `SkirtType` values `stCombined` and `stPerObject`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1555`: `skirt_type` option tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:437-441`: enum strings `"combined"` and `"perobject"`.
- `OrcaSlicer/src/libslic3r/Print.cpp:2593-2789`: `Print::_make_skirt`, where `stCombined` fills the global `m_skirt` and `stPerObject` fills each object's `m_skirt` from that object's convex hull.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5112-5250`: combined skirts emit from `print.skirt()`, while per-object skirts emit from each `PrintObject::object_skirt()`.

## Current Ares State

Ares already parses `skirt_type` and has `SkirtType::Combined` / `SkirtType::PerObject`. `SkirtType::Combined` emits the existing combined rectangular skirt. `SkirtType::PerObject` currently returns `SliceError::InvalidInput` because `LayerContours` has no object ownership or instance-offset model equivalent to Orca `PrintObject::object_skirt()`.

`LayerContours` does, however, already preserve multiple outer contours on a layer. In Ares' current geometry scaffold, the closest executable per-object boundary is therefore one generated skirt per outer contour on the layer. This is a source-cited compatibility shell around Orca's per-object branch, not a new Ares-owned object model.

## Design

For `SkirtType::PerObject`, `generate_skirts()` will stop rejecting the option and will generate skirt loops independently around each outer contour of each eligible skirt layer. A two-object first layer represented by two disjoint outer contours will produce two skirt paths, each expanded from its own contour bounds, instead of one global combined skirt around both contours.

The existing combined behavior remains unchanged for `SkirtType::Combined`: all contours on an eligible layer still share one global rectangular bounds skirt.

Per-object generation will reuse the existing rectangular skirt path shape, `effective_loop_count`, `skirt_height`, draft-shield layer eligibility, `min_skirt_length`, `single_loop_draft_shield`, and `skirt_start_angle` behavior. The first generated per-object path in a layer remains the only path affected by the layer's start-angle/min-length first-path policy, matching the current Ares policy for combined skirts and keeping the slice narrow.

Containment handling is intentionally limited to outer contours. Inner hole contours must not receive independent per-object skirts. This matches Orca's object convex-hull direction better than blindly iterating every contour.

The implementation must use the same odd/even containment-depth rule already used for brim outer-contour detection in `crates/ares-core/src/brims.rs`: choose a contour reference point, count containing contours, and treat even depths as outer boundaries. Do not infer outer contours from winding, because `Contour::new` normalizes winding.

Because `crates/ares-core/src/skirts/mod.rs` and `crates/ares-core/src/skirts/tests.rs` are near the 400 LOC limit, the implementation should split focused logic instead of growing those files substantially. The expected split is a new `crates/ares-core/src/skirts/per_object.rs` for per-object skirt helpers and a new focused `crates/ares-core/src/skirts/tests/per_object.rs` test module if test additions would push `tests.rs` past the limit. Keep module declarations compact and do not refactor unrelated skirt behavior.

## Docs Impact

Update `docs/roadmap.md` with a dated runtime-slice entry after implementation review approval. No public API guide, CLI help, WASM binding docs, or user-facing examples need updates because this slice only changes existing `ares-core` behavior for an already parsed Orca option.

## Deferred Behavior

Full Orca parity remains deferred:

- Real `PrintObject` ownership in the slicing artifacts.
- Object instance shifts and multiple copies.
- Per-object support-layer hulls.
- Object-specific `skirt_start_angle` from object config.
- By-object print sequence gating and `m_skirt_done` interactions.
- Brim/support-brim suppression around object skirts.
- Round offset geometry, convex hull generation, and exact `ExtrusionLoop` ownership.
- Multi-extruder skirt loop assignment.
- Orca binary E2E parity.

## Acceptance Criteria

- `skirt_type = "perobject"` no longer fails at `generate_skirts()` or in the Ares pipeline for disjoint outer contours.
- Per-object mode emits one skirt path per disjoint outer contour for one loop, while combined mode still emits one enclosing skirt path for the same input.
- The generated per-object skirt paths reach `LayerPrintPaths`, extrusion moves, speed moves, and final G-code `;SKIRT:` / `;PRINT_PATH:skirt:` output.
- Inner hole contours do not receive independent per-object skirts.
- Existing `SkirtType::Combined`, `min_skirt_length`, `single_loop_draft_shield`, `skirt_start_angle`, and draft-shield tests continue to pass.
- All Rust tests use `cargo nextest run`; no `cargo test` commands are introduced.
- Touched Rust files remain at or below 400 LOC.

## Verification

- RED: `cargo nextest run -p ares-core per_object_skirt`
- GREEN targeted: `cargo nextest run -p ares-core per_object_skirt skirt_type_gcode`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust LOC guard
