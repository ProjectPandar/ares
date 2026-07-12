# Consume Auto Lift Z-Hop Runtime Design

## Purpose

Consume the existing OrcaSlicer `z_hop_types = "Auto Lift"` option into concrete Ares Z-hop G-code behavior. The current Ares parser accepts `"Auto Lift"` but collapses it to `Normal`, so the option does not affect runtime output the way Orca's `zhtAuto` does.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r` Z-hop selection behavior, not a new Ares-owned movement policy.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:382-388` defines `enum ZHopType` with `zhtAuto`, `zhtNormal`, `zhtSlope`, and `zhtSpiral`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1375-1378` includes `z_hop`, `z_hop_types`, and `travel_slope` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:526-530` maps `"Auto Lift"` to `zhtAuto`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5149-5162` registers the `z_hop_types` enum values and defaults to `zhtSlope`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5625-5628` forces `zhtAuto` to `LiftType::SpiralLift` during layer-change retraction.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7443-7455` maps explicit non-auto Z-hop types to normal, slope, or spiral lift.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7539-7544` and `OrcaSlicer/src/libslic3r/GCode.cpp:7573-7578` select `SpiralLift` for auto travel over overhangs and `SlopeLift` otherwise.

## Rust Destination Boundary

- `crates/ares-core/src/options/layer_change_retraction/config.rs` owns the runtime `ZHopLiftMode` passed to travel and layer-change G-code.
- `crates/ares-core/src/options/layer_change_retraction/lift_type.rs` owns parsing `z_hop_types`, `filament_z_hop_types`, and `travel_slope` into that mode.
- `crates/ares-core/src/gcode_travel_retraction.rs` owns ordinary travel retraction and pending lift moves.
- `crates/ares-core/src/gcode_layer_change_retraction.rs` owns layer-change retraction and pending lift moves.
- `crates/ares-core/src/tests/travel_retraction_gcode/z_hop_type.rs` and `crates/ares-core/src/tests/layer_change_retraction_gcode/z_hop_type.rs` own the executable G-code behavior checks for this slice.

## Included Behavior

1. Parse `"Auto Lift"` as a distinct `ZHopLiftMode::Auto { radians }` instead of collapsing it to `Normal`.
2. Preserve existing defaults: missing `z_hop_types` still behaves as Orca's default `Slope Lift`, and `travel_slope` still defaults to 3 degrees.
3. Preserve explicit behavior for `"Normal Lift"`, `"Slope Lift"`, and `"Spiral Lift"`.
4. Preserve the existing effective first-value handling for `filament_z_hop_types`, including `nil` falling back to unprefixed `z_hop_types`.
5. For ordinary travel retraction, make `Auto Lift` emit the current slope-lift path. This implements the non-overhang branch of Orca's auto travel selection for Ares's current travel model.
6. For layer-change retraction, make `Auto Lift` emit the current spiral-lift path, matching Orca's explicit layer-change override.
7. Keep invalid value validation and error wording unchanged.
8. Keep `ares-core` platform neutral and WASM-compatible; do not add filesystem, terminal, UI, OpenGL, native runtime, or new dependencies.

## Deferred Behavior

- Full ordinary-travel overhang crossing detection from `GCode::needs_retraction`, including `is_through_overhang`, current plate transforms, lower-overhang polygons, support special cases, and choosing `SpiralLift` only for auto travel over overhangs.
- Toolchange, wipe tower, nozzle change, cut, and purge retraction paths that also pass a selected lift type in Orca.
- Multi-extruder runtime filament switching beyond Ares's existing first effective `filament_z_hop_types` value.
- Exact Orca path clipping and writer internals unrelated to the current Ares travel/layer-change Z-hop tests.

## Acceptance Criteria

1. A focused RED run after changing only the Auto Lift expectations fails because current Ares still emits normal lift for auto travel and auto layer-change.
2. After implementation, `cargo nextest run -p ares-core z_hop_type` passes.
3. Adjacent retraction tests pass with `cargo nextest run -p ares-core travel_retraction_gcode layer_change_retraction_gcode`.
4. Full verification passes before commit:
   - `cargo fmt --check`
   - `cargo nextest run --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo check -p ares-core --target wasm32-unknown-unknown`
   - `git diff --check`
   - `git diff --cached --check`
   - touched Rust file LOC check with each touched Rust file at or below 400 lines
5. No new dependency, crate, feature flag, or compatibility fallback is introduced.
6. Documentation notes this slice as a concrete option-consumption milestone rather than another option metadata addition.

## Rollback

Rollback is a single git revert of the implementation commit. The change is confined to option parsing, travel/layer-change retraction G-code selection, tests, and documentation.
