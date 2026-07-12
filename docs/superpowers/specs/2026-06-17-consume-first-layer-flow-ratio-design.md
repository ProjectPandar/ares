# Consume First Layer Flow Ratio Design

## Goal

Consume the already registered and parsed Orca `first_layer_flow_ratio` option in Ares extrusion generation so it changes emitted G-code E values for supported first-layer print roles, instead of remaining registry-only option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1214-1221` declares the Orca "other flow ratios" group on `PrintRegionConfig`, including `first_layer_flow_ratio`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1314-1322` registers `first_layer_flow_ratio` as a `coFloat` with `min = 0`, `max = 2`, `default_value = 1`, and UI text saying it affects first layer flow for the listed roles and does not affect brims and skirts.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6415-6436` applies other flow ratios during E-per-mm calculation and multiplies by `first_layer_flow_ratio` when `m_layer != nullptr && m_layer->id() == 0`.

This slice ports only that G-code extrusion-consumption behavior for Ares roles that already exist: `ExternalPerimeter`, `InternalPerimeter`, and `SparseInfill`. `Skirt` and `Brim` must not receive the first-layer multiplier. `Bridge`, `InternalBridge`, `OverhangPerimeter`, `SolidInfill`, `GapFill`, and support roles are outside this slice because Ares either has separate bridge flow behavior or lacks those Orca roles/geometry today.

## Current Ares State

- `crates/ares-core/src/options/flow_ratios.rs` already has shared `0.0..=2.0` parsing for sparse and wall flow ratios.
- `crates/ares-core/src/options.rs::SliceOptions::extrusion_options` builds `ExtrusionOptions` from parsed values.
- `crates/ares-core/src/extrusions.rs::generate_extrusion_moves` has access to `Layer::id()` and calls `ExtrusionOptions::extrusion_per_mm(role, layer.height())`.
- `ExtrusionOptions::extrusion_per_mm` currently only receives role and layer height, so it cannot know whether it is producing first-layer E values.

## Design

Add `first_layer_flow_ratio` to `ExtrusionOptions`, defaulting to `1.0`, with a builder-style setter matching the existing flow ratio setters.

Parse `first_layer_flow_ratio` through the existing flow-ratio parser path, using Orca's bounds: inclusive `0.0..=2.0`, numeric JSON values and numeric strings accepted, non-numeric or out-of-range values rejected as `SliceError::InvalidInput`.

Change extrusion calculation so first-layer status is explicit. Add an `extrusion_per_mm_for_layer(role, layer_height, is_first_layer)` method, keep the existing `extrusion_per_mm(role, layer_height)` as the non-first-layer convenience path for existing callers/tests, and have `generate_extrusion_moves` pass `layer.id() == 0`.

When `is_first_layer` is true, multiply the role-specific flow by `first_layer_flow_ratio` only for:

- `PrintPathRole::ExternalPerimeter`
- `PrintPathRole::InternalPerimeter`
- `PrintPathRole::SparseInfill`

Do not multiply `Skirt` or `Brim`, matching the upstream tooltip. Do not multiply `Bridge` or `InternalBridge` in this slice because their Ares behavior is already driven by bridge-specific options and there is no current evidence that Ares first-layer bridge paths should be modeled as Orca section roles here.

## Acceptance Criteria

- A focused red test proves a first-layer external perimeter G-code E delta changes by the ratio while a second-layer external perimeter E delta is unchanged by `first_layer_flow_ratio`.
- A focused red test proves first-layer brim E deltas are unchanged by `first_layer_flow_ratio`.
- Unit coverage proves accepted bounds `0.0` and `2.0`, and rejects non-finite/out-of-range/non-numeric `first_layer_flow_ratio`.
- Existing sparse and wall flow ratio behavior continues to pass.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the Rust LOC guard pass.

## Out of Scope

- Adding new Orca roles such as overhang perimeter, internal solid infill, gap fill, support, or support interface.
- Implementing `set_other_flow_ratios` gating. Existing Ares flow ratio slices consume specific parsed options directly; this slice follows that current behavior until a separate source-cited gate slice is planned.
- Changing geometry generation, path ordering, widths, speeds, or option registry metadata.
