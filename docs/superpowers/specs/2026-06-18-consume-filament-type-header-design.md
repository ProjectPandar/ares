# Consume Filament Type Header Design

## Goal

Consume the existing OrcaSlicer filament type query behavior in concrete Ares G-code output. This is a runtime output slice for options Ares already parses, not another option-metadata milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:657` declares `DynamicPrintConfig::get_filament_type(std::string &displayed_filament_type, int id = 0)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:680-681` declares `get_filament_vendor()` and no-display `get_filament_type()` query helpers.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8649-8711` implements support-material display mapping: support PLA returns value `PLA-S` and displayed value `Sup.PLA`; support PA returns value `PA-S` and displayed value `Sup.PA`; non-support filaments return their raw type.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9374-9397` implements first-entry `get_filament_vendor()` and `get_filament_type()` query helpers.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2784-2817` defines `filament_type` and `filament_is_support`; `PrintConfig.cpp:2854+` defines `filament_vendor`.

## Current Ares State

- `crates/ares-core/src/options/filament_type.rs` already implements `SliceOptions::filament_type_display(id)`, `filament_type()`, and `filament_vendor()` using Orca-compatible query behavior.
- Existing option tests cover support filament mapping for `GFS00`, `GFS01`, raw `PLA`/`PA` support fallbacks, raw non-support types, and first-entry vendor/type helpers.
- `crates/ares-core/src/gcode_header.rs` emits Ares G-code header diagnostics, hardware values, and pipeline counts, but it does not consume `filament_type`, `filament_is_support`, `filament_id`, or `filament_vendor`.

## Design

Add filament identity lines to the Ares G-code header using the existing Orca-compatible query helpers:

- `; filament_type = <first raw/query type>`
- `; filament_vendor = <first vendor>`
- `; filament_display_type = <displayed type for filament id 0>`

The header writer should receive the already-owned `SliceOptions` from `format_gcode` and call the existing query helpers there. This keeps option parsing centralized in `SliceOptions` and makes the previously implemented query behavior visible in a slicing output artifact.

For this slice, emit only the first filament/display entry. Ares currently emits single-tool G-code and does not yet model tool changes or per-object extruder assignment, so full per-tool filament header tables are deferred.

## Deferred Behavior

- Tool selection, tool-change G-code, wipe tower behavior, AMS mapping, filament compatibility checks, and multi-extruder routing.
- Per-tool or per-object filament tables beyond the first query entry.
- Temperature, fan, speed, or material compatibility behavior driven by filament type.
- Any new option registry metadata, dependencies, crates, filesystem behavior, UI behavior, or independent Ares pipeline feature.

## Docs Impact

No user-facing documentation update is required for this slice. The observable contract is the generated G-code header, and the new behavior is covered by focused G-code tests plus existing option-query tests.

## Acceptance Criteria

- G-code header output includes raw first `filament_type`, first `filament_vendor`, and displayed first filament type.
- A support PLA profile with `filament_type = ["PLA"]`, `filament_vendor = ["Orca"]`, `filament_is_support = [true]`, and `filament_id = ["GFS00"]` emits exact header lines `; filament_type = PLA`, `; filament_vendor = Orca`, and `; filament_display_type = Sup.PLA`.
- Non-support filament profiles emit the raw filament type as both `filament_type` and `filament_display_type`.
- Invalid filament type/vendor/support arrays still surface `SliceError::InvalidInput` through G-code formatting.
- Existing header diagnostics and existing filament type option tests still pass.
- No touched Rust file exceeds 400 LOC.

## Verification

- `cargo test -p ares-core --lib filament_type_gcode`
- `cargo test -p ares-core --lib filament_type`
- `cargo test -p ares-core --lib layer_gcode`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
