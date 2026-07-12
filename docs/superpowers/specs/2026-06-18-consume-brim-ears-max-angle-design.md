# Consume Brim Ears Max Angle Design

## Goal

Consume the existing `brim_ears_max_angle` option in the current Ares brim-ear generator so brim ears are emitted only for contour corners sharp enough to satisfy Orca's brim-ear angle threshold.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:926`: declares `ConfigOptionFloat brim_ears_max_angle`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1672-1682`: defines `brim_ears_max_angle`, range `0..180`, default `125`, and documents that `0` creates no brim ears while values near `180` allow ears on everything except straight sections.
- `OrcaSlicer/src/libslic3r/Brim.cpp:311-349`: `make_brim_ears_auto` computes `angle_threshold = (180 - brim_ears_max_angle) * PI / 180.0` and gathers convex or concave points that pass that threshold.
- `OrcaSlicer/src/libslic3r/Brim.cpp:459-460`: reads `brim_ears_detection_length` and `brim_ears_max_angle` from object config before auto ear generation.
- `OrcaSlicer/src/libslic3r/Brim.cpp:541,554`: passes `brim_ears_max_angle` into outer and inner auto brim-ear generation.

## Ares Boundary

- Extend `BrimOptions` with `brim_ears_max_angle_degrees`, defaulting to Orca's `125`.
- Keep the existing public `BrimOptions::new(width, object_gap, brim_type)` constructor as the default-Orca path. Do not change its signature or require external callers to pass the new value.
- Add only internal/builder-style plumbing, specifically a crate-visible setter such as `pub(crate) const fn with_brim_ears_max_angle_degrees(self, value: f64) -> Self`, so parsed `SliceOptions` can set the value without expanding the public constructor surface. Add a getter only if implementation or tests need it.
- Parse `brim_ears_max_angle` in `crates/ares-core/src/options/brim.rs` using the same range as Orca: `0.0..=180.0`.
- Keep `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/options/tests/core.rs` at or below the 400 LOC limit. If tests need new coverage, put them in a new dedicated options test module file rather than growing the near-limit files.
- Keep `crates/ares-core/src/brims.rs` at or below the 400 LOC limit. If angle math, candidate selection, or tests would push it near the limit, move those helpers into a dedicated brim submodule/file such as `crates/ares-core/src/brims/ears.rs` while preserving the existing public `brims` module surface.
- Filter Ares brim-ear candidate contour vertices by interior angle:
  - A corner emits ears only when its interior angle is less than or equal to `brim_ears_max_angle`.
  - `0` emits no ears.
  - The current rectangular case remains unchanged under the default `125` because 90 degree corners pass.
- Switch automatic brim-ear candidate centers from the current bounding-box corners to actual contour vertices before angle filtering. Keep the existing square-ear path shape once a candidate vertex is selected.
- Keep the existing square-ear approximation and existing first-layer/path/G-code plumbing.

## Included Behavior

- Default or omitted `brim_ears_max_angle` preserves current rectangular brim-ear output.
- `brim_ears_max_angle: 0` suppresses brim ears for `brim_type: "brim_ears"` without changing other brim types.
- A sharper polygon corner below the configured angle emits ears while a wider corner above the configured angle does not.
- Invalid values below `0`, above `180`, non-numeric values, and non-finite numeric values return `SliceError::InvalidInput` mentioning `brim_ears_max_angle`.

## Deferred Behavior

- `brim_ears_detection_length` contour decimation is deferred to a separate slice.
- Painted brim ear point support from model metadata is deferred.
- Round ear polygons, offset/diff geometry, EFC outline snapping, inner auto-ear concave detection, support brim ears, and full Orca `ExPolygons` boolean behavior are deferred.
- No new option metadata, dependencies, crates, public API expansion, or independently designed brim pipeline.

## Tests

- Add a brims unit test proving default `125` still emits four ears on a square.
- Add a brims unit test proving `0` emits no ears for `BrimType::BrimEars`.
- Add a brims unit test using a contour with one sharp corner and wider corners to prove angle filtering controls which vertices receive ears.
- Add an options test in a dedicated file proving `brim_ears_max_angle` accepts boundary values and rejects invalid values without pushing any Rust file over 400 LOC.
- Add or update a G-code/pipeline test proving `brim_type: "brim_ears"` plus `brim_ears_max_angle: 0` reaches final G-code as zero brim artifacts.
- Verify with targeted tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the Rust LOC gate.

## Docs Impact

This spec is the documentation artifact for the slice. No CLI or WASM API docs change is needed because `brim_ears_max_angle` already exists in the option registry and the public options map shape does not change.
