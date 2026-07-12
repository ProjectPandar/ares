# Consume `support_bottom_interface_spacing` Bottom-Only Runtime

## Goal

Make the already-parsed `support_bottom_interface_spacing` option affect Ares' current rectangular support-interface print-path proxy without adding new support-generation ownership. This slice routes only the existing bottom-only option shell through bottom-interface spacing; all full Orca contact-layer classification remains deferred.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1019`: `support_bottom_interface_spacing` is a `PrintObjectConfig` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6115-6122`: `support_bottom_interface_spacing` is a `coFloat`, default `0.5`, minimum `0`, labeled "Bottom interface spacing", measured in millimeters, and zero means solid bottom interface lines.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:103-110`: support interface fill uses `support_angle + 90`; top interface spacing is `support_interface_spacing + interface flow spacing`, while bottom interface spacing is `support_bottom_interface_spacing + interface flow spacing`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:116-120`: `support_interface_top_layers = 0` routes top interface spacing/density to base support spacing/density.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:245-251`: `SupportParameters` stores separate top and bottom interface spacing/density fields.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1696-1741`: interface extrusion chooses bottom interface density only for bottom-contact/interface layers; upstream bottom contacts are gated by `top_interfaces && support_interface_bottom_layers != 0`.

## Current Ares Boundary

- `crates/ares-core/src/print_paths/support_interface.rs` parses `support_interface_top_layers` and `support_interface_bottom_layers`, resolves `-1` to the top count, and reclassifies generic support-interface paths to support material only when both resolved counts are zero.
- `crates/ares-core/src/print_paths/support_interface_spacing.rs` consumes `support_interface_spacing`, `support_bottom_interface_spacing`, `support_interface_pattern`, and `support_interface_loop_pattern` for current closed rectangular `SupportMaterialInterface` paths.
- `crates/ares-core/src/print_paths/generate.rs` runs support-interface layer reclassification before support-interface spacing.
- Current Ares print paths do not distinguish top contact, bottom contact, raft contact, base interface, or generic interface layers. There is only one `PrintPathRole::SupportMaterialInterface` role.

Because Orca applies bottom spacing only to classified bottom-contact/interface layers, this slice must not apply bottom spacing to every generic interface rectangle. The only Ares-owned runtime proxy this slice may use is the existing option shell where `support_interface_top_layers = 0` and resolved `support_interface_bottom_layers > 0`, which can currently represent "bottom-only interface retained" without adding top/bottom path metadata. This is a compatibility shell, not full Orca support-contact parity.

## Included Behavior

1. Continue parsing and validating `support_bottom_interface_spacing` from existing `SliceOptions` values during print-path finalization.
2. Accept finite numeric JSON values and numeric strings in millimeters.
3. Reject negative values, non-finite values, unit-suffixed strings, arrays, booleans, nulls, and objects with `SliceError::InvalidInput` mentioning `support_bottom_interface_spacing`.
4. Use Orca's default `0.5` when the option is omitted.
5. Validate `support_bottom_interface_spacing` before the support-ironing early return.
6. Use `support_bottom_interface_spacing + support interface width` as the line pitch only when the existing Ares option shell is bottom-only:
   - `support_interface_top_layers == 0`
   - resolved `support_interface_bottom_layers > 0`
7. Preserve `support_interface_spacing + support interface width` as the pitch for omitted/default, top-enabled, mixed top+bottom, and non-bottom-only generic interface paths.
8. Preserve `support_interface_pattern`, `support_interface_loop_pattern`, `support_angle`, support expansion, and support-ironing behavior.
9. Keep non-rectangular, non-closed, and non-interface paths unchanged.

## Deferred Behavior

- Full Orca support contact-area generation from overhang polygons.
- Distinct Ares roles or metadata for top contact, bottom contact, raft contact, base interface, and generic interface layers.
- Applying bottom spacing to mixed top+bottom objects based on true bottom-contact path ownership.
- Exact upstream `bottom_interfaces = top_interfaces && support_interface_bottom_layers != 0` contact-layer extrusion behavior.
- Top-interface fallback to base support spacing/density when `support_interface_top_layers = 0`.
- Exact Orca `Flow::spacing()` parity and density computation beyond the current extrusion-width pitch approximation.
- Bottom-interface smoothing radius, bridge-flow handling for bottom contacts, base-interface layers, raft contacts, tree/organic support, support region merging, and Orca binary E2E geometry parity.

## Acceptance Criteria

1. Invalid `support_bottom_interface_spacing` values fail `finalize_print_paths` with `SliceError::InvalidInput` mentioning `support_bottom_interface_spacing`.
2. Valid numeric and numeric-string `support_bottom_interface_spacing` values are accepted in default/top, bottom-only, mixed-layer, and support-ironing scenarios.
3. In the bottom-only option shell, changing only `support_bottom_interface_spacing` changes finalized line geometry and emitted G-code.
4. In top-enabled or default generic interface scenarios, `support_interface_spacing` still controls current interface geometry even when `support_bottom_interface_spacing` differs.
5. `support_ironing = true` still validates `support_bottom_interface_spacing` before preserving the solid interface rectangle.
6. `support_interface_top_layers = 0` with omitted/default zero bottom layers still reclassifies the generic interface path to `SupportMaterial` before spacing.

## Verification

- Update focused tests in `crates/ares-core/src/pipeline/tests/support_bottom_interface_spacing.rs`.
- `cargo nextest run -p ares-core support_bottom_interface_spacing`
- `cargo nextest run -p ares-core support_interface_spacing`
- `cargo nextest run -p ares-core support_interface_top_layers_runtime`
- `cargo nextest run -p ares-core support_interface_pattern support_interface_loop_pattern support_ironing`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that `support_bottom_interface_spacing` now affects the existing bottom-only rectangular support-interface proxy, while full bottom-contact/interface classification and mixed top/bottom routing remain deferred.
