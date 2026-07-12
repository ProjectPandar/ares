# Precise Z Height Runtime Design

## Goal

Consume the already-registered `precise_z_height` option as concrete layer-planning behavior. Ares should stop unconditionally clamping the final planned layer to the model top and should align the final object height only when `precise_z_height` is enabled, matching the OrcaSlicer slicing boundary.

## Upstream Boundary

Line numbers in this section are from the vendored `OrcaSlicer/` tree in this repository; `rg -n 'this->add\("precise_z_height"|precise_z_height' OrcaSlicer/src/libslic3r` resolves the option definition to `PrintConfig.cpp:3597` in that checkout.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1059` declares `((ConfigOptionBool, precise_z_height))` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3597-3604` defines `precise_z_height` as a `coBool` option with default `false` and describes precise object height as fine-tuning the last few layer heights.
- `OrcaSlicer/src/libslic3r/Print.cpp:277-279` treats `precise_z_height` changes as requiring object slicing.
- `OrcaSlicer/src/libslic3r/Slicing.hpp:192-195` exposes `generate_object_layers(..., bool is_precise_z_height)`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:24-32` defines the effective minimum layer height as `0.07` when configured as `0`, otherwise at least `0.01`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:37-42` defines the effective maximum layer height as `max(min_layer_height, max_layer_height == 0 ? 0.75 * nozzle_diameter : max_layer_height)`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:145-146` widens min/max bounds to include configured regular `layer_height`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:807-866` generates object layer boundary pairs and calls `adjust_layer_series_to_align_object_height(...)` only when `is_precise_z_height` is true.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:713-804` fine-tunes the last five generated layers, bounded by `min_layer_height` and `max_layer_height`, and leaves the layer series unchanged when fewer than six layers are available or no bounded adjustment is possible.

## Current Ares Gap

`crates/ares-core/src/planning.rs` currently computes fixed layer heights but always clamps each `next_z` to `z_max`. That makes Ares always emit a final layer exactly at the model top, so the `precise_z_height` option cannot change slicing behavior. This is source-inaccurate for Orca's default `false` path.

## Ares Destination Boundary

- `crates/ares-core/src/options.rs`: add a crate-private typed accessor `precise_z_height()` that parses a JSON bool and defaults to `false`.
- `crates/ares-core/src/planning.rs`: update `plan_layers` to build the default fixed-height layer series without final-layer clamping, then apply a small Orca-cited last-five-layer adjustment only when `options.precise_z_height()?` is true.
- `crates/ares-core/src/planning/tests.rs`: move the existing inline planning tests into a dedicated test module before adding new precise-Z tests, keeping both `planning.rs` and its test file under the 400 LOC repository limit.
- Existing downstream stages continue to consume `Layer { id, height, print_z }` through the same API.
- No CLI, filesystem, UI, OpenGL, viewer, profile inheritance, adaptive-layer-height, support, or independent Ares pipeline behavior is introduced.

## Included Behavior

- Missing `precise_z_height` defaults to `false`.
- Non-bool `precise_z_height` values return `SliceError::InvalidInput`.
- With `precise_z_height = false`, `plan_layers` uses `initial_layer_height` for the first layer and regular `layer_height` for subsequent layers, stopping when the midpoint of the next fixed-height layer would reach or pass the model height, matching `Slicing.cpp:854-858`. It must not shorten the final layer just to hit `z_max`.
- With `precise_z_height = true`, `plan_layers` first builds the same default series, then tries to align the final planned `print_z` to the model height by distributing the gap over the last five layers.
- The adjustment must preserve layer order and recompute each affected `print_z` from the previous affected boundary.
- The adjustment must respect the current effective layer-height bounds:
  - `min_layer_height` comes from `options.min_layer_heights()?[0]`, with Orca's `0` fallback represented as `0.07`; nonzero values are raised to at least `0.01`, matching `Slicing.cpp:24-32`.
  - `max_layer_height` comes from `options.max_layer_heights()?[0]`, with Orca's `0` fallback represented as `0.75 * options.nozzle_diameters()?[0]`.
  - Both bounds are widened to include the configured regular `layer_height`, mirroring `Slicing.cpp:145-146`.
- If fewer than six layers exist, or the last five layers cannot absorb the gap within bounds, keep the default non-aligned series unchanged.
- If the series already ends at the model height within Ares' existing six-decimal tolerance, leave it unchanged.

## Fixed-Height Formula For This Slice

This slice ports Orca's fixed-profile behavior into Ares' existing simplified layer planner, not adaptive layer profiles.

For model bounds `z_min..z_max`:

1. Round `z_min` and `z_max` to six decimals, as current Ares does.
2. Emit the first layer at `z_min + initial_layer_height`, unless that value is already beyond `z_max`; if the first layer would be beyond `z_max`, use `z_max` so very short valid models still produce one layer.
3. For later layers, use `next_z = previous_z + layer_height`.
4. Before emitting each later fixed-height layer, require `previous_z + 0.5 * layer_height < z_max`.
5. Do not clamp later `next_z` values to `z_max` in the default path.
6. When `precise_z_height` is true, run the last-five-layer alignment after the default series is built.

This mirrors the `generate_object_layers` stopping rule in `Slicing.cpp:829-862` while staying inside Ares' existing fixed-height planner.

## Deferred Behavior

- Adaptive layer-height profiles, `layer_height_profile`, profile smoothing, layer-height texture generation, and the full `SlicingParameters` struct.
- Raft-specific first-layer behavior, object shrinkage compensation, support/interface extruder layer-height bound merging, multi-extruder min/max aggregation, and object bottom offsets.
- Orca warnings from `Print.cpp:1316-1324`.
- Any generated Rust `PrintObjectConfig` hierarchy or `PRINT_CONFIG_CLASS_DEFINE` expansion.
- Changes to extrusion, infill combination, support generation, G-code headers, or registry metadata.

## Acceptance Criteria

- A focused option test proves `precise_z_height` defaults to `false`, accepts `true`/`false`, and rejects non-bool values.
- A focused layer-planning test with a tall model proves default `false` behavior no longer truncates the final layer to `z_max`; for `initial_layer_height = 0.2`, `layer_height = 0.2`, `min_layer_height = 0.07`, and model height `1.31`, the planned top remains `1.4`.
- A focused layer-planning test for the same model proves `precise_z_height = true` adjusts the final top to `1.31` by reducing the last five layer heights within bounds.
- A focused layer-planning test proves `precise_z_height = true` leaves a short model unchanged when fewer than six layers exist.
- A focused layer-planning test proves the default fixed-height planner does not emit an overshooting extra layer when the next regular layer midpoint would reach or pass the model height.
- A focused layer-planning test proves bounded adjustment respects `min_layer_height` by leaving the default series unchanged when the gap cannot be absorbed by the last five layers.
- Existing tests that intentionally assert exact final-layer clamping are updated to assert the new default Orca-like behavior or precise-enabled behavior, whichever they are meant to cover.
- `cargo nextest run -p ares-core` passes.
- `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard pass before completion.

## Docs Impact

- Update `docs/roadmap.md` only if it currently describes `precise_z_height` as metadata-only or deferred in a live roadmap section. If no current runtime-status entry exists, this SDD spec and implementation plan are the behavior-tracking docs for this slice.

## Safety And Simplicity

This is a small source-cited rewrite slice in `ares-core` layer planning. It should reuse existing `SliceOptions` parsing helpers and `Layer` data, avoid new dependencies, and avoid building a general adaptive-layer subsystem.
