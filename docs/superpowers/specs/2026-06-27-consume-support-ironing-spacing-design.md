# Consume Support Ironing Spacing Design

## Objective

Consume OrcaSlicer's `support_ironing_spacing` option into concrete Ares support-interface ironing path geometry. This slice must produce actual generated support ironing line coordinates, not new option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:997-1000` declares `support_ironing`, `support_ironing_pattern`, `support_ironing_flow`, and `support_ironing_spacing` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406-6446` defines support ironing option metadata. `support_ironing_spacing` is a millimeter float with default `0.1`, minimum `0`, and maximum `1`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:58-61` copies `object_config.support_ironing_spacing` into support generation parameters beside support ironing flow and pattern.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1877-1907` creates support contact-layer ironing fill paths, assigns `f->spacing = support_params.ironing_spacing`, and emits them as `erIroning` with support ironing flow.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6110-6140` handles support `erIroning` as support ironing output.

## Current Ares Boundary

- `crates/ares-core/src/print_paths/generate.rs` finalizes print paths by applying support ironing after ordinary ironing.
- `crates/ares-core/src/print_paths/support_ironing.rs` currently duplicates each `SupportMaterialInterface` path as `PrintPathRole::Ironing`, preserving source metadata and support-interface extrusion ownership.
- `crates/ares-core/src/options/ironing_flow.rs` already parses `support_ironing_flow` for support ironing extrusion height scaling.
- `crates/ares-core/src/pipeline/tests/support_ironing_paths.rs` locks existing support ironing enablement, support-flow, speed, metadata, and validation behavior.
- `crates/ares-core/src/pipeline/tests/ironing_spacing.rs` locks ordinary ironing spacing and explicitly keeps ordinary ironing spacing from changing support ironing.

## Included Behavior

1. Parse `support_ironing_spacing` from `SliceOptions::values()` with Orca's default `0.1` and valid range `0.0..=1.0`.
2. Return a `SliceError::InvalidInput` containing `support_ironing_spacing` for non-numeric, non-finite, null, boolean, object, array, or out-of-range values.
3. Pass the parsed spacing into support ironing finalization.
4. When support ironing is enabled and a source `SupportMaterialInterface` path is a closed four-point axis-aligned rectangle, generate open horizontal rectilinear `Ironing` paths over the rectangle bounds at `support_ironing_spacing`.
5. Preserve the existing support ironing compatibility shell for unsupported shapes: two-point paths, non-rectangular closed paths, and zero spacing remain a single duplicate of the source path.
6. Preserve current support ironing metadata on generated paths: role `Ironing`, extrusion role `SupportMaterialInterface`, support flow effective layer height scaling, unsupported span, seam gap, layer id, print z, and source path ordering.
7. Keep ordinary `ironing_spacing` and `filament_ironing_spacing` independent from support ironing geometry.

## Deferred Behavior

- Full Orca support contact-layer polygon discovery and `polys_to_iron` generation.
- `support_ironing_pattern` behavior, including concentric support ironing.
- Support ironing angle selection from `layer_cache.ironing_angle`.
- Non-rectangular polygon clipping/fill generation.
- Reordering/chaining parity with Orca support fill entities.
- Multi-extruder support-interface ownership beyond Ares' current support-interface path model.
- Orca binary E2E geometry parity.

## Design

Add a small support ironing runtime config in `crates/ares-core/src/options/ironing_flow.rs` or an adjacent support-ironing parser boundary. It should carry `flow_ratio` and `spacing_mm` so `generate.rs` can pass one support ironing config into `apply_support_ironing`.

Update `crates/ares-core/src/print_paths/support_ironing.rs` so rectangular support-interface paths can expand into multiple ironing paths. The implementation should follow the already-tested ordinary ironing rectangle strategy, but stay local to support ironing because support has different option ownership, no inset, and support-interface extrusion metadata. For a closed rectangle with bounds `[min_x, max_x] x [min_y, max_y]` and spacing `s > 0`, generate lines from `(min_x, y)` to `(max_x, y)` while `y <= max_y + epsilon`, starting at `min_y` and stepping by `s`. Generated support ironing lines are open paths.

Capacity preallocation does not need exact output size; correctness is more important than a speculative counting pass. Keep changes small and avoid adding shared geometry abstractions unless the implementation would otherwise duplicate large blocks.

## Acceptance Criteria

- `support_ironing_spacing = 1.0` over a closed `4.0 x 3.0` support-interface rectangle produces four open support ironing lines at `y = 0.0, 1.0, 2.0, 3.0`.
- `support_ironing_spacing = 0.5` over the same rectangle produces seven open support ironing lines from `y = 0.0` through `y = 3.0`.
- `support_ironing_spacing = 0.0` preserves the existing single closed duplicate behavior.
- Invalid `support_ironing_spacing` values fail through `finalize_print_paths` with `SliceError::InvalidInput` containing the option key.
- Existing support ironing flow and metadata tests still pass.
- Existing ordinary ironing spacing tests still pass and continue to prove ordinary spacing does not control support spacing.

## Verification

- RED: `cargo nextest run -p ares-core support_ironing_spacing`
- GREEN focused: `cargo nextest run -p ares-core support_ironing_spacing support_ironing_paths ironing_spacing`
- Final verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC check, each file at or below 400 lines

## Docs Impact

Update `docs/roadmap.md` after implementation review to record that `support_ironing_spacing` is now consumed in concrete support ironing geometry and to keep remaining support ironing pattern/angle/full polygon-fill work deferred.
