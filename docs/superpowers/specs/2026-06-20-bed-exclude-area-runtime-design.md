# Bed Exclude Area Runtime Design

## Goal

Consume the already-registered `bed_exclude_area` option as concrete slicing validation. Ares should reject an STL slice when the current single model's XY bounds intersect the configured bed exclusion polygon, instead of leaving the option as registry/staged metadata only.

## Upstream Boundary

Line numbers are from the vendored `OrcaSlicer/` tree in this repository.

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:696-703` defines `bed_exclude_area` as a `coPoints` option with default `0x0` and describes it as an unprintable XY polygon.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1483-1484` carries the `//BBS: add bed_exclude_area` comment and `((ConfigOptionPoints, bed_exclude_area))` tuple on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11280-11292` implements `get_bed_excluded_area(const PrintConfig&)` by reading `cfg.bed_exclude_area.values`, scaling each point, making the polygon counter-clockwise, and returning it as `Polygons`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11295-11304` implements `get_bed_shape_with_excluded_area(...)` by subtracting `get_bed_excluded_area(cfg)` from the bed polygon and replacing the bed shape with the first diff result when present.
- `OrcaSlicer/src/libslic3r/Print.cpp:612-617` and `Print.cpp:936-941` consume `get_bed_excluded_area(print_config)` in sequential and layered print clearance validation.
- `OrcaSlicer/src/libslic3r/GCode.cpp:127-132` reads `print.config().bed_exclude_area.values` for the filament-change cutter area path when the exclusion area has four points.

## Current Ares Gap

At the time of this runtime slice, `bed_exclude_area` was registered in `crates/ares-core/src/options/registry/definitions/table/early.rs` while separate staged helpers did not affect `run_slicing_pipeline`, `slice`, path generation, or G-code output. Those source-line-only helpers and tests were later removed; the runtime validator in `crates/ares-core/src/options/bed_excluded_area.rs` remains.

## Ares Destination Boundary

- `crates/ares-core/src/options/bed_excluded_area.rs`: own runtime parsing for the external `SliceOptions` value of `bed_exclude_area` and model-intersection validation.
- `crates/ares-core/src/model.rs`: expose the loaded model XY bounds needed by the validation slice.
- `crates/ares-core/src/pipeline.rs`: validate the loaded model against parsed `bed_exclude_area` before layer planning.
- `crates/ares-core/src/tests/bed_exclude_area_gcode.rs`: add end-to-end `slice(...)` tests proving intersecting excluded areas reject slicing, non-intersecting excluded areas preserve output, invalid point values fail, and default `0x0` remains inactive.
- No new crates, dependencies, public API surface, G-code commands, UI behavior, complete printable-area geometry, object placement, or polygon boolean engine is introduced.

## Included Behavior

- Missing `bed_exclude_area` and the Orca default `"0x0"` are inactive because they do not define a polygon with at least three finite points.
- Ares accepts the current Orca string form `"XxY, XxY, ..."` for `bed_exclude_area`.
- Ares also accepts JSON point arrays like `[[x, y], [x, y], ...]` at the external `SliceOptions` boundary because in-memory tests and profile composition commonly use structured JSON values.
- Non-finite coordinates, malformed point pairs, empty point tokens, and unsupported JSON value shapes return `SliceError::InvalidInput` with `bed_exclude_area` in the message.
- After STL model loading succeeds, Ares computes the model XY bounds from all triangle vertices.
- If the active exclusion polygon intersects the model XY bounding rectangle, `run_slicing_pipeline` and `slice` return `SliceError::InvalidInput` before layer planning or G-code bytes are produced.
- Polygon-vs-rectangle intersection for this slice means any exclusion point inside the model bounds, any model-bounds corner inside the exclusion polygon, or any exclusion edge crossing a model-bounds edge.
- If the exclusion polygon does not intersect the model XY bounds, the existing slicing and G-code output remain unchanged except for the option count/header already reflecting provided options.

## Deferred Behavior

- Full Orca `get_bed_shape_with_excluded_area` parity, including scaling to `coord_t`, `make_counter_clockwise` mutation, Clipper `diff`, multiple result polygons, and bed polygon replacement.
- Sequential/by-object clearance parity, multi-object instance hull checks, arranged object shifts, plate origins, skirt/brim/support clearance, and height-polygons.
- Filament-change cutter travel path behavior from `GCode.cpp`, including `start_end_points` and four-point cutter-area routing.
- `printable_area`, `extruder_printable_area`, wrapping/head-wrap detection, UI validation, preset GUI behavior, and profile-file serialization beyond existing `SliceOptions` parsing.
- Concave polygon edge cases beyond deterministic polygon-vs-current-model-bounds intersection.
- Any independently designed Ares placement or bed-shape subsystem.

## Acceptance Criteria

- E2E tests prove a square-pyramid STL slice rejects a four-point `bed_exclude_area` polygon intersecting the model XY bounds.
- E2E tests prove a four-point `bed_exclude_area` polygon outside the model XY bounds still slices successfully.
- E2E tests prove missing/default `bed_exclude_area` stays inactive and does not reject the existing deterministic STL slice.
- E2E tests prove JSON point arrays are accepted for non-intersecting exclusion polygons.
- E2E tests prove malformed `bed_exclude_area` values return `SliceError::InvalidInput` with the option key in the error.
- The implementation keeps all touched `crates/**/src/**/*.rs` files at or below 400 LOC.
- Focused verification passes with `cargo nextest run -p ares-core bed_exclude_area`.
- Final verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard.

## Docs Impact

Update `docs/roadmap.md` with a short runtime slice entry stating that `bed_exclude_area` now performs current-model XY exclusion validation and that full Orca bed-shape boolean/clearance/cutter behavior remains deferred.

## Safety And Simplicity

This is a narrow validation slice at the existing in-memory core boundary. It reuses `SliceOptions`, `Model`, and `run_slicing_pipeline`, rejects invalid external option input at the boundary, and does not add dependencies or a general geometry engine.
