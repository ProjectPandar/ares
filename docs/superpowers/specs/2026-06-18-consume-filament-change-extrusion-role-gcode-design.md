# Consume Filament Change Extrusion Role G-code Design

## Goal

Consume the existing `filament_change_extrusion_role_gcode` option in `ares-core` by emitting filament-level custom G-code when the generated print extrusion role changes.

This is a concrete slicing/G-code behavior slice. It must not add new option metadata except for the runtime accessor needed to consume the already-registered option.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1395` declares `filament_change_extrusion_role_gcode` as `ConfigOptionStrings` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6534-6542` defines the option label, tooltip, multiline UI behavior, and default vector containing one empty string.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6677-6696` consumes `filament_change_extrusion_role_gcode.get_at(current_filament_id)` when `path.role()` changes, after machine-level role-change G-code and before process-level role-change G-code.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11126` lists placeholders for `filament_change_extrusion_role_gcode`: `layer_num`, `layer_z`, `extrusion_role`, and `last_extrusion_role`.

## Ares Destination Boundary

- `crates/ares-core/src/options/custom_gcode.rs` owns custom G-code accessors for `SliceOptions`.
- `crates/ares-core/src/gcode_placeholders.rs` owns direct placeholder replacement for custom G-code templates until the full Orca placeholder parser is ported.
- `crates/ares-core/src/gcode_role_change.rs` owns role-change custom G-code state and rendering coordination.
- `crates/ares-core/src/gcode_move_emit.rs` owns per-move G-code emission ordering around role-change custom G-code.
- `crates/ares-core/src/options/tests/custom_gcode_runtime.rs` owns custom G-code accessor tests.
- `crates/ares-core/src/tests/custom_gcode_role_change.rs` owns runtime role-change custom G-code tests.

## Included Behavior

1. Add `SliceOptions::filament_change_extrusion_role_gcode()` accepting either a string value or a string array.
2. Treat an absent value, empty string, empty array, or first array entry of empty string as no-op.
3. Reject non-string scalar values and arrays containing any non-string value with `SliceError::InvalidInput`, including arrays whose first entry is an empty string but later entries are non-string values.
4. For the current single-filament Ares state, select the first string array value as the active filament template.
5. Add a renderer for `filament_change_extrusion_role_gcode` that replaces brace and bracket forms of:
   - `layer_num`
   - `layer_z`
   - `extrusion_role`
   - `last_extrusion_role`
6. Preserve unknown placeholders, conditionals, and expression placeholders literally.
7. Reuse the existing print-role tracking during `format_gcode`.
8. Emit rendered filament-level role-change G-code on the same print role-change event as machine/process role-change G-code.
9. Emit filament-level role-change G-code after machine-level `change_extrusion_role_gcode` and before process-level `process_change_extrusion_role_gcode`, matching the implemented Orca branch order.
10. Emit all role-change custom G-code before the generated `;EXTRUSION:print:...` line.
11. Do not emit filament-level role-change G-code for travel moves.
12. Do not emit before the first print move because there is no prior print extrusion role in current Ares state.
13. Use Ares `PrintPathRole::as_str()` values for `extrusion_role` and `last_extrusion_role`.
14. Preserve existing default output when the option is absent or empty.
15. Keep `ares-core` platform-neutral and WASM-safe.

## Deferred Behavior

These upstream branches are intentionally out of scope for this slice:

- active filament id selection beyond the current single-filament first-value behavior,
- multi-extruder and toolchange-driven filament switching,
- Orca's full `ExtrusionRole` string vocabulary beyond current Ares `PrintPathRole::as_str()` values,
- full Orca placeholder parser expression and conditional evaluation,
- adaptive pressure advance behavior surrounding role changes,
- role marker parity with `;_EXTRUSION_ROLE`,
- Orca E2E parity for multi-extruder and calibration branches.

Future expansion must cite the exact upstream branch it ports before changing this behavior.

## Acceptance Criteria

- Accessor tests prove string, string-array, absent, empty-array, and invalid inputs for `filament_change_extrusion_role_gcode`.
- Runtime tests prove no output when the option is absent, empty string, or empty array.
- Runtime tests prove no output when the option is a string array whose first entry is an empty string.
- Runtime tests prove no filament role-change custom G-code is emitted before the first print move.
- Runtime tests prove filament role-change custom G-code is emitted when generated print roles change, before the relevant `;EXTRUSION:print:...` line.
- Runtime tests prove filament role-change custom G-code is emitted after machine role-change custom G-code and before process role-change custom G-code when all three templates are configured.
- Runtime tests prove travel moves do not trigger filament role-change custom G-code.
- Runtime tests prove brace and bracket placeholders are replaced.
- Runtime tests prove unknown conditionals/expressions are preserved.
- Accessor tests prove arrays containing any non-string value are rejected, including when the first array entry is an empty string.
- `cargo fmt --check`, targeted tests, full `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the `crates/ares-core/src` 400 LOC gate pass before commit.

## Docs Impact

No user-facing docs changes are required for this slice. The behavior consumes an already-registered Orca-compatible option inside `ares-core`; the reviewed spec and implementation plan are the durable documentation for the source-cited rewrite boundary until broader configuration/user documentation exists for runtime custom G-code hooks.

## Design Notes

This slice intentionally consumes the filament-level role-change hook using the first configured filament template because current Ares does not yet model active filament id changes in the print move stream. That still turns an existing registered option into concrete G-code behavior for the current single-filament pipeline while keeping multi-filament selection as an explicitly cited future slice.
