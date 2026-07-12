# Consume Print Bed Placeholder Design

## Goal

Consume OrcaSlicer's `printable_area` option into concrete machine start G-code behavior by rendering `[print_bed_min]`, `[print_bed_max]`, and `[print_bed_size]`.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2874-2876` computes `BoundingBoxf bbox_bed(print.config().printable_area.values)` and registers:
  - `print_bed_min` as `{ bbox_bed.min.x(), bbox_bed.min.y() }`
  - `print_bed_max` as `{ bbox_bed.max.x(), bbox_bed.max.y() }`
  - `print_bed_size` as `{ bbox_bed.size().x(), bbox_bed.size().y() }`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:684-689` defines `printable_area` as `coPoints` with default `ConfigOptionPoints{ Vec2d(0, 0), Vec2d(200, 0), Vec2d(200, 200), Vec2d(0, 200) }`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1481` declares the `printable_area` config member.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11030-11038` defines the three dimensions placeholder names as `coFloats`.

## Ares Destination Boundary

- Add a focused `crates/ares-core/src/gcode_print_bed_placeholders.rs` helper module.
- `gcode_print_bed_placeholders` parses `SliceOptions` key `printable_area` from the existing dynamic option values, falling back to the existing Ares registry string representation of Orca's point default (`"0x0,200x0,200x200,0x200"`) when omitted.
- Supported `printable_area` input forms are:
  - Orca/Ares string point lists with comma-separated `XxY` points, for example `"0x0,200x0,200x200,0x200"`.
  - JSON arrays of point pairs, for example `[[0, 0], [220, 0], [220, 210], [0, 210]]`.
  - JSON arrays with numeric strings inside point pairs, for example `[["-5.5", "1.25"], ["205", "1.25"], ["205", "215.75"], ["-5.5", "215.75"]]`.
- The helper computes the axis-aligned bounding box from all parsed points and returns comma-joined numeric strings formatted through existing G-code decimal formatting.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` renders only `[print_bed_min]`, `[print_bed_max]`, and `[print_bed_size]` in `machine_start_gcode`, and parses `printable_area` only when the machine start template contains at least one of those placeholders.

## Included Behavior

- A machine start template containing `[print_bed_min]`, `[print_bed_max]`, or `[print_bed_size]` renders values before the first `;LAYER_CHANGE`.
- Missing `printable_area` uses the existing Orca registry default and renders `print_bed_min = 0,0`, `print_bed_max = 200,200`, and `print_bed_size = 200,200`.
- Non-origin and negative-coordinate printable areas use the actual bounding-box min, max, and size.
- Decimal coordinates are preserved with the same compact decimal formatting used by adaptive bed mesh placeholders.
- Empty strings, empty arrays, one-point areas, malformed string points, repeated `x` separators, JSON points that are not two-element arrays, non-numeric coordinates, non-finite coordinates, and wrong top-level JSON types are invalid and surface as `SliceError::InvalidInput` mentioning `printable_area` when one of the three print-bed placeholders is consumed.
- Malformed `printable_area` is ignored when `machine_start_gcode` does not contain `[print_bed_min]`, `[print_bed_max]`, or `[print_bed_size]`.
- The placeholders remain literal outside machine start G-code, including `layer_change_gcode`.

## Deferred Behavior

- Do not port Orca's `BoundingBoxf` type; use a small local min/max calculation.
- Do not consume `extruder_printable_area`, `bed_exclude_area`, bed shape clipping, `get_bed_shape`, `get_bed_shape_with_excluded_area`, plate offset, calibration PA bounding boxes, first-layer convex hull, first-layer print min/max/size, or head-wrap detection in this slice.
- Do not change adaptive bed mesh behavior, model placement, travel bounds, path generation, clipping, or any runtime G-code outside the three machine-start placeholder replacements.
- Do not add option metadata, candidate crates, dependencies, filesystem behavior, terminal behavior, UI, OpenGL, or native-only behavior.

## Acceptance Criteria

1. Focused RED tests demonstrate that `[print_bed_min]`, `[print_bed_max]`, and `[print_bed_size]` are not rendered before implementation.
2. Focused GREEN tests prove the three placeholders render from the default `printable_area`.
3. Tests prove configured string and JSON array `printable_area` values affect min, max, and size.
4. Tests cover non-origin/negative coordinates, decimal formatting, numeric strings inside JSON point pairs, invalid input propagation for the explicitly listed invalid forms when placeholders are used, malformed `printable_area` ignored when placeholders are unused, and literal preservation in layer-change scope.
5. Implementation touches only the focused core G-code/options test surface needed for these placeholders and keeps touched Rust files at or below 400 LOC.
6. Verification uses `cargo nextest run`, not `cargo test`, with focused tests, adjacent related tests, full workspace tests, clippy, wasm check, format check, diff checks, and LOC guard before commit.
7. The commit uses the repository Lore commit protocol and is pushed to `origin/codex/consume-slicing-options`.

## Test Strategy

- Add `crates/ares-core/src/tests/print_bed_placeholders_gcode.rs`.
- Register it from `crates/ares-core/src/tests/mod.rs` near other machine-start placeholder G-code modules.
- Use `slice(square_pyramid_ascii_stl(), options)` so tests verify rendered G-code bytes, not parser output only.
- Run focused command `cargo nextest run -p ares-core print_bed_placeholders`.
- Run adjacent command `cargo nextest run -p ares-core adaptive_bed_mesh_gcode bed_exclude_area printable_area`.

## Verification Commands

- `cargo fmt --check`
- `cargo nextest run -p ares-core print_bed_placeholders`
- `cargo nextest run -p ares-core adaptive_bed_mesh_gcode bed_exclude_area printable_area`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- `git diff --cached --check`
- `for file in crates/ares-core/src/gcode_machine_start_placeholders.rs crates/ares-core/src/gcode_print_bed_placeholders.rs crates/ares-core/src/lib.rs crates/ares-core/src/tests/mod.rs crates/ares-core/src/tests/print_bed_placeholders_gcode.rs; do test "$(wc -l < "$file")" -le 400 || exit 1; done`

## Docs Impact

No user-facing documentation update is required because the repository does not currently have a dedicated placeholder reference document. This source-cited SDD spec, implementation plan, and focused regression tests document the behavior.

## Safety

The change is platform-neutral Rust in `ares-core`, does not perform file I/O, terminal I/O, UI, OpenGL, networking, or native-only operations, and adds no dependencies.
