# Consume Min Skirt Length Design

## Goal

Consume Orca `min_skirt_length` as executable combined-skirt behavior in Ares instead of leaving it as registry/metadata-only option coverage.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1558` declares the `PrintConfig` `min_skirt_length` option tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5618-5628` defines `min_skirt_length` as `coFloat`, with minimum `0` and default `0.0`.
- `OrcaSlicer/src/libslic3r/Print.cpp:2686-2725` generates combined skirt loops and, when `min_skirt_length > 0`, keeps adding loops until the current extruder reaches the requested filament extrusion length.
- `OrcaSlicer/src/libslic3r/Print.cpp:2740-2783` applies the same loop-extension rule to per-object skirts.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5112-5236` emits combined skirts from `print.skirt()` and per-object skirts from each object skirt.

## Current Ares State

- `SliceOptions::skirt_options()` parses loop count, distance, height, speed, draft shield, and skirt type, but not `min_skirt_length`.
- `SkirtOptions` carries no minimum skirt extrusion length.
- `generate_skirts()` emits exactly the configured loop count for combined skirts.
- `SkirtType::PerObject` is already rejected because Ares has no per-object contour ownership or object-specific skirt artifacts.
- Ares has `ExtrusionOptions::extrusion_per_mm(PrintPathRole::Skirt, layer_height)` and `initial_layer_height()`, so it can compute the same unit Orca compares against: filament length in millimeters.

## Design

Add `min_skirt_length_mm` to `SkirtOptions`, defaulting to `0.0`. Parse `min_skirt_length` at the `SliceOptions::skirt_options()` boundary as a non-negative finite float with Orca default `0.0`.

Extend `generate_skirts(...)` with a `skirt_extrusion_per_mm` parameter. The caller will pass the skirt extrusion-per-mm for the initial layer using the already parsed `ExtrusionOptions` and `SliceOptions::initial_layer_height()`.

For `SkirtType::Combined`, preserve the existing configured-loop behavior when `min_skirt_length == 0`. When `min_skirt_length > 0`, generate at least the configured `skirt_loops`, then keep adding outward loops on the first generated skirt layer until the sum of each loop's centerline length multiplied by `skirt_extrusion_per_mm` reaches `min_skirt_length`.

When `skirt_loops == 0`, `min_skirt_length` does not create skirt loops by itself. This follows the cited Orca combined-skirt loop in `Print.cpp:2686-2725`, where the loop starts from `m_config.skirt_loops` and does not execute when that count is zero. Existing draft-shield behavior is not broadened in this slice.

For layers beyond the initial generated skirt layer, keep the configured loop count and existing draft-shield / height behavior. This intentionally consumes the upstream combined-skirt priming behavior Ares can represent now without inventing per-object support or multi-extruder skirt assignment.

For `SkirtType::PerObject`, keep the existing unsupported-input error. The per-object `min_skirt_length` branch remains deferred with per-object skirt generation.

Because Ares accepts API input directly and would otherwise allocate one `SkirtPath` per generated loop, combined min-length generation is bounded to 10,000 total loops per layer. If the requested `min_skirt_length` cannot be reached within that bound, `generate_skirts()` returns `SliceError::InvalidInput`. This is an Ares public-boundary safety check around the source-cited behavior, not a legacy fallback.

## Deferred Upstream Behavior

- Multi-extruder skirt-length rotation remains deferred; Ares currently emits a single stream of skirt paths without Orca's per-extruder skirt assignment machinery.
- Per-object minimum skirt length remains deferred with per-object skirt generation.
- Orca's rounded offset geometry remains deferred; Ares continues using its current rectangular combined-skirt bounds.
- Exact Orca behavior for arbitrarily huge `min_skirt_length` values is not mirrored; Ares rejects requests that would require more than 10,000 generated combined skirt loops.
- `single_loop_draft_shield` and `skirt_start_angle` remain separate option-consumption slices.

## Acceptance Criteria

- `SliceOptions::skirt_options()` returns `min_skirt_length_mm() == 0.0` when `min_skirt_length` is absent.
- `min_skirt_length` accepts numeric and numeric-string non-negative finite values.
- Invalid `min_skirt_length` values return `SliceError::InvalidInput` at the skirt option boundary.
- With `min_skirt_length == 0`, combined skirt generation and G-code output preserve the existing one-loop default behavior.
- With `min_skirt_length > 0`, combined skirt generation emits additional first-layer skirt loops until the computed skirt extrusion length reaches the requested minimum.
- With `skirt_loops == 0`, `min_skirt_length > 0` emits no skirt loops, matching Orca's zero-iteration configured loop boundary.
- Requests that would need more than 10,000 total combined skirt loops return `SliceError::InvalidInput`.
- The additional loops appear as real skirt artifacts, print paths, moves, extrusion moves, speed moves, and G-code path-following commands rather than metadata-only changes.
- `skirt_type: "perobject"` still returns the existing unsupported per-object error even when `min_skirt_length` is set.
- Rust source files touched by this slice stay below the repository 400 LOC split threshold.

## Files

- Modify `crates/ares-core/src/skirts.rs` for the `SkirtOptions` field, accessor/builder, loop-length calculation, and min-length loop extension.
- Modify `crates/ares-core/src/options.rs` to parse `min_skirt_length` and pass it into `SkirtOptions`.
- Add `crates/ares-core/src/options/tests/min_skirt_length.rs` for focused option-boundary tests.
- Modify `crates/ares-core/src/options/tests.rs` only by adding `min_skirt_length` to the existing one-line focused test module macro.
- Add `crates/ares-core/src/tests/min_skirt_length_gcode.rs` for slice/G-code regressions.
- Modify `crates/ares-core/src/tests/mod.rs` only to register the focused G-code test module.
- Modify `crates/ares-core/src/pipeline.rs` only to pass skirt extrusion-per-mm to `generate_skirts()`.
- Modify `crates/ares-core/src/pipeline/test_support.rs` only to pass skirt extrusion-per-mm to `generate_skirts()` for pipeline test helpers.

## Docs Impact

No architecture or roadmap update is required for this narrow runtime consumption slice. This source-cited design document, the implementation plan, and regression tests document the included combined-skirt behavior and the explicit deferrals.

## Verification

- `cargo test -p ares-core min_skirt_length --lib`
- `cargo test -p ares-core min_skirt_length_gcode --lib`
- `cargo fmt --check`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `wc -l` on touched Rust files, including `crates/ares-core/src/pipeline/test_support.rs`, to confirm none exceed 400 LOC
