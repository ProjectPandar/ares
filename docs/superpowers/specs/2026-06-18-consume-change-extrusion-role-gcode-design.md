# Consume Change Extrusion Role G-code Design

## Goal

Consume the existing `change_extrusion_role_gcode` option in `ares-core` by emitting custom G-code when the generated print extrusion role changes.

This is a concrete slicing/G-code behavior slice. It must not add new option metadata except for the runtime accessor needed to consume the already-registered option.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1393` declares `change_extrusion_role_gcode` as `ConfigOptionString` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6525-6532` defines the option label, tooltip, multiline UI behavior, and empty string default.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6677-6696` consumes `change_extrusion_role_gcode` when `path.role()` changes, before the extrusion role marker and before the actual extrusion command.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11125` lists placeholders for `change_extrusion_role_gcode`: `layer_num`, `layer_z`, `extrusion_role`, and `last_extrusion_role`.

## Ares Destination Boundary

- `crates/ares-core/src/options/custom_gcode.rs` owns custom G-code accessors for `SliceOptions`.
- `crates/ares-core/src/gcode_placeholders.rs` owns direct placeholder replacement for custom G-code templates until the full Orca placeholder parser is ported.
- `crates/ares-core/src/gcode.rs` owns the current layer/move G-code output loop.
- A new focused internal module may own role-change custom G-code state and rendering coordination so `gcode.rs` remains under the 400 LOC project limit.
- A new focused test shard under `crates/ares-core/src/tests/` should own runtime tests so existing custom G-code test files remain under 400 LOC.

## Included Behavior

1. Add `SliceOptions::change_extrusion_role_gcode()` accepting a string value, absent value as no-op, and rejecting non-string values with `SliceError::InvalidInput`.
2. Add a renderer for `change_extrusion_role_gcode` that replaces brace and bracket forms of:
   - `layer_num`
   - `layer_z`
   - `extrusion_role`
   - `last_extrusion_role`
3. Preserve unknown placeholders, conditionals, and expression placeholders literally.
4. Track the last print extrusion role during `format_gcode`.
5. Emit rendered `change_extrusion_role_gcode` immediately before the generated `;EXTRUSION:print:...` and movement commands when the current print move role differs from the last seen print move role.
6. Do not emit role-change G-code for travel moves.
7. Do not emit before the first print move because there is no prior print extrusion role in current Ares state.
8. Use Ares `PrintPathRole::as_str()` values for `extrusion_role` and `last_extrusion_role`.
9. Preserve existing default output when the option is absent or empty.
10. Keep `ares-core` platform-neutral and WASM-safe.

## Deferred Behavior

These upstream branches are intentionally out of scope for this slice:

- `filament_change_extrusion_role_gcode`,
- `process_change_extrusion_role_gcode`,
- Orca's full `ExtrusionRole` string vocabulary beyond current Ares `PrintPathRole::as_str()` values,
- full Orca placeholder parser expression and conditional evaluation,
- adaptive pressure advance behavior surrounding role changes,
- role marker parity with `;_EXTRUSION_ROLE`,
- active filament/process selection for role-change G-code,
- Orca E2E parity for multi-extruder and calibration branches.

Future expansion must cite the exact upstream branch it ports before changing this behavior.

## Acceptance Criteria

- Accessor tests prove string, absent, and invalid inputs for `change_extrusion_role_gcode`.
- Runtime tests prove no output when the option is absent or empty.
- Runtime tests prove no role-change custom G-code is emitted before the first print move.
- Runtime tests prove role-change custom G-code is emitted when generated print roles change, before the relevant `;EXTRUSION:print:...` line.
- Runtime tests prove travel moves do not trigger role-change custom G-code.
- Runtime tests prove brace and bracket placeholders are replaced.
- Runtime tests prove unknown conditionals/expressions are preserved.
- `cargo fmt --check`, targeted tests, full `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the `crates/ares-core/src` 400 LOC gate pass before commit.

## Design Notes

This slice intentionally chooses the machine-level `change_extrusion_role_gcode` first because current Ares already has print roles and generated print moves. It leaves filament-level and process-level role-change templates for later slices that can cite and implement their required selection state.
