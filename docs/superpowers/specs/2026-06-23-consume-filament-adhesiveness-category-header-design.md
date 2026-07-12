# Consume Filament Adhesiveness Category Header Design

## Goal

Consume the existing Orca `filament_adhesiveness_category` option as concrete Ares G-code header behavior. When a caller supplies this integer vector, Ares must emit the serialized config line `; filament_adhesiveness_category = ...` in generated G-code, matching the existing Ares pattern for source-cited Orca `GCode::append_full_config` exports.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2596-2601` defines `filament_adhesiveness_category` as `coInts`, label `Adhesiveness Category`, minimum `0`, develop mode, and default `ConfigOptionInts{0}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1320` places `filament_adhesiveness_category` in the `GCodeConfig` option tuple list.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`, iterating full config keys and appending each non-banned, non-nil option as `; key = cfg.opt_serialize(key)`. `filament_adhesiveness_category` is not in the banned list.

## Current Ares State

- The former source-line-only tuple module was removed by the Option pinning cleanup; this slice relies on the cited upstream boundary and retained registry definition.
- `crates/ares-core/src/options/registry/tests/keys/first.rs` and `crates/ares-core/src/options/registry/tests/metadata/filament.rs` already know the key as registry metadata.
- `crates/ares-core/src/gcode_header.rs` currently exports a curated subset of Orca config values through `SliceOptions::filament_config_exports()`, including integer-vector exports such as `filament_printable`, `required_nozzle_HRC`, and `filament_map`.
- `crates/ares-core/src/options/filament_type.rs` owns `FilamentConfigExports` plus the existing config-option vector serialization helpers and is already at the repository's 400 LOC limit threshold. The implementation must not grow this file beyond 400 lines.

## Design

Use the same Ares header-export path already used for adjacent Orca `append_full_config` slices:

1. Add a focused regression test module `crates/ares-core/src/tests/filament_adhesiveness_category_gcode.rs`.
2. Move the existing filament config export struct and serializer helpers out of `filament_type.rs` into a focused internal module such as `crates/ares-core/src/options/filament_config_export.rs`. This keeps the touched Rust files under the 400 LOC cap while preserving existing export behavior.
3. Add `filament_adhesiveness_category: Option<String>` to `FilamentConfigExports`.
4. Parse `filament_adhesiveness_category` only as a JSON array of i32 integers, serialize with existing Orca-compatible comma-separated `ConfigOptionInts` formatting, and reject values below `0` because the upstream option definition sets `min = 0`.
5. Append `; filament_adhesiveness_category = ...` in `gcode_header.rs` when the option is present.

## Included Behavior

- Supplied values like `[0]` emit `; filament_adhesiveness_category = 0`.
- Multiple supplied values like `[0, 2, 7]` emit `; filament_adhesiveness_category = 0,2,7`.
- Missing `filament_adhesiveness_category` emits no optional config-export line, matching existing Ares optional header export behavior.
- Invalid values are rejected at the G-code formatting boundary with `SliceError::InvalidInput`, including scalars, strings, bool arrays, string arrays, floats, negative integers, null, and integers outside i32 range.
- Invalid values are still rejected when BTT thumbnail header suppression is requested, matching existing adjacent config-export validation tests.

## Deferred Behavior

- Do not implement material adhesion policy, build-plate selection, UI behavior, preset/profile loading, or any slicing geometry behavior.
- Do not implement wipe-tower loading/unloading speed behavior or any toolchange runtime behavior from the adjacent `PrintConfig.cpp:2603+` options.
- Do not make `append_full_config` exhaustive for every Orca option in this slice.
- Do not add new crates, dependencies, filesystem access, terminal behavior, UI behavior, OpenGL/viewer behavior, or Ares-owned pipeline design.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core filament_adhesiveness_category_gcode` fails before implementation because the header line is absent.
- After implementation, `cargo nextest run -p ares-core filament_adhesiveness_category_gcode` passes.
- Adjacent existing header export tests still pass with `cargo nextest run -p ares-core filament_printable_gcode required_nozzle_hrc_gcode filament_map_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- All touched Rust source files remain at or below 400 LOC.
