# Consume draft_shield Zero-Loop Skirt Design

## Goal

Consume the existing OrcaSlicer `draft_shield` behavior that turns `skirt_loops = 0` into one effective skirt loop when draft shield is enabled. This is a narrow `libslic3r` rewrite slice: Ares' current skirt generator already supports draft-shield output across non-empty layers, but it still treats zero configured loops as no skirt output even when `draft_shield = "enabled"`.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5573-5586` defines `draft_shield` and documents that enabled draft shield makes the skirt as tall as the highest printed object.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11308-11323` implements the dynamic-config helper behavior: `has_skirt` is true when draft shield is not disabled, and `get_real_skirt_dist` overrides `loops` from `0` to `1` when draft shield is enabled.
- `OrcaSlicer/src/libslic3r/Print.cpp:572-582` makes infinite skirt/draft-shield behavior depend on enabled draft shield and positive configured skirt loops, while `has_skirt` remains tied to positive `skirt_height`.
- `OrcaSlicer/src/libslic3r/Print.cpp:1933-1938` applies the same `skirts == 0 && has_infinite_skirt()` one-loop override when computing skirt/brim clearance.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4254-4258` keeps emitting skirt loops on later layers while the skirt height is not satisfied or infinite skirt is active.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4358-4388` applies `single_loop_draft_shield` after the first layer by selecting only the last skirt loop.

## Current Ares Boundary

- `crates/ares-core/src/options.rs:245` parses `skirt_loops`, `skirt_height`, `draft_shield`, `single_loop_draft_shield`, `skirt_type`, `min_skirt_length`, and `skirt_start_angle` into `SkirtOptions`.
- `crates/ares-core/src/skirts/mod.rs` owns the current platform-neutral skirt generation boundary and already supports combined skirts, draft-shield output beyond `skirt_height`, minimum skirt length, single-loop draft shield after the first generated layer, and start-angle ordering.
- `crates/ares-core/src/skirts/mod.rs:112-118` currently returns configured loops unchanged, so `draft_shield = enabled` plus `skirt_loops = 0` suppresses all skirt output.
- `crates/ares-core/src/skirts/tests.rs:70-86` currently locks the wrong behavior by asserting zero enabled draft-shield loops emit no paths.
- `crates/ares-core/src/tests/skirt_gcode.rs` already has G-code-level skirt and draft-shield coverage.

## Requirements

1. When `SkirtOptions` has `draft_shield = Enabled` and configured `loops = 0`, Ares must use one effective loop for skirt generation.
2. The one-loop override must affect `generate_skirts` output, pipeline diagnostics, print paths, extrusion moves, speed moves, and final G-code through the existing skirt pipeline.
3. Disabled draft shield with `skirt_loops = 0` must still emit no skirt paths.
4. Enabled draft shield with positive `skirt_loops` must preserve the configured loop count.
5. `single_loop_draft_shield` must still reduce later generated layers to one loop after the first generated layer. For the zero-loop override this naturally means one loop on every non-empty generated draft-shield layer.
6. `min_skirt_length` must continue applying only to the first generated skirt layer. If the zero-loop override creates the first layer's base loop, min-length expansion may add extra first-layer loops through the existing min-length code.
7. Keep the current Ares simplification that draft-shield height is represented by output on every non-empty contour layer. Do not port full Orca convex-hull, object-height, support-layer, wipe-tower, sequential-print, or per-object draft-shield behavior in this slice.
8. Do not edit `crates/ares-core/src/options.rs`; it is exactly at the 400 LOC project limit and the behavior can be implemented inside `SkirtOptions`.

## Rust Destination

- Modify `crates/ares-core/src/skirts/mod.rs` only for runtime behavior.
- Modify `crates/ares-core/src/skirts/tests.rs` by replacing the incorrect zero-loop draft-shield unit test and adding only minimal assertions if needed while keeping the file under 400 LOC.
- Modify `crates/ares-core/src/tests/skirt_gcode.rs` with one focused G-code integration test if line budget allows.
- Update `docs/roadmap.md` after implementation review.

## Deferred Behavior

Full Orca `_make_skirt` convex-hull behavior, exact object-height cutoff, support-layer inclusion, wipe tower inclusion, brim/draft-shield intersection geometry, per-object skirt generation, multi-extruder skirt allocation, sequential-print handling, object clearance calculation, and binary Orca E2E parity remain deferred. This slice does not add new option metadata, new crates, dependencies, UI behavior, filesystem behavior, or terminal behavior to `ares-core`.

## Acceptance Criteria

- Focused RED test demonstrates current failure for `draft_shield = enabled` with `skirt_loops = 0`.
- Unit-level skirt generation shows enabled draft shield with zero loops emits one path on each non-empty generated layer.
- G-code-level slicing shows `draft_shield = "enabled"` and `skirt_loops = 0` reaches concrete skirt output and `; total_skirt_path_count` through the public slice API.
- Existing skirt behavior for disabled zero loops, positive draft-shield loops, `single_loop_draft_shield`, and `skirt_speed` remains intact.
- Verification uses `cargo nextest run`, not `cargo test`.
