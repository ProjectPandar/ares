# Consume Elephant Foot Layers Density

## Goal

Consume the already registered Orca `elefant_foot_layers_density` option into concrete Ares infill generation so it changes internal solid infill spacing for the elephant-foot compensation layers instead of remaining metadata-only.

The user goal is to keep converting existing Orca options into source-cited slicing or G-code behavior before adding more option metadata. This slice is bounded to the upstream FFF fill behavior that changes solid-infill density above the first layer.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:727-735` defines `elefant_foot_compensation_layers` as an integer option with default `1` and min `1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:737-747` defines `elefant_foot_layers_density` as a percent option with default `100`, min `50`, max `100`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:929-931` places `elefant_foot_compensation`, `elefant_foot_compensation_layers`, and `elefant_foot_layers_density` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1159-1161` marks those keys as slice-affecting.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1338-1344` consumes `elefant_foot_layers_density` for solid infill. When density is not 100% and the surface is solid infill, Orca applies the option from layer id 1 through `elefant_foot_compensation_layers`, with layer id 1 being the second layer, using:

```text
1.0 - (1.0 - elefant_density) * (elefant_layers - (layer_id - 1)) / elefant_layers
```

## Ares Boundary

- Parse runtime values in `crates/ares-core/src/options/infill.rs` because `InfillOptions` already owns density and spacing decisions for sparse, bottom surface, top surface, bridge, and internal bridge infill.
- Apply the density only in the infill-generation path when `InfillLayerRole::InternalSolid` is not being overridden by internal bridge density. `crates/ares-core/src/infills.rs` is already close to the repo's 400 LOC limit, so new production logic must live in a focused submodule such as `crates/ares-core/src/infills/elephant_foot.rs`; the main module may only add the minimal module declaration and call site.
- Keep output role names unchanged. Affected paths remain `solid_infill` and use the existing solid infill speed, flow, acceleration, jerk, and G-code emission code.
- Keep every touched Rust file at or below 400 LOC. If any implementation file would exceed that limit, split the logic before review.

## Required Behavior

- Default behavior is unchanged: missing `elefant_foot_layers_density` parses as 100% and leaves internal solid infill spacing at `solid_line_width`.
- Accept finite numeric and numeric-string percent values in `50..=100`.
- Reject values below 50, above 100, non-finite strings, non-numeric strings, booleans, and nulls with an error that names `elefant_foot_layers_density`.
- Parse `elefant_foot_compensation_layers` for this infill path using Orca's default of 1 layer and reject zero or malformed values.
- Apply density only to internal solid infill layers with zero-based layer index `1..=elefant_foot_compensation_layers`.
- Use Orca's linear ramp formula. For example, with `elefant_foot_layers_density = 50` and `elefant_foot_compensation_layers = 2`, layer 1 uses 50% density and layer 2 uses 75% density; later internal solid layers return to 100%.
- Do not change bottom surface density, top surface density, sparse infill density, bridge density, internal bridge density, shell role classification, path roles, or extrusion/G-code formatting beyond the changed path count and spacing caused by internal solid infill density.

## Tests

- Add option parsing tests covering default 100%, accepted values, and rejected invalid values.
- Add infill unit tests showing the Orca ramp changes only internal solid infill layers above the first layer and stops after the configured compensation layer count.
- Add a pipeline/G-code test showing configured `elefant_foot_layers_density` changes `solid_infill` path count while `bottom_surface` and `top_solid_infill` counts remain unchanged.
- Run focused tests with `cargo nextest run`, then full workspace verification before commit.

## Non-goals

- Do not implement full Orca elephant-foot polygon shrinking from `PrintObjectSlice.cpp`.
- Do not change the existing brim EFC outline behavior.
- Do not implement SLA `elefant_foot_min_width` or `SLAPrintSteps.cpp` behavior.
- Do not change `elefant_foot_compensation` geometry beyond the existing Ares brim-outline slice.
- Do not add new option metadata, crates, dependencies, UI behavior, filesystem behavior, or platform-specific behavior.
