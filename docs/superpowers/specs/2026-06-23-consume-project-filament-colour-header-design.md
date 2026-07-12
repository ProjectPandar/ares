# Consume Project Filament Colour Header Design

## Goal

Consume the currently deferred OrcaSlicer project-filament colour options `filament_multi_colour` and `filament_colour_new` into concrete Ares G-code header output.

This is a source-cited rewrite slice of OrcaSlicer configuration export behavior. It does not add new option metadata and does not design a new Ares pipeline feature.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2385-2390` defines `filament_multi_colour` as `coStrings`, immediately followed by `filament_colour_type`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1608-1612` places `filament_colour_new` in the `PrintConfig` project-filaments section as `ConfigOptionFloats`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`, iterating non-banned config keys and writing `; key = cfg.opt_serialize(key)` for non-nil options.

## Rust Destination Boundary

- `crates/ares-core/src/options/filament_type.rs` owns the existing `FilamentConfigExports` structure and Orca-compatible header serialization helpers for string, bool, int, and float vectors.
- `crates/ares-core/src/gcode_header.rs` owns the current Ares header export sequence for filament config keys.
- `crates/ares-core/src/tests/` owns async G-code header behavior tests for individual filament export keys.

## Included Behavior

- When `filament_multi_colour` is present as a JSON array of strings, Ares writes one header line:
  - `; filament_multi_colour = <serialized ConfigOptionStrings value>`
- `filament_multi_colour` reuses the same string-vector serialization already used by `filament_colour` and `filament_colour_type`: semicolon-separated values, quoting empty single values and values containing whitespace, quotes, backslashes, CR, or LF.
- When `filament_colour_new` is present as a JSON array of numbers, Ares writes one header line:
  - `; filament_colour_new = <serialized ConfigOptionFloats value>`
- `filament_colour_new` reuses the existing non-negative finite float-vector serialization already used by `filament_change_length`.
- Missing `filament_multi_colour` and missing `filament_colour_new` remain silent and do not add default header lines.
- Invalid values return `SliceError::InvalidInput` naming the offending option key, including when a BTT thumbnail setting would otherwise suppress header output.
- The change stays within `ares-core` and remains platform-neutral: no file I/O, terminal behavior, UI, OpenGL, native viewer runtime, or non-WASM behavior.

## Deferred Behavior

- Full Orca `append_full_config` exhaustive export ordering and every config key not already handled by Ares remain deferred.
- UI/project-filament color editing, gradient rendering, AMS/multi-colour semantics, and viewer behavior remain deferred.
- Default insertion for omitted keys remains deferred; this slice only exports user-supplied option values.
- `filament_colour_new` use as a project-filament calculation input before slicing remains deferred beyond header serialization.
- Flush matrix correction, wipe tower/tool-change behavior, filament map expansion, and multi-extruder project-filament state remain deferred.
- Movement, extrusion, fan, temperature, speed, acceleration, and layer planning behavior remain unchanged.

## Acceptance Criteria

- A focused nextest run fails before implementation when tests expect `filament_multi_colour` and `filament_colour_new` header lines.
- After implementation, focused nextest passes for both new header behavior tests.
- `cargo fmt --check` passes.
- `cargo nextest run --workspace` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- `cargo check -p ares-core --target wasm32-unknown-unknown` passes.
- `git diff --check` passes.
- Touched Rust files stay at or below 400 LOC.
- `docs/roadmap.md` records this source-cited runtime slice and explicitly lists included and deferred behavior.

## Non-Goals

- No new crates, dependencies, feature flags, compatibility shims, or legacy fallbacks.
- No changes under `OrcaSlicer/`.
- No edits to `crates/ares-core/src/options.rs`, which is already at the 400 LOC project limit.
