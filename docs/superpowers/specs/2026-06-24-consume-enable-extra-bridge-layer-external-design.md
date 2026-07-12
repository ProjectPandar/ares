# Consume enable_extra_bridge_layer External Bridge Design

Consume OrcaSlicer's `enable_extra_bridge_layer` option into concrete Ares bridge G-code behavior for the external-bridge half of Orca's extra bridge layer pass. This is a source-cited Rust rewrite slice of `libslic3r`, not a new Ares-owned bridge pipeline.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:236-239` declares `EnableExtraBridgeLayer` with `eblDisabled`, `eblExternalBridgeOnly`, `eblInternalBridgeOnly`, and `eblApplyToAll`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:384-390` maps enum strings `disabled`, `external_bridge_only`, `internal_bridge_only`, and `apply_to_all`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1871-1900` defines the `enable_extra_bridge_layer` option, its default `disabled`, and documents the external/internal/apply-to-all modes.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1704-1797` implements the external bridge pass: when the option is `external_bridge_only` or `apply_to_all`, Orca finds `stBottomBridge` polygons, intersects them with `stInternal` surfaces on the next layer, creates `stInternalAfterExternalBridge`, and temporarily reclassifies that new type back to `stBottomBridge`.

## Ares Destination Boundary

- Parse the existing dynamic option key in `crates/ares-core/src/bridges.rs` as part of `BridgeOptions`.
- Feed the parsed mode into the existing bridge context used by `crates/ares-core/src/infills.rs` and `crates/ares-core/src/print_paths.rs`.
- Keep the Ares geometry approximation at whole-layer contour overlap, matching the current `bridge_no_support` support model. Do not add polygon clipping, partial-surface splitting, support generation, or new dependencies in this slice.

## Behavior

Add an internal enum for `enable_extra_bridge_layer` with these accepted string values:

- missing or `"disabled"`: keep current behavior.
- `"external_bridge_only"`: when an unsupported bottom bridge layer is generated, the next supported internal layer over the same whole contour is generated and emitted as `bridge`.
- `"internal_bridge_only"`: accepted and stored, but it does not affect external bridge output in this slice.
- `"apply_to_all"`: same external-bridge behavior as `"external_bridge_only"`; the internal-bridge half remains deferred.

Invalid non-string or unknown values return `SliceError::InvalidInput` mentioning `enable_extra_bridge_layer`.

The extra external bridge layer must reuse existing bridge output behavior: `PrintPathRole::Bridge`, `bridge_density`, `bridge_angle`, `bridge_speed`, `bridge_flow`, thick external bridge flow, fan overrides, and G-code role comments. It must not create a new public print path role or a separate G-code formatter path.

## Deferred Behavior

- Orca's exact polygon intersection, shrink/expand filtering by perimeter widths, partial `stInternal` remainder surfaces, multi-region handling, and TBB scheduling.
- The internal bridge half from `PrintObject.cpp:3234-3360`, including `stSecondInternalBridge`, perpendicular second-layer angle, and full overlap splitting.
- Full support-generation parity, automatic bridge detection beyond current Ares `bridge_no_support`, UI/preset behavior, generated option metadata changes, crates, dependencies, file I/O, terminal behavior, OpenGL, or WASM-hostile code.

## Acceptance Criteria

- Runtime option tests prove the default mode is `disabled`, all four enum strings parse, and invalid values fail with `SliceError::InvalidInput` naming `enable_extra_bridge_layer`.
- Infill/print-path tests prove a bridge layer created by current `bridge_no_support` can mark the next layer for bridge treatment only when the mode is `external_bridge_only` or `apply_to_all`.
- G-code tests prove a three-layer contour stack changes the layer above an unsupported external bridge from solid/bottom-surface output to `;PRINT_PATH:bridge:` output when `enable_extra_bridge_layer = "external_bridge_only"`.
- G-code tests prove `"disabled"` and `"internal_bridge_only"` preserve current output for that external-bridge fixture.
- G-code tests prove the extra external bridge layer composes with existing `bridge_density` and `bridge_angle`.
- Full verification must pass with `cargo fmt --check`, focused `cargo nextest run -p ares-core ...`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust file LOC checks.

## Safety

The change stays inside platform-neutral `ares-core` data and pure geometry/path classification. It has no filesystem, terminal, UI, OpenGL, network, dependency, or native-only behavior. Rollback is reverting this spec, the plan, the option parser, bridge-context changes, tests, and the roadmap note for this slice.

## Self-Review

- No placeholders or TBD items remain.
- The scope is intentionally external-bridge-only because that is the part Ares can map onto existing `bridge_no_support` and `PrintPathRole::Bridge` behavior without inventing a new bridge pipeline.
- `internal_bridge_only` is parsed but behaviorally deferred for external bridge output, matching the explicit upstream split between `PrintObject.cpp:1704-1797` and `PrintObject.cpp:3234-3360`.
