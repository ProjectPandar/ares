# Consume Klipper accel_to_decel G-code Design

## Scope

Consume the existing OrcaSlicer `accel_to_decel_enable` and `accel_to_decel_factor` options in concrete Ares G-code output for Klipper acceleration commands.

This is a narrow `libslic3r` rewrite slice. It does not redesign Ares acceleration planning. It only ports the `GCodeWriter` behavior that appends `ACCEL_TO_DECEL` when Klipper acceleration is emitted.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1419-1420` declares `accel_to_decel_enable` as `ConfigOptionBool` and `accel_to_decel_factor` as `ConfigOptionPercent` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3152-3165` defines defaults: `accel_to_decel_enable = true`, `accel_to_decel_factor = 50`, factor range `1..=100`, and labels/tooltips.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:216-247` consumes these options in `GCodeWriter::set_acceleration_internal`: for Klipper, emit `SET_VELOCITY_LIMIT ACCEL=<acceleration>` and append `ACCEL_TO_DECEL=<acceleration * factor / 100>` only when `accel_to_decel_enable` is true.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:308-335` repeats the same Klipper `ACCEL_TO_DECEL` append inside Orca's combined `set_accel_and_jerk` path.

## Current Ares State

- `crates/ares-core/src/options/tests/registry_helpers/known_count/values.rs` records option defaults; the former generated source-line modules were removed by the Option pinning cleanup.
- `crates/ares-core/src/options/acceleration.rs` parses acceleration magnitudes, but not the two accel-to-decel options.
- `crates/ares-core/src/gcode_writer.rs` currently emits all dynamic acceleration changes as `M204 S...`, including Klipper, so the two existing options do not affect output.
- `crates/ares-core/src/gcode_move_emit.rs` already routes `SpeedMove::acceleration_mm_s2()` through `GCodeWriter::set_acceleration_with_comment`.

## Required Behavior

1. Add a small runtime option reader for `accel_to_decel_enable` and `accel_to_decel_factor`.
2. Defaults must match Orca: enabled is `true`; factor is `50`.
3. Accept `accel_to_decel_enable` only as a boolean.
4. Accept `accel_to_decel_factor` only as a finite JSON number percent in `1..=100`; numeric strings must be rejected for this option.
5. Reject invalid factor types, numbers that cannot convert to finite `f64`, values below `1`, and values above `100` with `SliceError::InvalidInput` mentioning `accel_to_decel_factor`. Ordinary external `serde_json` parsing rejects non-finite/out-of-range JSON numbers before `SliceOptions` exists, so the runtime parser must keep an `is_finite()` guard while tests document that boundary.
6. When `gcode_flavor` is `klipper` and an acceleration command is emitted, output `SET_VELOCITY_LIMIT ACCEL=<rounded_acceleration> ACCEL_TO_DECEL=<integer_accel_to_decel>`.
7. Compute `integer_accel_to_decel` the way Orca does: round acceleration first to the emitted unsigned integer, then apply the configured numeric percent and truncate the final `(rounded_acceleration * factor_percent) / 100` result. For example, acceleration `333.6` rounds to `334`; with factor `33`, emit `ACCEL_TO_DECEL=110`; with factor `33.5`, emit `ACCEL_TO_DECEL=111`.
8. When `accel_to_decel_enable` is `false`, Klipper acceleration output must omit `ACCEL_TO_DECEL`.
9. Non-Klipper acceleration output remains unchanged as `M204 S...`.
10. Existing suppression behavior remains unchanged: no output for `None`, zero, or unchanged acceleration.
11. `gcode_comments = true` still appends the existing acceleration comment to the whole command.

## Destination Boundary

- `crates/ares-core/src/options/acceleration.rs`: add the runtime reader and validation type for accel-to-decel settings.
- `crates/ares-core/src/gcode_writer/acceleration.rs`: new child module containing `GCodeWriter::set_acceleration_with_comment`, the writer-side accel-to-decel setter, and Klipper/non-Klipper acceleration formatting. This moves the existing method out of `gcode_writer.rs` before extending behavior so `gcode_writer.rs` remains below the 400 LOC limit.
- `crates/ares-core/src/gcode_writer.rs`: add `mod acceleration;`, initialize the writer accel-to-decel field, and expose only the setter/current-state hooks needed by the child module.
- `crates/ares-core/src/gcode.rs`: pass parsed accel-to-decel settings to the writer after `gcode_flavor` is known. This file is currently below 400 LOC and the intended edit is a single parse/set call; if the implementation would exceed 400 LOC, split startup writer configuration into a focused helper before adding behavior.
- Tests should live in existing focused option/G-code test areas unless a file is near the 400 LOC limit. Split files before any touched Rust file exceeds 400 LOC.

## Docs Impact

No user-facing documentation update is required for this slice. The behavior consumes existing Orca-compatible options that are already present in the registry; the SDD spec and plan are the durable implementation documentation for this narrow rewrite slice.

## Non-Goals

- Do not port Klipper combined `set_accel_and_jerk`; Ares currently emits acceleration and jerk separately.
- Do not change acceleration selection by role; that behavior is already represented by `SpeedMove`.
- Do not add machine acceleration clamping or separate travel acceleration commands.
- Do not add dependencies, public API, UI, filesystem behavior, or platform-specific behavior.
- Do not add source-line-only metadata modules.

## Acceptance Criteria

- Focused option tests cover defaults, boolean parsing, numeric factor parsing, and invalid inputs.
- Focused writer tests cover Klipper enabled, Klipper disabled, Klipper custom factor with uneven integer-truncation and decimal-factor cases, non-Klipper unchanged, suppression of repeated values, and comment appending.
- A G-code runtime test proves a real Klipper slice emits `SET_VELOCITY_LIMIT ACCEL=... ACCEL_TO_DECEL=...` before a move and no longer emits `M204 S...` for dynamic acceleration.
- A G-code runtime test proves disabling `accel_to_decel_enable` omits `ACCEL_TO_DECEL`.
- Verification uses `cargo nextest run`, not `cargo test`.
- `cargo fmt --check`, targeted nextest, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC checks pass before commit.
