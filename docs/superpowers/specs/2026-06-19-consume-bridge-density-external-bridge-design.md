# Consume `bridge_density` in External Bridge Spacing

## Goal

Consume OrcaSlicer `bridge_density` as concrete Ares slicing behavior for external bottom bridge infill spacing. This slice continues the option-consumption work after `bridge_angle`: generated bridge infill geometry and G-code must change when `bridge_density` changes, instead of adding more registry-only milestones.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1237-1250` registers `bridge_density` as "External bridge density", defaults it to `100%`, constrains it to `10..=120%`, and describes it as controlling external bridge line density/spacing.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:887-905` keeps solid external surfaces at solid density before bridge-specific adjustment.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:951-968` marks bridge surfaces as bridge fill and uses bridge flow spacing for bridge infill generation.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1331-1334` overrides external bridge fill density with `region.config().bridge_density.get_abs_value(1.0)` and disables solid-spacing adjustment.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2767-2769` computes rectilinear line spacing as `spacing / params.density`.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:1843` and `FillBase.cpp:2263` use the same `spacing / params.density` rule in fill connection helpers.

## Current Ares State

- `crates/ares-core/src/options/infill.rs` parses sparse/solid infill spacing inputs and `bridge_angle`, but it does not parse `bridge_density`.
- `crates/ares-core/src/infills.rs` computes solid bottom-surface spacing from `options.solid_line_width()` for all solid roles.
- The previous `bridge_angle` slice added `InfillBridgeContext` and a shared `fully_unsupported_layer` predicate so infill geometry can know when final print paths will become `PrintPathRole::Bridge`.
- `crates/ares-core/src/print_paths.rs` still owns final role conversion to `PrintPathRole::Bridge`; this slice should keep that role boundary and only adjust bridge infill spacing before print paths are built.
- Ares' current bridge flow and thick-bridge extrusion behavior remain separate in `crates/ares-core/src/bridges.rs` and `crates/ares-core/src/extrusions.rs`.

## Ares Destination Boundary

Implement the smallest source-cited runtime slice that makes `bridge_density` affect Ares' existing external bridge geometry:

- Parse `bridge_density` into `InfillOptions` as a percent value with Orca default `100.0`, minimum `10.0`, and maximum `120.0`.
- For `bridge_density == 100`, preserve current external bridge spacing and output.
- For `bridge_density != 100`, adjust scanline spacing only when the same conditions used by `bridge_angle` identify an external bottom bridge: `InfillLayerRole::BottomSurface`, `bridge_no_support = true`, and `fully_unsupported_layer(...)` is true for that layer.
- Compute bridge spacing as `solid_line_width / (bridge_density / 100.0)`, matching upstream's `spacing / density` rule.
- Keep `bridge_density` independent from `bridge_angle`: either option may be used alone, and when both are set, angle controls direction while density controls spacing.
- Do not apply `bridge_density` to supported bottom surfaces, top surfaces, internal solid infill, sparse infill, first-layer bottom surfaces, or internal bridges.
- Keep implementation inside `ares-core`; add no filesystem, terminal, UI, OpenGL, native-only API, crate, or dependency.

## Explicitly Deferred

- `internal_bridge_density` and internal bridge surface behavior.
- Full per-surface `Surface` graph ownership for bridge density.
- Bridge flow spacing parity beyond Ares' existing line-width/scalar infill spacing model.
- Automatic bridge direction detection, support generation, support contact filtering, mixed supported/unsupported contour classification, max bridge length, and bridge detector ownership.
- Flow/extrusion changes from density; this slice changes generated geometry spacing/count. Existing bridge flow and thick bridge extrusion remain separate.
- Top/bottom surface density options; those are separate solid-surface density slices.
- Registry-only metadata expansion, new crates, UI behavior, terminal behavior, and independent Ares-owned pipeline design.

## Design

Store `bridge_density_percent` on `InfillOptions` near `bridge_angle_degrees`. The value belongs in infill options because it changes scanline generation spacing, not extrusion flow or final print-path role assignment.

Generalize the `bridge_angle` bridge-context decision into a small internal `BridgeInfillOverride` value built during `generate_infills_with_bridge_context(...)`. It should be `Some` only for external bottom bridges according to the existing shared unsupported-layer predicate. `InfillPasses::new(...)` receives the optional fixed angle as today. `generate_layer_infills(...)` receives the optional bridge density and passes an effective spacing into `spacing_for_role(...)`.

Spacing selection should remain simple:

- Sparse roles use existing sparse spacing.
- Non-bridge solid roles use existing `options.solid_line_width()`.
- External bottom bridge roles with bridge context use `options.solid_line_width() / (options.bridge_density_percent() / 100.0)`.

This mirrors Orca's density adjustment while staying within Ares' current rectangular scanline fill model. Because Ares' final `PrintPathRole::Bridge` classification shares the same bridge predicate, a bridge-density geometry change and the bridge role comment in G-code remain aligned.

## Tests

Use TDD with focused RED/GREEN checks:

- Option tests:
  - default `bridge_density` parses as `100.0`;
  - numeric and numeric-string values within `10..=120` parse;
  - values below `10`, above `120`, nonnumeric strings, booleans, null, NaN, and infinity fail through `SliceOptions::infill_options()`.
- Infill unit tests:
  - without bridge context, `bridge_density = 50` preserves bottom-surface spacing;
  - with bridge context but `bridge_no_support = false`, `bridge_density = 50` preserves bottom-surface spacing;
  - with bridge context, `bridge_no_support = true`, and a fully unsupported second bottom layer, `bridge_density = 50` doubles line spacing and reduces the generated bridge lines;
  - with the same unsupported layer, `bridge_density = 120` reduces line spacing and increases generated bridge lines;
  - with `bridge_density = 100`, current bridge spacing is preserved.
- Pipeline/G-code tests:
  - `bridge_no_support = true`, `bridge_density = 50`, and aligned bottom bridge infill emit fewer second-layer `;PRINT_PATH:bridge:` lines than default density;
  - a supported repeated rectangular bottom surface with the same `bridge_density` keeps default bottom-surface spacing and role.

## Acceptance Criteria

1. `bridge_density` has at least one non-test runtime use that changes generated bridge infill geometry before G-code output.
2. `bridge_density = 100` preserves current external bridge behavior.
3. Non-default `bridge_density` affects only Ares' existing external bottom bridge path: bottom surface, fully unsupported by the shared predicate, and `bridge_no_support = true`.
4. Supported bottom surfaces, top solid infill, internal solid infill, sparse infill, and first-layer bottom surfaces keep their existing spacing behavior.
5. `bridge_angle` and `bridge_density` compose without either option disabling the other.
6. Final `PrintPathRole::Bridge` assignment remains aligned with the spacing override by sharing the unsupported-layer predicate.
7. All touched Rust source files stay at or below 400 LOC.
8. No new dependencies, crates, platform-specific behavior, or option-only milestones are introduced.

## Verification

- Targeted RED/GREEN option, infill, and pipeline/G-code tests.
- `cargo test -p ares-core --lib bridge_density`
- `cargo test -p ares-core --lib bridge_angle`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- Rust LOC guard for files under `crates/`.

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with the spec, reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

Update `docs/roadmap.md` after implementation to record that `bridge_density` now has narrow external bottom bridge spacing consumption in Ares, while `internal_bridge_density`, full per-surface bridge ownership, and full bridge detector parity remain deferred.
