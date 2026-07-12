# Consume Print Flow Ratio Design

## Goal

Consume the registered Orca `print_flow_ratio` option in Ares extrusion generation so it changes emitted G-code E values globally, instead of remaining option registry metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1181` declares `print_flow_ratio` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2239-2250` registers `print_flow_ratio` as a `coFloat` with `min = 0.01`, `max = 2`, `default_value = 1`, and tooltip text saying it changes all extrusion flow in G-code proportionally.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6398-6401` multiplies path extrusion volume by `this->config().print_flow_ratio` before role-specific flow ratios.

This slice ports only the global `print_flow_ratio` multiplier into Ares extrusion E calculation. It does not port Orca filament flow ratio, top/bottom solid infill flow ratios, scarf flow ratio, support flow ratios, or new extrusion roles.

## Current Ares State

- `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs` already registers `print_flow_ratio` metadata with default `1`.
- `crates/ares-core/src/options.rs::SliceOptions::extrusion_options` builds `ExtrusionOptions` from parsed slicing options.
- `crates/ares-core/src/extrusions.rs::ExtrusionOptions::extrusion_per_mm_for_layer` computes E per millimeter from geometric area and currently applies role-specific flow ratios plus first-layer flow.
- Existing G-code tests can compare E deltas through `format_gcode` and `rectangular_pipeline`.

## Design

Add `print_flow_ratio` to `ExtrusionOptions`, defaulting to `1.0`, with a builder-style setter matching the existing flow ratio setters.

Parse `print_flow_ratio` from `SliceOptions::extrusion_options` using Orca's bounds: inclusive `0.01..=2.0`, numeric JSON values and numeric strings accepted, non-numeric, non-finite, zero, negative, and values above `2.0` rejected as `SliceError::InvalidInput`.

Apply `print_flow_ratio` as a global multiplier in `ExtrusionOptions::extrusion_per_mm_for_layer` after the geometric extrusion area is converted to filament length and before or alongside existing role/first-layer multipliers. Because multiplication is commutative, the observable behavior must be:

- all existing Ares print roles scale proportionally with `print_flow_ratio`;
- role-specific ratios such as `brim_flow_ratio`, wall flow ratios, sparse infill flow ratio, bridge flow, and first-layer flow still multiply on top of the global ratio;
- default or omitted `print_flow_ratio` keeps existing extrusion unchanged.

## Acceptance Criteria

- A focused red test proves `print_flow_ratio` changes emitted G-code E deltas for at least one perimeter path by the configured ratio.
- A focused unit test proves `print_flow_ratio` composes with an existing role ratio, for example brim flow, instead of replacing it.
- Option parsing coverage proves accepted numeric values and numeric strings at `0.01`, `1.0`, and `2.0`, and rejects `0.0`, negatives, values above `2.0`, non-finite strings, and non-numeric strings.
- Existing first-layer, wall, sparse infill, brim, bridge, and speed behavior continues to pass.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the Rust LOC guard pass.

## SDD Gates

- The spec/design review must return literal `VERDICT: APPROVE` before planning.
- The implementation plan review must return literal `VERDICT: APPROVE` before code changes.
- The final implementation review must return literal `VERDICT: APPROVE` before commit and push.

## Docs Impact

No product docs or user-facing examples are required for this slice. The runtime behavior is covered by this source-cited SDD spec and plan, and Ares does not currently have user documentation that enumerates consumed option behavior separately from generated G-code/tests.

## Out of Scope

- Implementing `filament_flow_ratio`.
- Implementing top/bottom solid infill, scarf, support, overhang, gap fill, or solid infill flow behavior.
- Adding `set_other_flow_ratios` gating.
- Changing geometry generation, path ordering, widths, speeds, or option registry metadata.
