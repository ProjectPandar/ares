# Consume Seam Gap Design

## Goal

Consume OrcaSlicer `seam_gap` as concrete closed-loop perimeter behavior in Ares: a positive seam gap shortens the final closing extrusion segment of perimeter loops before G-code emission, instead of remaining option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1182` declares `ConfigOptionFloatOrPercent seam_gap`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5382-5390` defines `seam_gap`, its non-negative minimum, and its default `10%`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5725-5737` computes `seam_gap` from nozzle diameter and uses it as loop clipping length before extrusion.
- `OrcaSlicer/src/libslic3r/ExtrusionEntity.cpp:460-474` removes the last `seam_gap` length from a sloped extrusion loop segment when seam clipping applies.

## Ares Destination Boundary

- Parse `seam_gap` inside `SliceOptions::perimeter_options()` using the existing float-or-percent parser and the external perimeter line width as Ares' current nozzle-width equivalent for perimeter geometry.
- Store the resolved millimeter value on `PerimeterOptions`.
- Carry the value through `PerimeterPath` and `PrintPath`.
- Apply it in `generate_toolpath_moves` only when closing perimeter loops by shortening the auto-generated closing print move toward the start point.

## Included Behavior

- Missing `seam_gap` defaults to `10%` of external perimeter line width.
- Numeric `seam_gap` values are millimeters.
- String percentages such as `"50%"` resolve against external perimeter line width.
- `0` preserves current closed-loop behavior.
- Positive values shorten external, internal, and overhang perimeter closing moves.
- If `seam_gap` is greater than or equal to the closing segment length, omit the closing move instead of emitting a zero-length or reversed move.
- Skirt, brim, infill, gap fill, and support paths do not receive perimeter `seam_gap` clipping in this slice.

## Deferred Upstream Behavior

- Full Orca seam scarf / seam slope behavior remains deferred.
- Orca `m_enable_loop_clipping` policy switches remain deferred; Ares applies this slice directly to perimeter loop closure because Ares currently closes perimeter paths in `generate_toolpath_moves`.
- Orca print-statistics fields such as `total_seam_gap_distance` remain deferred.
- Filament-specific seam gap aliases remain deferred.
- Non-rectangular geometric robustness beyond shortening an existing straight closing segment remains deferred.

## Acceptance Criteria

1. Perimeter option parsing accepts missing, numeric, zero, and percentage `seam_gap` values.
2. Invalid negative or non-numeric `seam_gap` values return `SliceError::InvalidInput`.
3. With a rectangular external perimeter, `seam_gap = 1.0` changes the closing move target from the start point to the point one millimeter before the start along the closing edge.
4. With `"50%"` and `line_width = 0.4`, the closing move is shortened by `0.2mm`.
5. Internal and overhang perimeter loops carry the same clipping behavior.
6. Full G-code output differs in emitted `;MOVE:print:*perimeter` and `;EXTRUSION:print:*perimeter` lines, proving the option changes concrete G-code, not only stored configuration.
7. Verification uses `cargo nextest run`, not `cargo test`.

## Testing

- RED: add focused tests before implementation and confirm `cargo nextest run -p ares-core seam_gap` fails because closing perimeter moves still end at the loop start.
- GREEN: after implementation, the same command passes.
- Adjacent checks: `cargo nextest run -p ares-core seam_position overhang_reverse`.
- Full verification before commit: `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.

## Documentation Impact

This spec is the required documentation for the slice. No user-facing CLI or README changes are required because the existing byte-oriented slicing API already accepts options through `SliceOptions`.
