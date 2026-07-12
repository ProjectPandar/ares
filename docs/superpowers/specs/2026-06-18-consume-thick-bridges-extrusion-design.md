# Consume Thick Bridges Extrusion Design

## Goal

Consume OrcaSlicer `thick_bridges` in Ares extrusion planning so existing `PrintPathRole::Bridge` moves produce different extrusion amounts when thick external bridges are enabled.

This slice adds concrete slicing/G-code behavior for an option Ares already parses. It does not add option metadata or a new bridge-detection pipeline.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1855-1863`
  - `thick_bridges` is a boolean option.
  - Default is `false`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:986`
  - `PrintObjectConfig` tuple entry for `thick_bridges`.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:31-48`
  - `LayerRegion::bridging_flow(role, thick_bridge)` chooses the bridge flow model.
  - When `thick_bridge` is true, Orca uses `Flow::bridging_flow(sqrt(bridge_flow) * nozzle_diameter, nozzle_diameter)`.
  - When `thick_bridge` is false, Orca uses the normal role flow with `bridge_flow` applied as a flow ratio.
- `OrcaSlicer/src/libslic3r/Flow.hpp:106`
  - `Flow::bridging_flow(dmr, nozzle_diameter)` creates a rounded bridge flow.
- `OrcaSlicer/src/libslic3r/Flow.cpp:201-208`
  - Bridge `mm3_per_mm()` uses circle area `diameter^2 * 0.25 * PI`.
- `OrcaSlicer/src/libslic3r/Flow.cpp:154-164`
  - Normal `with_flow_ratio()` adjusts cross-section while maintaining the normal extrusion spacing model.

## Current Ares Boundary

Ares already has:

- `BridgeOptions` parsing `thick_bridges` with Orca default `false`.
- `options/flow_ratios.rs::parse_extrusion_options()` already retrieving `bridge_options()` and applying `bridge_flow` / `internal_bridge_flow`.
- `ExtrusionOptions::extrusion_per_mm_for_layer()` assigning extrusion per mm by `PrintPathRole`.
- `PrintPathRole::Bridge` and `PrintPathRole::InternalBridge`.
- Pipeline tests that construct bridge/internal-bridge paths and assert G-code extrusion comments.

This slice belongs in `ares-core` extrusion planning.

## Design

Add a focused bridge-flow mode to `ExtrusionOptions`:

- `bridge_flow`: existing multiplier for ordinary non-thick bridge flow.
- `thick_bridges`: new boolean field, default `false`.

Wire `options/flow_ratios.rs::parse_extrusion_options()` to pass `options.bridge_options()?.thick_bridges()` into `ExtrusionOptions`.

For `PrintPathRole::Bridge` only:

- If `thick_bridges == false`, keep the current Ares behavior: normal rounded-rectangle extrusion area using the role width and layer height, multiplied by `bridge_flow`.
- If `thick_bridges == true`, use Orca's rounded bridge cross-section: `diameter = sqrt(bridge_flow) * nozzle_diameter`, `mm3_per_mm = diameter * diameter * 0.25 * PI`, then convert by filament area and the already-supported global flow multipliers.

For all other roles:

- Preserve current behavior.
- `PrintPathRole::InternalBridge` continues to use `internal_bridge_flow` and does not consume `thick_bridges` in this slice. Orca has a separate `thick_internal_bridges` option, so using `thick_bridges` for internal bridges would be wrong.

## Included Behavior

- Default `thick_bridges: false` preserves existing bridge extrusion amounts.
- `thick_bridges: true` changes only `PrintPathRole::Bridge` extrusion per mm.
- `bridge_flow` composes with thick bridge area via `sqrt(bridge_flow) * nozzle_diameter`, which means the resulting circular cross-section area is `bridge_flow * nozzle_diameter^2 * 0.25 * PI`.
- Filament diameter, filament flow ratio, and print flow ratio still apply through the existing Ares extrusion multiplier path.
- `PrintPathRole::InternalBridge`, perimeters, sparse infill, skirt, and brim are unaffected.

## Deferred Behavior

- `thick_internal_bridges`.
- Support material bridge/interface behavior.
- Bridge detection, bridge density / internal bridge density spacing, bridge angle, extra bridge layers, support generation, and wipe tower bridge-flow behavior.
- Exact Orca spacing changes for generated bridge fill paths. Ares does not yet own a bridge fill spacing generator in this slice.
- Option metadata changes or new crates.

## LOC-Safe Implementation Boundary

This repository enforces a 400 LOC ceiling for Rust files, and several nearby files are already close to that limit. The implementation must stay within these edit boundaries:

- Do not add lines to `crates/ares-core/src/options.rs` unless compensated in the same change; it is already near the limit and the option wiring belongs in `crates/ares-core/src/options/flow_ratios.rs`.
- Add `thick_bridges` option wiring in `crates/ares-core/src/options/flow_ratios.rs::parse_extrusion_options`.
- Keep `crates/ares-core/src/extrusions.rs` edits narrow. If adding the field, builder, and bridge formula would push it near or over 400 LOC, split the bridge cross-section calculation into a focused sibling module such as `crates/ares-core/src/extrusions/bridge_flow.rs` before adding logic.
- Do not add thick-bridge tests to the already-large `crates/ares-core/src/extrusions/tests.rs`. Put focused tests in a new `crates/ares-core/src/extrusions/tests/thick_bridges.rs` module, with only the module declaration added to the parent test module.
- Avoid adding option parsing assertions to `crates/ares-core/src/options/tests/bridge_wiring.rs` unless required by an implementation gap. Existing invalid bool parsing coverage should remain sufficient for this slice.

## Acceptance Criteria

- Unit tests prove default and explicit `thick_bridges: false` preserve current bridge extrusion per mm.
- Unit tests prove `thick_bridges: true` changes `PrintPathRole::Bridge` extrusion per mm according to the circular bridge cross-section.
- Tests prove `bridge_flow` composes with thick bridge extrusion using the Orca formula.
- Tests prove `PrintPathRole::InternalBridge` is unaffected by `thick_bridges`.
- A pipeline/G-code test proves enabling `thick_bridges` changes bridge `;EXTRUSION:print:bridge:` output for an existing bridge path.
- Existing invalid bool parsing for `thick_bridges` remains covered.
- `cargo test -p ares-core --lib`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repo LOC check pass.

## Docs Impact

No user-facing docs are required beyond this source-cited spec and the implementation plan. The change consumes an existing Orca option in runtime behavior and does not add CLI flags, WASM APIs, public command syntax, or roadmap changes.

## Safety

The change is local to option-to-extrusion planning. It does not alter geometry generation, bridge detection, path ordering, speed planning, fan planning, file I/O, CLI behavior, WASM bindings, or G-code formatting syntax beyond existing extrusion values for `PrintPathRole::Bridge`.
