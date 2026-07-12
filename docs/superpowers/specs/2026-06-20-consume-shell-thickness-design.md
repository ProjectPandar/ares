# Consume Shell Thickness Design

## Goal

Consume OrcaSlicer's `bottom_shell_thickness` and `top_shell_thickness` options in Ares' concrete solid shell classification so existing option metadata changes generated dense infill roles, print paths, and G-code artifacts before more option metadata is added.

## Upstream Boundary

Line numbers below are pinned to the local OrcaSlicer checkout commit `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1079-1080` declares `bottom_shell_layers` and `bottom_shell_thickness` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1119-1139` defines `bottom_shell_layers` default `3` and `bottom_shell_thickness` default `0.0`, with the tooltip stating thickness increases bottom solid layers when the layer-count thickness is thinner.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1167-1168` declares `top_shell_layers` and `top_shell_thickness` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6564-6584` defines `top_shell_layers` default `4` and `top_shell_thickness` default `0.6`, with the same thickness-increases-layer-count behavior.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3731-3766` propagates top and bottom solid surfaces across neighboring layers while either the configured shell layer count or configured shell thickness condition is still satisfied.

## Current Ares Boundary

- `crates/ares-core/src/options/shell_layers.rs` parses only `bottom_shell_layers` and `top_shell_layers` into `ShellLayerOptions`.
- `crates/ares-core/src/print_paths.rs` uses `ShellLayerOptions` to classify dense infill print paths as `BottomSurface`, `SolidInfill`, or `TopSolidInfill`.
- `crates/ares-core/src/options/infill/layer_role.rs` uses separate copied layer counts inside `InfillOptions` to choose bottom, internal-solid, and top surface pattern roles.
- `crates/ares-core/src/pipeline.rs` and `crates/ares-core/src/pipeline/test_support.rs` already have the `Layer` vector available at both infill and print-path generation boundaries.

## Design

Add shell-thickness parsing to `ShellLayerOptions` and compute effective shell membership from the actual planned layer stack. The effective shell behavior remains local and deterministic:

- `bottom_shell_layers` remains the minimum count of bottom solid layers.
- `bottom_shell_layers = 0` disables bottom shell membership even when `bottom_shell_thickness > 0.0`, matching Orca's `num_solid_layers == 0` propagation guard.
- `bottom_shell_thickness = 0.0` disables thickness expansion, matching Orca's option definition.
- `bottom_shell_thickness > 0.0` expands bottom shell membership upward using the Orca propagation window: a candidate upper layer belongs to the bottom shell when its `bottom_z` minus the bottom source layer's `bottom_z` is strictly less than `bottom_shell_thickness - EPSILON`, without shrinking below `bottom_shell_layers`. In Ares, `bottom_z` is derived from `Layer::print_z() - Layer::height()` because the current `Layer` struct stores print Z and height.
- `top_shell_layers` remains the minimum count of top solid layers.
- `top_shell_layers = 0` disables top shell membership even when `top_shell_thickness > 0.0`, matching Orca's `num_solid_layers == 0` propagation guard.
- `top_shell_thickness = 0.0` disables thickness expansion.
- `top_shell_thickness > 0.0` expands top shell membership downward using the Orca propagation window: a candidate lower layer belongs to the top shell when the top source layer's `print_z` minus the candidate layer's `print_z` is strictly less than `top_shell_thickness - EPSILON`, without shrinking below `top_shell_layers`.
- When bottom and top effective memberships overlap, bottom shell continues to win for earlier layers because existing Ares role classification already checks bottom before top. Preserve that precedence.
- Use a Rust-local `const SHELL_THICKNESS_EPSILON_MM: f64 = 1e-6` in the `ShellLayerOptions` implementation. This matches the magnitude already used by Ares layer planning and makes exact-thickness behavior testable.

The implementation should centralize the effective-membership logic in `ShellLayerOptions` so `InfillOptions` and `print_paths` use the same decision. Add methods with this visibility:

- Preserve existing public `ShellLayerOptions` compatibility: keep `Clone`, `Copy`, `Debug`, `Eq`, and `PartialEq`; keep `ShellLayerOptions::new(bottom_layers, top_layers)` as the public count-only constructor with both thicknesses set to `0.0`; keep `ShellLayerOptions::default()` equivalent to `ShellLayerOptions::new(3, 4)`. The Orca runtime defaults `bottom_shell_thickness = 0.0` and `top_shell_thickness = 0.6` must come from `SliceOptions::shell_layer_options()`, not from changing the public count-only default constructor path.
- `ShellLayerOptions::with_thicknesses(bottom_layers, bottom_thickness_mm, top_layers, top_thickness_mm) -> Self` for parser/tests. This may be crate-visible. Its implementation must preserve `Eq` compatibility by using a comparable internal representation or manual equality, not by dropping the existing `Eq` trait.
- Public additive accessors `ShellLayerOptions::bottom_shell_thickness_mm(self) -> f64` and `top_shell_thickness_mm(self) -> f64` for validation tests and downstream inspection.
- Crate-visible implementation helpers `ShellLayerOptions::is_bottom_shell(self, layers: &[Layer], layer_index: usize) -> bool`, `is_top_shell(self, layers: &[Layer], layer_index: usize) -> bool`, and `solid_role(self, layers: &[Layer], layer_index: usize, unsupported_bridge: bool) -> PrintPathRole` with bottom-before-top precedence. These helpers must not be public API additions.

`InfillOptions` should keep a `ShellLayerOptions` value rather than copied raw layer counts. Existing public convenience helpers such as `InfillOptions::effective_pattern(layer_index, layer_count)` must keep their signatures for API stability and remain count-only helpers. Runtime infill generation must not use those count-only helpers for shell classification; add a crate-visible layer-stack-aware method `InfillOptions::layer_role_for_layers(&self, layers: &[Layer], layer_index: usize) -> InfillLayerRole`. Do not add a new public pattern helper for this slice.

Do not change the public `generate_print_paths(...)` function signature. Add an optional `print_layers: Option<&[Layer]>` field and a builder `PrintPathInput::with_print_layers(&[Layer]) -> Self`. When print layers are provided, `generate_print_paths` must validate their length, `Layer::id()`, `Layer::print_z()`, finite `print_z`, finite `height`, and positive `height` against the existing skirt/brim/perimeter/gap-fill/infill metadata before using them. The public function should use layer-stack-aware shell classification only when `PrintPathInput` carries validated print layers; existing external callers that omit print layers retain the current count-only classification. `run_slicing_pipeline` and `pipeline::test_support` must pass `.with_print_layers(&layers)` so real slicing consumes shell thickness.

Sparse infill behavior should follow the existing Ares shell surface handling: when `0.0 < sparse_infill_density < 100.0`, effective shell membership from layer counts and shell thickness selects bottom/top surface roles; when `sparse_infill_density == 0.0`, infill generation remains empty and shell thickness does not create infill paths.

LOC plan:

- `crates/ares-core/src/print_paths/tests.rs` is already 400 LOC; split shared fixtures into `crates/ares-core/src/print_paths/tests/support.rs` before adding shell-thickness tests under a new child test module.
- `crates/ares-core/src/pipeline/test_support.rs` is already 400 LOC; when threading `.with_print_layers(&layers)` through the existing builder chain, also shrink the nearby `extrusion_per_mm(PrintPathRole::Skirt, options.initial_layer_height().unwrap())` expression so the file stays at or below 400 LOC.
- `crates/ares-core/src/options/infill.rs` is 390 LOC; replace the two copied shell-count fields with one `ShellLayerOptions` field instead of adding net-new fields.
- `crates/ares-core/src/infills.rs` is 388 LOC; keep edits to call-site replacements and avoid new helper blocks there.

## Validation And Errors

- Parse `bottom_shell_thickness` and `top_shell_thickness` as finite non-negative millimeters.
- Parsed `SliceOptions::shell_layer_options()` defaults remain Orca-compatible: `bottom_shell_thickness = 0.0`, `top_shell_thickness = 0.6`.
- Invalid negative, non-finite, or non-numeric values return `SliceError::InvalidInput` mentioning the offending key.
- Existing layer-count validation remains unchanged.

## Non-Goals

- Do not implement Orca's full `discover_vertical_shells()` projection algorithm, `ensure_vertical_shell_thickness`, `extra_solid_infills`, or top/bottom region polygon scattering.
- Do not implement Orca's exact per-surface source propagation. This slice applies the same layer-count and thickness window semantics at Ares' existing whole-layer role-classification boundary.
- Do not implement the adjacent spiral-mode lower-shell perimeter condition from `OrcaSlicer/src/libslic3r/LayerRegion.cpp:83-84`; this slice is limited to infill and print-path role classification.
- Do not change sparse infill generation when `sparse_infill_density` is below 100 except for already-existing shell surface handling.
- Do not add new option metadata or dependencies.
- Do not make breaking public API changes. The allowed public API additions are the additive `PrintPathInput::with_print_layers(&[Layer])` builder needed to let real slicing pass planned layer metadata without changing `generate_print_paths(...)`, plus read-only shell-thickness accessors on the already-public `ShellLayerOptions`.

## Docs Impact

- Update `docs/roadmap.md` only if it currently lists this option as unconsumed or milestone-pending.
- Do not add an architecture decision record for this narrow slice: it follows the existing `PrintConfig.hpp` rewrite gate and does not introduce a new architecture boundary.
- Do not regenerate option metadata docs because this slice consumes existing parsed metadata and adds runtime behavior only.

## Acceptance Criteria

1. Option tests prove defaults include `bottom_shell_thickness = 0.0` and `top_shell_thickness = 0.6`, explicit finite values parse, and invalid values are rejected with key-specific errors.
2. Unit tests prove effective shell role decisions expand bottom and top memberships from thickness while preserving layer-count minima, zero-layer disables for both sides, exact-thickness strictness, just-below and just-above threshold behavior around `SHELL_THICKNESS_EPSILON_MM`, variable-height behavior, and bottom-before-top precedence on overlap.
3. Pipeline/G-code tests prove `bottom_shell_thickness` can turn an otherwise internal dense layer into `bottom_surface` print paths and `;PRINT_PATH:bottom_surface:` G-code.
4. Pipeline/G-code tests prove `top_shell_thickness` can turn an otherwise internal dense layer into `top_solid_infill` print paths and `;PRINT_PATH:top_solid_infill:` G-code.
5. Sparse infill tests prove `0.0 < sparse_infill_density < 100.0` uses shell-thickness-expanded bottom/top surface roles, while `sparse_infill_density == 0.0` still emits no infill paths.
6. `generate_print_paths` tests prove provided `print_layers` reject length mismatch, layer metadata mismatch, non-finite `print_z`, non-finite `height`, and non-positive `height`, while calls that omit `PrintPathInput::with_print_layers` keep count-only role behavior.
7. Public compatibility tests prove `ShellLayerOptions::new(3, 4) == ShellLayerOptions::default()`, `ShellLayerOptions` still satisfies `Eq`, and parsed `SliceOptions::shell_layer_options()` still returns Orca thickness defaults `0.0` / `0.6`.
8. Existing external-style `generate_print_paths` tests that omit `PrintPathInput::with_print_layers` keep count-only role behavior, proving the public function signature and old call path remain stable.
9. Existing solid surface pattern, density, bridge, speed, extrusion, and custom G-code tests continue to pass under `cargo nextest run`.

## Verification Commands

- `cargo nextest run -p ares-core shell_thickness`
- `cargo nextest run -p ares-core top_bottom_solid_surface solid_surface_patterns`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- `wc -l <each touched Rust file>` and split or shrink any touched Rust file over 400 LOC before commit.
