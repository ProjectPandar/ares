# Consume Support Ironing Flow Design

## Source Boundary

This slice ports the support-interface ironing flow part of OrcaSlicer:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:997-1000`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406-6444`
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:58-61`
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1898-1912`

Orca stores `support_ironing_flow` as `ConfigOptionPercent` with default `10`, range `0..=100`, and applies it by creating an ironing flow from support-interface flow with the support-interface flow height multiplied by `0.01 * support_ironing_flow`.

## Ares Boundary

Implement this in `ares-core` only:

- `crates/ares-core/src/options/ironing_flow.rs` parses the `support_ironing_flow` percent ratio.
- `crates/ares-core/src/options.rs` may change the existing `ironing_flow` module declaration visibility without increasing line count.
- `crates/ares-core/src/print_paths/generate.rs` passes the ratio into print-path finalization.
- `crates/ares-core/src/print_paths/support_ironing.rs` applies the ratio to Ares' existing duplicate support-interface ironing paths.
- `crates/ares-core/src/print_paths.rs` stores an optional extrusion role override on `PrintPath`.
- `crates/ares-core/src/moves.rs` carries that optional extrusion role override to `ToolpathMove`.
- `crates/ares-core/src/extrusions.rs` uses the override only for extrusion math and adaptive volumetric geometry.
- `crates/ares-core/src/pipeline/tests/support_ironing_paths.rs` covers the G-code-visible E-delta behavior and invalid option handling.

The public output remains an `ironing` print path/G-code role. This slice does not introduce a new public support-ironing role.
`crates/ares-core/src/options.rs` is already at the 400 LOC limit and must not gain lines in this slice; changing `mod ironing_flow;` to `pub(crate) mod ironing_flow;` is allowed because it preserves LOC and exposes only a crate-private parser module.

## Current Gap

The previous support-ironing runtime slice duplicates each `SupportMaterialInterface` path as `PrintPathRole::Ironing`. That reaches real print paths, extrusion moves, speed moves, fan handling, and G-code, but the duplicate currently uses ordinary `ironing_flow` / `filament_ironing_flow`. The upstream `support_ironing_flow` option exists in the metadata registry but does not affect the generated support-ironing E deltas.

## Required Behavior

When `support_ironing = true`, every duplicated support-interface ironing path must keep public/display role `PrintPathRole::Ironing` but use `PrintPathRole::SupportMaterialInterface` as its extrusion calculation role. This mirrors Orca's `support_material_interface_flow.with_height(...)` source boundary: line width, hardware, and support-interface flow ratio come from support interface behavior, while the generated G-code comment, speed role, fan role, and role vocabulary remain `ironing`.

Every duplicated support-interface ironing path must use an effective layer height equal to the source support-interface effective layer height multiplied by `support_ironing_flow / 100`. If the source path has no explicit effective layer height, infer the layer height from `LayerPrintPaths::print_z()` and the previous finalized layer `print_z`; for the first layer, use its own `print_z`. Store the scaled value on the duplicated path with `PrintPath::with_effective_layer_height_mm`, and store `SupportMaterialInterface` as an optional extrusion role override. Downstream toolpath and extrusion stages must carry and use that override only for extrusion math and effective line width. G-code comments, speed moves, role fan overrides, and public path role output must continue to observe `Ironing`.

`support_ironing_flow` must:

- default to `10%`;
- accept numeric and numeric-string percent values;
- accept `0`, keep the duplicated `ironing` path visible in G-code, and emit zero additional E for the support-ironing print segment;
- reject non-finite, negative, greater-than-100, percent-suffixed, bool, object, array, and null values with `SliceError::InvalidInput` mentioning `support_ironing_flow`;
- affect only support-ironing duplicate paths created from support-interface paths;
- not change ordinary `PrintPathRole::Ironing` paths generated independently from support ironing;
- not multiply support-ironing duplicate E deltas by ordinary `ironing_flow` or `filament_ironing_flow`;
- not change whether support ironing is enabled; `support_ironing = false` or omitted still emits no duplicate ironing path.

The support-ironing duplicate should continue to preserve source points, unsupported span, seam gap, and closed status. Its G-code role stays `ironing`, so existing ironing speed and ironing fan behavior remain unchanged.

## Deferred Behavior

Keep these out of scope:

- full Orca support ironing fill generation from top contact-layer polygons;
- `support_ironing_pattern`;
- `support_ironing_spacing`;
- support-specific G-code labels beyond the existing `ironing` role;
- exact multi-extruder/current-support-interface-filament ownership beyond Ares' existing single active support-interface path;
- full Orca support generation invalidation graph parity;
- UI/preset behavior and localization.

## Acceptance Criteria

- A focused RED test demonstrates `support_ironing_flow` is ignored before implementation by comparing support-ironing E deltas for `10` and `50`.
- After implementation, `support_ironing_flow = 25` produces a support-ironing E delta equal to support-interface extrusion math using the scaled `0.05mm` effective layer height for Ares' current `0.2mm` test layer, while keeping ordinary `ironing_flow` and `filament_ironing_flow` changes from controlling support ironing.
- `support_ironing_flow = 0` keeps the support-ironing `;EXTRUSION:print:ironing` line but its E value does not advance beyond the preceding support-interface E value.
- A separate focused test proves ordinary independent `PrintPathRole::Ironing` still follows `ironing_flow`, not `support_ironing_flow`.
- Invalid `support_ironing_flow` values fail through `options.extrusion_options()` or `format_gcode()` with `SliceError::InvalidInput` and an error string containing the key.
- Existing support-ironing path duplication, speed, fan, and metadata tests continue to pass.
- Verification uses `cargo nextest run`, not `cargo test`.

## Documentation Impact

Update `docs/roadmap.md` with a new dated runtime slice entry after implementation review. The entry must cite the upstream boundary above, state the concrete Ares behavior, and list the deferred behavior.
