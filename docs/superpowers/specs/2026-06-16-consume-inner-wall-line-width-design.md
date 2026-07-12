# Consume Inner Wall Line Width Design

## Goal

Make `inner_wall_line_width` affect concrete slicing output instead of remaining metadata-only: internal perimeter artifacts must use the configured inner wall width for their spacing, and internal perimeter G-code extrusion must use that width for E values.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1093` declares `outer_wall_line_width`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1155` declares `inner_wall_line_width`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027` defines `outer_wall_line_width` as a nozzle-relative float-or-percent width.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4896` defines `inner_wall_line_width` as a nozzle-relative float-or-percent width.
- `OrcaSlicer/src/libslic3r/Flow.cpp:111` constructs `Flow` from configured extrusion width and computes automatic width when the configured width is zero.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1149` uses `perimeter_flow` for other/internal perimeters.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1154` uses `ext_perimeter_flow` for external perimeters.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1161`-`1163` computes the first internal offset from external and internal perimeter spacing.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1303` uses the mixed external/internal spacing for the first internal perimeter and internal spacing for later perimeters.

## Current Ares Behavior

Ares already has:

- `PerimeterRole::External` and `PerimeterRole::Internal` in `crates/ares-core/src/perimeters.rs`.
- `PrintPathRole::ExternalPerimeter` and `PrintPathRole::InternalPerimeter` in `crates/ares-core/src/print_paths.rs`.
- `wall_loops` consumption that emits internal rectangular perimeter artifacts.
- `outer_wall_line_width` consumption in `SliceOptions::extrusion_options()` and `ExtrusionOptions::width_for_role(PrintPathRole::ExternalPerimeter)`.
- Internal perimeter speed consumption through `inner_wall_speed`.

Gap: `inner_wall_line_width` is not consumed. Internal perimeter geometry uses the external perimeter width from `perimeter_options()`, and internal perimeter extrusion falls back to generic `line_width`.

## Design

Add a distinct internal wall width to the existing extrusion and perimeter option flow:

- `SliceOptions::extrusion_options()` parses `inner_wall_line_width` with the same `extrusion_width` parser used for `outer_wall_line_width`, using the nozzle diameter for percentages.
- `ExtrusionOptions` stores `inner_wall_line_width`.
- `ExtrusionOptions::width_for_role(PrintPathRole::InternalPerimeter)` returns the configured inner wall width when it is greater than zero, otherwise it continues to fall back to `line_width`, then automatic nozzle-based width.
- `SliceOptions::perimeter_options()` passes both external and internal perimeter widths into `PerimeterOptions`.
- `PerimeterOptions` exposes `external_line_width()` and `internal_line_width()`.
- Rectangular internal loop offsets use Orca's classic spacing shape within the existing simplified rectangular generator:
  - first internal loop shrink = `(external_line_width + internal_line_width) / 2`
  - later internal loop shrink = first internal shrink + `(loop_index - 1) * internal_line_width`

This deliberately stays inside Ares's current simplified rectangular perimeter scaffold. It replaces the current single-width spacing approximation with the upstream-cited external/internal width split, without implementing polygon offsetting, Arachne, thin walls, gap fill, or variable-width bead planning.

## Included Behavior

- Numeric and percent `inner_wall_line_width` values are accepted through existing Orca-style extrusion width parsing.
- Zero `inner_wall_line_width` preserves the current fallback semantics for internal perimeter E width.
- Internal rectangular perimeter coordinates change when `inner_wall_line_width` differs from `outer_wall_line_width`.
- Internal perimeter E values change when `inner_wall_line_width` differs from `line_width`.
- Generated G-code for a rectangular multi-wall pipeline reflects the different internal perimeter coordinates and total extrusion.

## Deferred Behavior

- Full `libslic3r` polygon offset behavior.
- Arachne wall generation.
- `precise_outer_wall` and wall sequence handling.
- Rounded-rectangle `Flow::spacing()` math beyond Ares's current width-as-spacing approximation.
- Gap fill, thin wall detection, overhang walls, bridge walls, and non-rectangular internal offsets.
- Multi-extruder or per-object/per-region option inheritance beyond the current `SliceOptions` model.

## Acceptance Criteria

- Unit tests prove `inner_wall_line_width` is parsed into internal perimeter width, including percent values.
- Unit tests prove invalid negative `inner_wall_line_width` is rejected.
- Perimeter tests prove rectangular internal loop coordinates use mixed external/internal first spacing and internal spacing after that.
- Extrusion tests prove `PrintPathRole::InternalPerimeter` uses the configured inner width before fallback.
- Pipeline/G-code tests prove changing `inner_wall_line_width` changes concrete G-code output for internal perimeters.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Safety And File Size Constraints

- Keep changes inside existing core slicing modules and tests.
- Do not add dependencies.
- Do not introduce filesystem, terminal, UI, OpenGL, or native-only behavior into `ares-core`.
- Keep Rust source files at or below the repository's 400 LOC limit.
- `crates/ares-core/src/perimeters.rs` is already near the limit. Before adding new perimeter behavior tests, move its inline `#[cfg(test)] mod tests` content to `crates/ares-core/src/perimeters/tests.rs` and replace it with `#[cfg(test)] mod tests;`.
- `crates/ares-core/src/options.rs` is already near the limit. Do not add tests there; put `inner_wall_line_width` option tests in `crates/ares-core/src/options/tests/inner_wall_line_width.rs` and register that module from `crates/ares-core/src/options/tests.rs`.
- Keep implementation edits in `options.rs` compact enough that the file remains at or below 400 LOC; if that is not possible, split behavior into a focused helper module before implementing the option consumption.
- If any other touched Rust file exceeds 400 LOC after formatting, split tests or helper code before verification.
