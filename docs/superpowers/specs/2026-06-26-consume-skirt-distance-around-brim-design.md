# Consume Skirt Distance Around Brim Design

## Summary

Consume the existing `skirt_distance`, `brim_width`, `brim_object_gap`, and `brim_type` runtime interaction so a non-draft-shield combined skirt is generated around the already-generated first-layer brim envelope. This is a concrete `libslic3r` rewrite slice from `OrcaSlicer/src/libslic3r/Print.cpp`, not new Ares-owned pipeline behavior.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/Print.cpp:2402-2408` generates the skirt before the brim only when draft shield is active.
- `OrcaSlicer/src/libslic3r/Print.cpp:2488-2497` builds `m_first_layer_convex_hull` from first-layer islands plus generated brim islands.
- `OrcaSlicer/src/libslic3r/Print.cpp:2500-2504` generates the non-draft-shield skirt after brim generation so it is placed around the brim.
- `OrcaSlicer/src/libslic3r/Print.cpp:2649-2651` includes `m_first_layer_convex_hull` while building the combined skirt hull unless draft shield is enabled.
- `OrcaSlicer/src/libslic3r/Print.cpp:2694-2695` documents that `skirt_distance` is the gap from the skirt to the outermost brim, so the skirt offset must not add `brim_width` a second time once the brim hull is included.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11314-11335` keeps `get_real_skirt_dist` as distance plus skirt loops and line width for bounding-box-style calculations; this slice does not port that helper as a new public API.

## Current Ares Behavior

- `crates/ares-core/src/pipeline.rs` currently calls `generate_skirts` before `generate_brims`.
- `crates/ares-core/src/skirts/mod.rs` currently computes combined skirt bounds from `LayerContours` only.
- `crates/ares-core/src/brims.rs` already generates first-layer brim paths from `BrimOptions`, including `brim_width`, `brim_object_gap`, `brim_type`, `combine_brims`, brim ears, and EFC outline.
- Result: a profile with both `brim_width > 0` and `skirt_loops > 0` places the skirt at `skirt_distance` from the object contour rather than from the outermost generated brim path.

## Rust Destination Boundary

- Add a small skirt/brim coordination boundary inside `ares-core`, under `crates/ares-core/src/skirts/brim_envelope.rs`, to keep `pipeline.rs` below 400 LOC.
- Preserve the public `generate_skirts` API for existing callers.
- Add a narrow internal API that can generate combined skirts with an optional first-layer brim envelope.
- Reorder the internal `SlicingPipeline` computation so brims are generated before skirts, then pass first-layer brim paths into the new internal skirt generation path.
- Preserve `PipelineDiagnostics.completed_stages()` public order as `Skirts` before `Brims`, because that vector reports the historical logical pipeline surface rather than the local data dependency order introduced by this slice.
- Keep final `PrintPathInput::new(&layer_skirts, &layer_brims, ...)` ordering unchanged so G-code and print paths continue to emit skirts before brims.
- Update `crates/ares-core/src/pipeline/test_support.rs` through the same internal skirt/brim coordination API used by `pipeline.rs`, so contour-based tests exercise the same behavior without duplicating orchestration logic.
- Keep `ares-core` platform-neutral and WASM-compatible: no file I/O, terminal behavior, UI, OpenGL, new crates, or new dependencies.

## Included Behavior

- For `DraftShield::Disabled` and `SkirtType::Combined`, if first-layer brims exist, the first generated skirt loop must be outside the outermost first-layer brim path by `skirt_distance`.
- Brim envelope calculation must use actual generated `LayerBrims` paths, so it naturally respects `brim_width`, `brim_object_gap`, `brim_type`, `combine_brims`, brim ears, and EFC outline as represented by Ares today.
- Existing skirt behavior remains for no-brim profiles.
- Existing draft-shield behavior remains: draft shield skirts are not expanded around brims in this slice, matching Orca's “draft shield first” ordering.
- Existing per-object skirt behavior remains object-local in this slice. Orca's per-object branch offsets from object hulls, so this slice does not make per-object skirts use a global brim envelope.
- `min_skirt_length`, `single_loop_draft_shield`, `skirt_start_angle`, `skirt_height`, and `skirt_speed` must continue to behave as before.

## Deferred Behavior

- Full polygon convex hull and Clipper offset parity for arbitrary non-rectangular brim/skirt paths.
- Support brim, wipe tower, raft/support material, by-object print ordering, and `m_brimMap` / `m_supportBrimMap` parity.
- Draft-shield brim trimming.
- Public Rust API for `get_real_skirt_dist`.
- Any option metadata additions.

## Acceptance Criteria

- A focused RED test demonstrates that with a unit square, `brim_width = 0.8`, `brim_object_gap = 0.2`, `brim_type = "outer_only"`, `skirt_distance = 1.0`, and `skirt_line_width = 0.4`, the first combined skirt loop is generated at the outer brim envelope plus `1.0` mm, not just object bounds plus `1.0` mm.
- The same test or a companion test proves that no-brim profiles retain the existing skirt coordinates.
- A draft-shield test proves the new brim-envelope expansion is not applied when draft shield is enabled.
- G-code output contains the expanded skirt path before brim print paths in final print path order, preserving current output ordering while making geometry reflect the upstream brim-aware placement.
- Existing per-object skirt tests continue to pass without being rewritten around global brim behavior.
- `cargo fmt --check`, focused `cargo nextest run` commands, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the touched Rust LOC guard pass before commit.

## Docs Impact

No user-facing documentation update is required beyond this spec, the implementation plan, and the roadmap completion note. The change consumes existing options in runtime geometry; it does not add CLI/WASM API, option metadata, profile syntax, or user-visible commands.

## Safety And Rollback

The change is limited to in-memory `ares-core` geometry coordination between existing brims and skirts. It does not change file I/O, CLI argument parsing, WASM bindings, option registration, or external dependencies. Rollback is a single commit revert that restores the previous independent skirt/brim generation order.
