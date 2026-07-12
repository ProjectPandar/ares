# Consume Skirt Type Design

## Goal

Consume Orca `skirt_type` as runtime skirt behavior in Ares instead of leaving it as registry/metadata-only option coverage.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:286-288` defines `enum SkirtType` with `stCombined` and `stPerObject`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1555` declares the `PrintConfig` `skirt_type` option tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:437-441` maps `"combined"` to `stCombined` and `"perobject"` to `stPerObject`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5588-5598` defines `skirt_type` as `coEnum`, exposes both values, and defaults to `stCombined`.
- `OrcaSlicer/src/libslic3r/Print.cpp:2686-2740` generates combined skirt loops through the global print skirt path and switches to per-object skirt generation when `skirt_type == stPerObject`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5112-5236` emits combined print skirt paths from `print.skirt()` and per-object skirt paths from each object's `object_skirt()`.

## Current Ares State

- `crates/ares-core/src/options/registry/definitions/table/tail_raft_suffix.rs` registers `skirt_type` metadata with default `"combined"`.
- `SliceOptions::skirt_options()` does not parse `skirt_type`.
- `SkirtOptions` carries loop count, distance, height, speed, and draft shield, but not skirt type.
- `generate_skirts()` currently generates one combined skirt around all layer contours, which matches Orca `stCombined` at the level Ares can represent today.
- Ares has no object instance placement model in `LayerContours`, no per-object contour ownership in `generate_skirts()`, and no object-specific skirt output path equivalent to Orca `PrintObject::object_skirt()`.

## Design

Add a runtime `SkirtType` enum in the skirt domain:

- `Combined`
- `PerObject`

Parse Orca `skirt_type` option values at the `SliceOptions::skirt_options()` boundary, defaulting to `SkirtType::Combined`.

`SkirtOptions` will carry the parsed skirt type. `generate_skirts()` will keep the existing combined skirt behavior for `SkirtType::Combined`: one skirt path group around all contours on a layer, honoring existing loop, height, distance, speed, and draft-shield behavior.

For `SkirtType::PerObject`, `generate_skirts()` will return `SliceError::InvalidInput` with a message that per-object skirt generation requires per-object contour ownership. This is deliberately not a legacy fallback and not a fake implementation. Orca's `stPerObject` path depends on object convex hulls and object/instance offsets that current Ares skirt inputs do not carry, so accepting the value and silently emitting a combined skirt would produce incorrect behavior.

This slice consumes the option into the executable runtime boundary Ares can represent today: combined skirts remain valid and explicit per-object input is rejected until the upstream object-skirt data structures are ported.

## Deferred Upstream Behavior

- Per-object skirt generation remains deferred until Ares ports object-owned contour/hull data and object instance offsets from the corresponding `libslic3r` print/object structures.
- G-code emission of per-object skirts remains deferred until Ares has an object-specific skirt artifact equivalent to Orca `PrintObject::object_skirt()`.
- `skirt_start_angle`, `min_skirt_length`, and `single_loop_draft_shield` behavior remain separate option-consumption slices.
- Exact Orca round offset geometry remains deferred; current Ares skirt geometry still uses the existing rectangular bounds model.

## Acceptance Criteria

- `SliceOptions::skirt_options()` returns default `SkirtType::Combined` when `skirt_type` is absent.
- `skirt_type: "combined"` parses to `SkirtType::Combined` and preserves existing combined skirt generation.
- `skirt_type: "perobject"` parses to `SkirtType::PerObject`.
- Invalid `skirt_type` values return `SliceError::InvalidInput` at the skirt option boundary.
- Calling `generate_skirts()` with `SkirtType::PerObject` returns `SliceError::InvalidInput` instead of silently generating a combined skirt.
- A slice/G-code regression proves `skirt_type: "combined"` still emits the existing combined `;SKIRT:` and `;PRINT_PATH:skirt:` output for the square-pyramid fixture.
- A slice boundary regression proves `skirt_type: "perobject"` currently returns a clear unsupported-input error.
- Rust source files touched by this slice stay below the repository 400 LOC split threshold.

## Files

- Modify `crates/ares-core/src/skirts.rs` for `SkirtType`, `SkirtOptions`, and the explicit per-object unsupported runtime error.
- Add `crates/ares-core/src/options/skirt_type.rs` for option parsing.
- Add `crates/ares-core/src/options/tests/skirt_type.rs` for parsing tests.
- Modify `crates/ares-core/src/options.rs` only to include the new parser module and call it from `skirt_options()`.
- Modify `crates/ares-core/src/options/tests.rs` only to declare the new focused test module while preserving the 400 LOC limit.
- Modify `crates/ares-core/src/lib.rs` only to re-export `SkirtType` alongside the existing skirt API.
- Add `crates/ares-core/src/tests/skirt_type_gcode.rs` for slice/G-code regressions, and add only the module declaration to `crates/ares-core/src/tests/mod.rs`.

## Docs Impact

No architecture or roadmap update is required for this narrow runtime consumption slice. This source-cited design document, the implementation plan, and regression tests document the included combined-skirt behavior and the explicit deferral of per-object skirt generation.

## Verification

- `cargo fmt --check`
- `cargo test -p ares-core skirt_type --lib`
- `cargo test -p ares-core skirt_type_gcode --lib`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `wc -l` on touched Rust files to confirm none exceed 400 LOC
