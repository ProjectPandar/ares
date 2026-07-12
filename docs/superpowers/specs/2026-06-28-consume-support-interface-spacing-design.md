# Consume `support_interface_spacing` in Support Interface Paths

## Goal

Consume the existing `support_interface_spacing` option in concrete slicing output before adding more options. The runtime slice applies Orca's top-interface line spacing concept to Ares support-interface print-path artifacts that the current Rust pipeline can represent: closed rectangular `SupportMaterialInterface` paths during print-path finalization.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:967`: `support_interface_spacing` is a `PrintObjectConfig` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6104-6112`: `support_interface_spacing` is a `coFloat`, default `0.5`, minimum `0`, labeled "Top interface spacing", measured in millimeters, and its tooltip says zero means solid interface and support ironing forces a solid interface.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:106-107`: top interface spacing is `(support_ironing ? 0 : support_interface_spacing) + support_material_interface_flow.spacing()`, and top interface density is derived from support-interface flow spacing divided by that pitch.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:116-120`: `support_interface_top_layers = 0` routes top interface spacing to base support spacing and density instead of top interface settings.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1589-1592`: support interface angle and pattern selection are downstream of support interface pattern behavior, not part of this spacing-only slice.

## Current Ares Boundary

- `crates/ares-core/src/print_paths/generate.rs`: finalizes `LayerPrintPaths` after path generation.
- `crates/ares-core/src/print_paths/support_interface.rs`: already consumes `support_interface_top_layers` and `support_expansion` for current support path artifacts.
- `crates/ares-core/src/print_paths/support_ironing.rs`: already duplicates support interface paths for support ironing and generates rectangular support-ironing lines/loops.
- `crates/ares-core/src/options.rs` and option registry tests already expose the historical option metadata and default value for `support_interface_spacing`.
- Ares does not yet have Orca's full support-area generator, support-flow spacing model, support-interface pattern selection, or arbitrary polygon fill clipping. This slice is a temporary compatibility shell over current rectangular print paths and must stay source-cited to the upstream support-interface spacing boundary.

## Included Behavior

1. Parse `support_interface_spacing` from existing `SliceOptions` values during print-path finalization.
2. Accept finite numeric JSON values and numeric strings in millimeters.
3. Reject negative values, non-finite values, unit-suffixed strings, arrays, booleans, nulls, and objects with `SliceError::InvalidInput` mentioning `support_interface_spacing`.
4. Use Orca's default `0.5` when the option is omitted, so omitted values actively produce the default top-interface pitch.
5. For closed rectangular `SupportMaterialInterface` paths, replace the rectangle shell with horizontal open interface lines from `min_y` through `max_y`.
6. Use an effective line pitch of `support_interface_spacing + extrusion_options.width_for_role(PrintPathRole::SupportMaterialInterface)`, because Ares has not yet ported Orca's flow `spacing()` model.
7. Preserve role, extrusion role, effective layer height, unsupported span, seam gap, layer id, and print Z on generated lines; generated line paths are open.
8. Leave non-interface roles, non-rectangular interface paths, and non-closed interface paths unchanged.
9. Run after `support_interface_top_layers` role rewriting and `support_expansion`, so top-layer disabling prevents spacing conversion and expansion changes the rectangle that spacing fills.
10. When `support_ironing` is enabled, keep the current Ares support-interface compatibility shell as a solid closed rectangle instead of applying spacing conversion, reflecting Orca's forced-solid top interface rule for support ironing without claiming exact Orca fill geometry parity. Existing support ironing then consumes the rectangle with `support_ironing_spacing` and `support_ironing_pattern`.

## Deferred Behavior

- Orca support contact-area generation from overhang polygons.
- Exact support-interface flow `spacing()` parity and density computation beyond the current extrusion-width approximation.
- `support_interface_pattern`, `support_interface_loop_pattern`, `support_angle`, angle alternation, interlaced/grid/contact fill behavior, and path ordering.
- Bottom interface spacing and raft interface spacing.
- Arbitrary polygon fill clipping, holes, islands, chaining, and non-rectangular support-interface geometry.
- Tree support behavior and multi-extruder support ownership.
- Orca binary E2E geometry parity.

## Acceptance Criteria

1. Omitted `support_interface_spacing` uses the Orca default `0.5` and converts a closed rectangular support-interface path into open interface lines using `0.5 + support-interface extrusion width` pitch.
2. `support_interface_spacing: 0.0` generates denser open interface lines using only the support-interface extrusion width as pitch.
3. A larger spacing value reduces generated line count and changes emitted support-interface G-code coordinates.
4. Generated support-interface lines preserve source metadata and remain `SupportMaterialInterface` role paths.
5. `support_interface_top_layers: 0` runs before spacing conversion, so the same source path becomes `SupportMaterial` and is not converted into interface lines.
6. `support_ironing: true` leaves the support-interface rectangle solid and closed, and support ironing still duplicates that rectangle according to the support-ironing options.
7. Non-rectangular, non-closed, and non-interface paths are unchanged.
8. Invalid values produce `SliceError::InvalidInput` mentioning `support_interface_spacing`.

## Verification

- Targeted tests for `support_interface_spacing` in `ares-core`.
- Update impacted support-interface/support-ironing tests only where the new default spacing behavior changes expected path counts.
- `cargo nextest run -p ares-core support_interface_spacing`
- `cargo nextest run -p ares-core support_interface`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that `support_interface_spacing` is now consumed for current rectangular support-interface print-path artifacts and that full Orca support generation, pattern selection, and exact flow spacing remain deferred.
