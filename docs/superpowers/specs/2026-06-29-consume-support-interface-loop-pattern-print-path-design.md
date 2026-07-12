# Consume support_interface_loop_pattern in support interface print paths

## Objective

Consume the already-registered `support_interface_loop_pattern` option in the current Ares support-interface print-path compatibility shell before adding more option metadata. The slice must remain a source-cited Rust rewrite step toward OrcaSlicer support generation, not a new Ares-owned support pipeline.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:962`: `support_interface_loop_pattern` field in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6055-6060`: boolean option definition, default `false`, label `Interface use loop pattern`, tooltip says it covers top contact layer supports with loops.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:831-856`: `LoopInterfaceProcessor` owns loop-contact generation and trims top-contact polygons before fill.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1426-1428`: support path generation sets `n_contact_loops` to `1` when `support_interface_loop_pattern` is true.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1625-1646`: loop contacts are generated only on the top-contact path when top interface layers are enabled.

## Ares Destination Boundary

- `crates/ares-core/src/print_paths/support_interface_spacing.rs`: parse `support_interface_loop_pattern` and apply it while current closed rectangular `SupportMaterialInterface` paths are still available, before they are converted into open interface fill lines.
- `crates/ares-core/src/pipeline/tests/support_interface_loop_pattern.rs`: focused path and G-code characterization tests. Keep this file under 400 LOC.
- `crates/ares-core/src/pipeline/tests.rs`: register the new test module.
- `docs/roadmap.md`: record the source-cited runtime slice and deferrals.

## Current Ares Compatibility Boundary

Ares currently represents generated support interface regions as closed rectangular `PrintPathRole::SupportMaterialInterface` paths. It does not yet model Orca's separate top-contact, bottom-contact, interface, base-interface, overhang polygon, or arbitrary expolygon support-generator layers.

For this slice, `support_interface_loop_pattern = true` means:

1. Parse the option as a boolean with Orca's default `false`.
2. Reject non-boolean values with `SliceError::InvalidInput` mentioning `support_interface_loop_pattern`.
3. For each eligible closed rectangular `SupportMaterialInterface` path, emit one closed `SupportMaterialInterface` loop preserving path metadata and extrusion role, followed by the existing interface fill lines generated from the same source rectangle.
4. When `support_interface_pattern = grid`, the loop must precede both generated line families.
5. When `support_ironing = true`, current Ares behavior keeps the solid interface rectangle for support ironing; this slice must still validate `support_interface_loop_pattern`, but it must not add a separate loop because fill-line conversion is skipped.
6. When `support_interface_top_layers = 0`, the earlier role-conversion pass rewrites interface paths to `SupportMaterial`; loop-pattern behavior must not affect those converted paths.
7. Non-closed, non-rectangular, and non-interface paths are unchanged.

The loop geometry for this compatibility shell is the existing rectangle contour itself. It is intentionally not an offset or overhang-trimmed loop because Ares does not yet own Orca's top-contact expolygons, overhang-contact filtering, flow-width centerline offsetting, or loop-contact trimming.

## Included Behavior

- Boolean parsing of `support_interface_loop_pattern` from `SliceOptions` values.
- Default `false` preserves current G-code and path output.
- `true` adds one closed interface loop before generated interface fill lines for closed rectangular support-interface regions.
- Metadata preservation for the loop path: effective layer height, unsupported span, seam gap, closed flag, and support-interface extrusion role.
- Composition with `support_interface_spacing`, `support_interface_pattern`, `support_angle`, `support_ironing`, and `support_interface_top_layers`.
- Tests proving path count/order, closed loop preservation, G-code role count/order, invalid-value errors, support-ironing validation without added loop, and top-layer-zero no-op behavior.

## Deferred Behavior

- Full `LoopInterfaceProcessor::generate` parity.
- Top-contact-only classification separate from generic interface layers.
- Bottom-contact/interface/base-interface layer separation.
- Overhang polygon filtering and top-contact trimming after loop generation.
- Centerline offsets by support-interface flow width and circle-radius spacing.
- Arbitrary polygon, hole, and expolygon loop generation.
- Tree/organic support, raft contacts, soluble-interface base-interface interactions, and sheath interactions.
- Orca binary E2E parity, because Ares still owns only hand-constructed rectangular support-interface artifacts rather than support contact-region generation.

## Acceptance Criteria

- `support_interface_loop_pattern` absent or `false` keeps existing support-interface output.
- `support_interface_loop_pattern = true` produces a closed `SupportMaterialInterface` loop before the open interface fill lines for an eligible rectangle.
- The loop preserves source path metadata and support-interface extrusion role.
- `support_interface_pattern = grid` with loop-pattern true produces loop, interface-angle fill lines, then base-angle fill lines.
- Invalid values such as string, number, null, array, and object return `SliceError::InvalidInput` and mention `support_interface_loop_pattern`.
- With `support_ironing = true`, invalid loop-pattern values still error; valid `true` does not add a second interface loop and preserves the solid interface rectangle for support ironing.
- With `support_interface_top_layers = 0`, loop-pattern true does not add a loop because the path has already become `SupportMaterial`.
- G-code output shows an extra `support_material_interface` extrusion before fill-line extrusion when loop-pattern is true.
- Rust files touched by the slice remain below 400 LOC.
