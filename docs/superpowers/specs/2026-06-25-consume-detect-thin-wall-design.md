# Detect Thin Wall Runtime Slice Design

## Scope

Consume OrcaSlicer `detect_thin_wall` as a concrete Ares perimeter behavior instead of leaving it as option metadata. This is a source-cited Rust rewrite slice of the classic `libslic3r` perimeter generator, not a new Ares-owned slicing feature.

Upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1164-1165` owns the `PrintRegionConfig::detect_thin_wall` option.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6508-6514` defines the option as a boolean with default `false`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.hpp:117-124` documents that perimeter generation outputs loops with external thin walls and gap fill without thin walls.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:230-244` appends thin walls to perimeter traversal as external perimeter entities.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1243-1267` detects first-loop thin-wall regions when `config->detect_thin_wall` is true.

Ares destination boundary:
- `crates/ares-core/src/options/overhang_reverse.rs` parses the existing option into `PerimeterOptions`.
- `crates/ares-core/src/perimeters/options.rs` stores the parsed flag.
- `crates/ares-core/src/perimeters/thin_walls.rs` implements the narrow, currently representable classic-perimeter subset.
- `crates/ares-core/src/perimeters.rs` calls the helper during rectangle perimeter generation.
- `crates/ares-core/src/gap_fills/wall.rs` suppresses the old wall gap-fill output for the same rectangular thin-wall condition when `detect_thin_wall` is enabled.
- `crates/ares-core/src/moves.rs` must treat thin-wall external perimeters as open paths so the single line is not closed back to its start.

Open-path model:
- `PerimeterPath::new` remains the closed-loop constructor and continues to require at least three points.
- Add a narrow `PerimeterPath::open_external_thin_wall(points)` constructor that sets `role = PerimeterRole::External`, requires at least two points, and marks the path as open.
- Add a `PerimeterPath::is_closed()` accessor. All existing perimeter constructors return `true`; only the thin-wall constructor returns `false`.
- Add matching closure state to `PrintPath`, defaulted from the role for existing callers so normal external/internal/overhang perimeter loops still close, skirts/brims still close, and infill/gap-fill/support paths remain open.
- When converting perimeters to print paths, copy `PerimeterPath::is_closed()` into the resulting `PrintPath`.
- `build_print_domain` keeps the same path points and role mapping; no extra print-domain type is needed because Orca's thin walls are external perimeter entities, and Ares' print-domain role already represents that.
- `generate_toolpath_moves` closes paths by `PrintPath::is_closed()` rather than by perimeter role alone. This keeps normal external perimeter loops closed while allowing the two-point external thin wall to remain open.

## Included Behavior

When `detect_thin_wall = true`, Ares shall convert the existing narrow rectangular wall-gap condition into an external perimeter thin-wall line. For the current rectangular perimeter scaffold, this means:
- a narrow rectangle that would otherwise produce one wall `gap_fill` line instead produces zero wall `LayerGapFills` paths and one open `PerimeterRole::External` thin-wall path with the same centerline points,
- the generated thin wall reaches print paths as `PrintPathRole::ExternalPerimeter`,
- the print domain stores it under perimeter entities, not extras,
- G-code uses `;PRINT_PATH:external_perimeter:` and external-perimeter speed/extrusion behavior, not `gap_fill`,
- the generated external thin wall remains an open two-point path and does not emit an automatic closing segment.

When `detect_thin_wall = false` or omitted, current behavior remains unchanged: the same narrow rectangle continues to produce wall gap fill when `gap_infill_speed > 0`.

This replacement applies only to wall gap fills derived from the rectangular perimeter thin-wall condition. Solid-surface gap fill from `gap_fill_target` remains governed by the existing solid-surface gap-fill path and is not converted by this slice.

The option parser must use Orca's default `false`, accept JSON booleans, and reject non-boolean values before G-code output through the existing `SliceError::InvalidInput` path.

## Deferred Behavior

This slice does not implement full Orca thin-wall parity. Deferred upstream behavior includes:
- polygonal medial-axis detection for arbitrary thin-wall geometry,
- variable-width `ThickPolylines`,
- Arachne `WallToolPaths::print_thin_walls`,
- smaller external width fallback used when thin-wall detection is disabled,
- thin-wall holes and Orca print-order reversal for thin-wall holes,
- multi-region/multi-material interactions,
- overlap clipping against gap fill and infill surfaces beyond the current rectangular scaffold.

Existing Ares rectangular gap-fill code is treated as a temporary compatibility shell around the cited `PerimeterGenerator` concept only for the narrow rectangular case it already models.

## Docs Impact

Update `docs/roadmap.md` after implementation with a new dated runtime-slice entry for `detect_thin_wall`, including the same upstream boundary, the Ares rectangular subset, and the deferred full Orca parity items. No CLI, WASM, or user-facing API documentation is required because the option key already exists in the registry and the public byte-oriented API shape does not change.

## Test Strategy

Follow TDD with `cargo nextest run`, not `cargo test`.

RED:
- Add focused tests proving `detect_thin_wall = true` changes a narrow rectangular pipeline from gap-fill output to external-perimeter thin-wall output.
- Assert the enabled pipeline has zero wall `LayerGapFills` paths for the narrow rectangle, no gap-fill print-domain extras, no gap-fill print paths, and no gap-fill G-code.
- Add a unit-level perimeter test proving the thin-wall path is a two-point external perimeter.
- Add a move-level test proving a two-point external perimeter is not auto-closed.
- Add parser coverage proving `detect_thin_wall` defaults to false, accepts true, and rejects invalid non-booleans.
- Run `cargo nextest run -p ares-core detect_thin_wall` and confirm the new tests fail because the option is not consumed yet.

GREEN:
- Implement only the minimal parser, perimeter helper, and open-path closing behavior needed for the tests.
- Run `cargo nextest run -p ares-core detect_thin_wall`.
- Run adjacent regression checks: `cargo nextest run -p ares-core gap_fill_role_gcode perimeters::tests moves::tests`.

Full verification before commit:
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- `git diff --cached --check`
- touched Rust file LOC check, each file `<= 400` lines

## Acceptance Criteria

- `detect_thin_wall` is no longer metadata-only for Ares' rectangular thin-wall subset.
- The default `false` path preserves existing wall gap-fill behavior and tests.
- The enabled path emits external perimeter artifacts/G-code instead of gap-fill artifacts/G-code for the narrow rectangular case.
- The enabled path leaves no wall gap-fill artifact for the converted narrow rectangle in `layer_gap_fills`, print paths, print domain extras, diagnostics, or G-code.
- Thin-wall external perimeter paths are open and do not get a synthetic closing print move.
- No filesystem, terminal, UI, OpenGL, or native-only behavior is added to `ares-core`.
- No new dependencies or crates are introduced.
- `perimeters.rs` remains under the 400 LOC repository limit by placing helper logic in a submodule.
