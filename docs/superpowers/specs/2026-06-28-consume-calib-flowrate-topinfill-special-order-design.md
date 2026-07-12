# Consume Calib Flowrate Top Infill Special Order Design

## Goal

Consume OrcaSlicer's `calib_flowrate_topinfill_special_order` flag in Ares' existing top-surface infill path generation so the option changes concrete top solid infill path direction and emitted G-code coordinates instead of remaining registry metadata only.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1070` declares `calib_flowrate_topinfill_special_order` as an internal `ConfigOptionBool` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4156-4159` defines the option with default `false` and `comDevelop` mode.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:166-183` detects the flag only for `erTopSolidInfill`, marks the extrusion collection as unsorted, and reverses generated entities for calibration top solid infill.
- `OrcaSlicer/src/libslic3r/Fill/FillPlanePath.cpp:133-155` has an additional Archimedean-chords special-order branch for calibration top solid infill.

## Current Ares Boundary

- `crates/ares-core/src/options/registry/definitions/table/early.rs` already registers the option as bool metadata with default `false`.
- `crates/ares-core/src/options/infill/parse.rs` builds `InfillOptions` for all infill generation but currently does not parse this flag.
- `crates/ares-core/src/infills.rs` currently sorts scanline candidates and emits top-surface paths using the same ordering behavior as other solid infill roles.
- `crates/ares-core/src/infills/rotation.rs` decides scanline normalization and alternating behavior from the selected infill pattern.

## Included Behavior

- Add a private `InfillOptions` boolean carrying `calib_flowrate_topinfill_special_order`, parsed with Orca-compatible default `false`.
- Apply the flag only when the resolved `InfillLayerRole` is `TopSurface`.
- For Ares' current scanline top-surface compatibility shell, consume the flag by forcing generated top-surface scanline segments to reverse direction after candidate sorting. This mirrors the upstream `FillBase.cpp` calibration branch that calls `set_reverse()` on top solid infill entities.
- Preserve existing path sorting so candidate ordering stays deterministic; only the per-segment direction changes in this slice.
- Preserve all existing behavior for sparse infill, bottom surface, internal solid infill, internal bridge overrides, concentric infill, support paths, and non-top roles when the flag is set.
- Preserve existing top-surface behavior when the flag is omitted or `false`.
- Invalid non-boolean values for `calib_flowrate_topinfill_special_order` must return `SliceError::InvalidInput` through existing option parsing.
- Keep `ares-core` platform-neutral and WASM-compatible; add no dependencies and no file, terminal, UI, OpenGL, or native viewer behavior.

## Deferred Behavior

- Full Orca object/region ownership for `PrintObjectConfig` beyond the existing Ares single-options shell.
- Exact Orca `ExtrusionEntityCollection::no_sort` scheduling semantics beyond deterministic Ares scanline order.
- `FillArchimedeanChords` center-spiral special ordering from `FillPlanePath.cpp`; Ares currently rejects `archimedeanchords` surface patterns.
- Full Orca polygon clipping/chaining, expolygon behavior, Arachne, variable-width paths, and binary E2E geometry parity.
- Any new top-surface pattern, new crate, public option API, UI behavior, or self-designed Ares pipeline feature.

## Acceptance Criteria

- `InfillOptions` stores the parsed `calib_flowrate_topinfill_special_order` flag with default `false`.
- A focused option test proves the omitted/default flag is `false`, `true` parses as `true`, and non-boolean values are rejected.
- A focused infill geometry test proves enabling the flag reverses top-surface scanline segment direction on a top shell.
- A focused infill geometry test proves the same flag does not reverse bottom-surface or internal-solid scanline directions.
- A focused G-code test proves enabling the flag reaches concrete output by changing a top solid infill `;INFILL:solid:` marker and the matching `;PRINT_PATH:top_solid_infill:` marker coordinates.
- Existing top/bottom solid-surface pattern tests still pass.
- RED/GREEN evidence uses `cargo nextest run`, not `cargo test`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and a touched Rust LOC guard.

## Docs Impact

- Update `docs/roadmap.md` after implementation review with a runtime slice entry stating that `calib_flowrate_topinfill_special_order` now affects Ares top-surface scanline direction and downstream G-code while full Orca no-sort/chaining/Archimedean parity remains deferred.
- No architecture ADR is required because this preserves the current `ares-core` option-to-infill boundary and adds no new architectural invariant.
