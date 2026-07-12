# Consume Process Change Extrusion Role G-code Design

## Goal

Consume the existing `process_change_extrusion_role_gcode` option in `ares-core` by emitting process-level custom G-code when the generated print extrusion role changes.

This is a concrete slicing/G-code behavior slice. It must not add new option metadata except for the runtime accessor needed to consume the already-registered option.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1394` declares `process_change_extrusion_role_gcode` as `ConfigOptionString` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4948-4956` defines the option label, tooltip, multiline UI behavior, and empty string default.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6677-6696` consumes `process_change_extrusion_role_gcode` when `path.role()` changes, after machine-level and filament-level role-change G-code and before the extrusion role marker and actual extrusion command.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11127` lists placeholders for `process_change_extrusion_role_gcode`: `layer_num`, `layer_z`, `extrusion_role`, and `last_extrusion_role`.

## Ares Destination Boundary

- `crates/ares-core/src/options/custom_gcode.rs` owns custom G-code accessors for `SliceOptions`.
- `crates/ares-core/src/gcode_placeholders.rs` owns direct placeholder replacement for custom G-code templates until the full Orca placeholder parser is ported.
- `crates/ares-core/src/gcode_role_change.rs` owns role-change custom G-code state and rendering coordination.
- `crates/ares-core/src/gcode_move_emit.rs` owns per-move G-code emission ordering around role-change custom G-code.
- `crates/ares-core/src/tests/custom_gcode_role_change.rs` owns runtime role-change custom G-code tests.

## Included Behavior

1. Add `SliceOptions::process_change_extrusion_role_gcode()` accepting a string value, absent value as no-op, and rejecting non-string values with `SliceError::InvalidInput`.
2. Add a renderer for `process_change_extrusion_role_gcode` that replaces brace and bracket forms of:
   - `layer_num`
   - `layer_z`
   - `extrusion_role`
   - `last_extrusion_role`
3. Preserve unknown placeholders, conditionals, and expression placeholders literally.
4. Reuse the existing print-role tracking during `format_gcode`.
5. Emit rendered `process_change_extrusion_role_gcode` on the same print role-change event as `change_extrusion_role_gcode`.
6. Emit process-level role-change G-code after machine-level `change_extrusion_role_gcode` and before the generated `;EXTRUSION:print:...` line.
7. Do not emit process-level role-change G-code for travel moves.
8. Do not emit before the first print move because there is no prior print extrusion role in current Ares state.
9. Use Ares `PrintPathRole::as_str()` values for `extrusion_role` and `last_extrusion_role`.
10. Preserve existing default output when the option is absent or empty.
11. Keep `ares-core` platform-neutral and WASM-safe.

## Deferred Behavior

These upstream branches are intentionally out of scope for this slice:

- `filament_change_extrusion_role_gcode`, because Ares does not yet have active filament id/vector selection for role-change templates,
- Orca's full `ExtrusionRole` string vocabulary beyond current Ares `PrintPathRole::as_str()` values,
- full Orca placeholder parser expression and conditional evaluation,
- adaptive pressure advance behavior surrounding role changes,
- role marker parity with `;_EXTRUSION_ROLE`,
- Orca E2E parity for multi-extruder and calibration branches.

Future expansion must cite the exact upstream branch it ports before changing this behavior.

## Acceptance Criteria

- Accessor tests prove string, absent, and invalid inputs for `process_change_extrusion_role_gcode`.
- Runtime tests prove no output when the option is absent or empty.
- Runtime tests prove no process role-change custom G-code is emitted before the first print move.
- Runtime tests prove process role-change custom G-code is emitted when generated print roles change, before the relevant `;EXTRUSION:print:...` line.
- Runtime tests prove process role-change custom G-code is emitted after machine role-change custom G-code when both templates are configured.
- Runtime tests prove travel moves do not trigger process role-change custom G-code.
- Runtime tests prove brace and bracket placeholders are replaced.
- Runtime tests prove unknown conditionals/expressions are preserved.
- `cargo fmt --check`, targeted tests, full `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the `crates/ares-core/src` 400 LOC gate pass before commit.

## Design Notes

This slice intentionally chooses the process-level role-change hook because current Ares already has the same print role-change event used by Orca for machine, filament, and process role-change templates. It leaves filament-level role-change templates for a later slice that can cite and implement the required active filament selection state.
