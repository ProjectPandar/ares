# Detect Narrow Internal Solid Infill Design

## Goal

Consume OrcaSlicer `detect_narrow_internal_solid_infill` as a concrete slicing behavior in `ares-core`, not as metadata-only option growth.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1017` declares `detect_narrow_internal_solid_infill` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7154-7160` defines the option as a bool, default `true`, with tooltip behavior: detect narrow internal solid infill and use concentric pattern for those areas.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:603-820` implements `split_solid_surface(...)`, which partitions `stInternalSolid` surfaces into normal and narrow areas.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1145-1178` consumes `detect_narrow_internal_solid_infill`: only `stInternalSolid` fills are considered; all-narrow areas change their pattern to `ipConcentricInternal`; mixed areas are split into normal and `ipConcentricInternal` fills.
- `OrcaSlicer/src/libslic3r/Fill/FillConcentricInternal.cpp:12-91` generates concentric-internal paths via Arachne wall toolpaths.

## Ares Destination Boundary

- `crates/ares-core/src/options/infill.rs` owns runtime parsing and storage for infill options.
- `crates/ares-core/src/options/infill/layer_role.rs` maps an Ares layer role to the active infill pattern.
- `crates/ares-core/src/infills.rs` owns internal solid infill generation for platform-neutral slicing.
- Tests live in:
  - `crates/ares-core/src/options/tests/internal_solid_infill.rs`
  - `crates/ares-core/src/infills/tests/internal_solid.rs`
  - `crates/ares-core/src/pipeline/tests/internal_solid_infill.rs`

## Included Behavior

1. Parse `detect_narrow_internal_solid_infill` as a bool in `InfillOptions`, defaulting to `true` to match Orca.
2. Expose the parsed value through an `InfillOptions` getter and test helper.
3. Add an Ares `InfillPattern::ConcentricInternal` variant used only as a generated internal solid override.
4. Keep public parsing for `"concentric"` unchanged: `internal_solid_infill_pattern = "concentric"` still remains unimplemented as a user-selected pattern in this slice.
5. Detect all-narrow internal solid rectangles in `generate_infills` when:
   - current role is `InfillLayerRole::InternalSolid`,
   - `detect_narrow_internal_solid_infill` is true,
   - the layer has exactly one contour,
   - that contour is an axis-aligned rectangle,
   - the shorter rectangular side is less than or equal to `2 * solid_line_width`.
6. Preserve the existing Ares area suppression order: `minimum_sparse_infill_area` is checked before narrow internal solid detection, and this slice does not make internal solid infill bypass that threshold.
7. For all-narrow internal solid rectangles that pass the existing area threshold, override the fill pattern to `ConcentricInternal`.
8. Keep the existing Ares `InfillPath` contract: each infill path is exactly one two-point segment. Generate `ConcentricInternal` loops as ordered edge segments, not as multi-point closed polylines. For each inset loop, emit bottom, right, top, then left segments.
9. The first loop inset is `solid_line_width / 2.0`; subsequent loops step inward by `solid_line_width`. A loop is emitted only while both inset width and inset height remain positive.
10. Preserve `InfillRole::Solid` and the existing print-path/G-code role mapping (`solid_infill` for interior dense layers).
11. Leave bottom and top surface roles unaffected.
12. Leave existing perimeter gap fill and `gap_fill_target` untouched.

## Deferred Behavior

- Full `split_solid_surface(...)` parity for mixed normal+narrow polygons is deferred. Ares does not yet have the polygon offset/opening/diff primitives needed to port `Fill.cpp:603-820` faithfully.
- Full Arachne `FillConcentricInternal` variable-width wall generation is deferred. This slice implements a rectangle-only concentric-internal path generator that is deterministic and platform-neutral.
- User-selected `"concentric"` or `"concentric_internal"` surface pattern parsing is deferred.
- Non-rectangular all-narrow detection is deferred until Ares has a source-cited polygon offset/split boundary.

## Acceptance Criteria

1. A test proves `detect_narrow_internal_solid_infill` defaults to `true`, parses `false`, and rejects non-bool input.
2. An infill unit test proves a dense internal rectangle from `(0,0)` to `(4,0.8)` with `solid_line_width = 0.4`, `minimum_sparse_infill_area = 0`, `bottom_shell_layers = 1`, `top_shell_layers = 1`, and three layers uses concentric-internal paths on the middle layer by default. Expected middle-layer paths are four `InfillRole::Solid` two-point segments in this order:
   - `(0.2,0.2) -> (3.8,0.2)`
   - `(3.8,0.2) -> (3.8,0.6)`
   - `(3.8,0.6) -> (0.2,0.6)`
   - `(0.2,0.6) -> (0.2,0.2)`
3. An infill unit test proves setting `detect_narrow_internal_solid_infill = false` preserves the configured internal solid pattern on the same narrow rectangle.
4. An infill unit test proves the exact threshold is included: a rectangle whose shorter side is exactly `2 * solid_line_width` is detected as narrow.
5. An infill unit test proves top or bottom surface layers are not rerouted by this option.
6. A pipeline/G-code test proves the option changes generated interior solid infill geometry while preserving `solid_infill` G-code role comments.
7. `cargo test -p ares-core --lib` passes.
8. `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings` pass.
9. `cargo check -p ares-core --target wasm32-unknown-unknown` passes.
10. Rust source files under `crates/**/src/**/*.rs` remain at or below 400 LOC.

## Documentation Impact

This executable slice is captured in this SDD spec and the implementation plan. No `docs/roadmap.md` or `docs/architecture/*.md` update is required because the change does not alter crate boundaries, milestone priority, or a non-negotiable architecture decision. The segment-first infill path contract remains intact and is made explicit here.

## Safety

This is a local `ares-core` slicing change with no filesystem, terminal, OpenGL, native UI, or platform-specific behavior. It adds no dependencies and preserves WASM suitability. The behavior is constrained to internal solid infill generation and can be reverted by removing the new option field, pattern variant, and narrow-rectangle branch.
