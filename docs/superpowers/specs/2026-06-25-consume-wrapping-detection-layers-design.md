# Consume `wrapping_detection_layers` in Wrapping Detection G-code

## Source Boundary

This slice consumes the existing OrcaSlicer clumping/wrapping detection layer-count option instead of adding more option metadata.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1349` declares `((ConfigOptionInt, wrapping_detection_layers))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3993-3998` defines `wrapping_detection_layers` as an integer option with minimum `0` and default `20`.
- `OrcaSlicer/src/libslic3r/GCode/ToolOrdering.cpp:812-818` reads `config.wrapping_detection_layers` and marks only the first configured layer-tool entries as wrapping-detection wipe tower layers when wrapping detection is enabled.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1479-1481` stores the configured wrapping detection layer count for wipe tower planning when an exclude-area polygon exists.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:2577-2603` applies the wrapping-detection tower depth only while `layer_index < m_wrapping_detection_layers`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5052-5066` renders `wrapping_detection_gcode` through the custom G-code placeholder path.

## Ares Destination Boundary

The Rust destination is the existing Ares wrapping-detection compatibility layer:

- `crates/ares-core/src/gcode_wrapping_detection.rs`
- `crates/ares-core/src/tests/wrapping_detection_gcode.rs`

Ares already emits rendered wrapping detection G-code once per generated layer when `enable_wrapping_detection` is true and `wrapping_detection_gcode` is non-empty. This slice adds the upstream layer-count gate to that existing insertion point.

## Included Behavior

1. `wrapping_detection_layers` limits wrapping detection G-code emission to the first configured generated layers.
2. The default remains Orca's `20`, so existing two-layer fixtures still emit wrapping detection G-code on both layers when the option is omitted.
3. `wrapping_detection_layers = 1` emits only the first layer's wrapping detection block in the existing two-layer fixture.
4. `wrapping_detection_layers = 0` suppresses wrapping detection G-code entirely.
5. The parser accepts JSON integer numbers and integer strings for this integer option, rejects negative values, fractional values, booleans, arrays, objects, null, and non-integer strings, and reports `SliceError::InvalidInput` containing `wrapping_detection_layers`.
6. Existing behavior for `enable_wrapping_detection`, `wrapping_detection_gcode`, placeholder replacement, trailing newline insertion, and marker ordering remains unchanged.
7. `ares-core` stays platform-neutral and WASM-compatible; no filesystem, terminal, UI, OpenGL, or new dependency is introduced.

## Deferred Behavior

- Full Orca wipe tower generation and tower-depth planning remain deferred to a later source-cited `GCode/WipeTower.cpp` rewrite slice.
- `wrapping_exclude_area` geometry gating remains deferred; Ares currently has no wrapping-detection probe polygon/wipe tower geometry boundary.
- Exact Orca physical extruder placeholder mapping and `max_layer_z` tracking remain deferred; this slice preserves the current Ares placeholder compatibility behavior.
- Multi-extruder tool-ordering effects remain deferred to a later `GCode/ToolOrdering.cpp` slice.

## Design

`gcode_wrapping_detection::layer_command` will parse the configured layer count after confirming wrapping detection is enabled and the template is non-empty. The current Ares `layer_num` argument is one-based and is already exposed through `[layer_num]` placeholders, so the layer window check will be `layer_num <= wrapping_detection_layers`.

This maps Orca's zero-based `layer_index < m_wrapping_detection_layers` to Ares' current one-based layer number without changing existing placeholder semantics. A value of `0` therefore emits no wrapping detection block, matching Orca's minimum-zero option.

The parser will stay private to `gcode_wrapping_detection.rs` because the behavior is local to this compatibility insertion point. It will use `serde_json::Value` directly, matching existing Ares option-boundary parsing patterns.

## Tests

Use TDD with `cargo nextest run`, not `cargo test`.

Focused RED/GREEN command:

```bash
cargo nextest run -p ares-core wrapping_detection_layers
```

Add tests that prove:

- `wrapping_detection_layers = 1` emits only `;WRAP 1 ...`.
- `wrapping_detection_layers = 0` emits no wrapping detection lines.
- omitted `wrapping_detection_layers` preserves existing default emission for the two-layer fixture.
- invalid values return `SliceError::InvalidInput` containing `wrapping_detection_layers`.

Adjacent focused verification:

```bash
cargo nextest run -p ares-core wrapping_detection_gcode
```

Full verification before commit:

```bash
cargo fmt --check
cargo nextest run --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p ares-core --target wasm32-unknown-unknown
git diff --check
git diff --cached --check
```

Also run a touched Rust file LOC guard and keep every touched Rust file at or below 400 LOC.

## Docs Impact

This spec is the documentation update for the slice. No user-facing CLI or README text changes are required because the option is already accepted through the existing byte/options API and registry metadata.

## Rollback

Rollback is a normal git revert of the implementation commit. The changes are isolated to wrapping-detection G-code behavior, tests, and this SDD artifact.
