# Consume Brim Ears Detection Length Design

## Goal

Consume OrcaSlicer `brim_ears_detection_length` in Ares brim-ear generation so the option changes generated brim-ear paths, not just registry metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:925` declares `brim_ears_detection_length` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1684-1693` defines it as "Brim ear detection radius", default `1`, minimum `0`, and documents that geometry is decimated before sharp-angle detection; `0` deactivates the decimation.
- `OrcaSlicer/src/libslic3r/Brim.cpp:310-333` consumes `ear_detection_length` in `make_brim_ears_auto`: when the value is greater than zero, the contour is Douglas-Peucker decimated before convex or concave ear points are selected by `brim_ears_max_angle`.
- `OrcaSlicer/src/libslic3r/Brim.cpp:459`, `541`, and `554` pass the configured value into automatic brim-ear generation.

## Ares Destination Boundary

- Runtime option parsing belongs in `crates/ares-core/src/options/brim.rs` and `SliceOptions::brim_options()`.
- The typed value belongs in `crates/ares-core/src/brims.rs::BrimOptions`.
- The behavior belongs in `crates/ares-core/src/brims/ears.rs`, immediately before `ear_candidate_vertices` evaluates contour angles.

## Included Behavior

- Parse `brim_ears_detection_length` as a non-negative finite millimeter value with Orca default `1.0`.
- Expose the parsed value through `BrimOptions`.
- When `brim_type == "brim_ears"` and `brim_ears_detection_length > 0`, simplify the closed contour before angle detection using the same intent as Orca's `MultiPoint::_douglas_peucker` pre-pass.
- Preserve Ares' existing simplified square-ear generation and existing `brim_ears_max_angle` behavior.
- Treat `0` as "no simplification" so all current raw contour vertices remain eligible for angle detection.
- Keep all changes inside `ares-core`; no new crates or dependencies.

## Deferred Behavior

- Full `Brim.cpp` parity for `ExPolygons`, polygon offsetting, concave inner ears, brim area unioning, support brims, automatic brim width, EFC outline selection, and circular ear polygons remains deferred.
- The legacy boolean `brim_ears` remains registry metadata only; Ares continues to use `brim_type == "brim_ears"` as its runtime switch.
- `combine_brims` and `brim_use_efc_outline` remain deferred.

## Acceptance Criteria

- A contour with a small deviation produces an extra brim ear when `brim_ears_detection_length` is `0`, and produces fewer ears when the detection length is large enough to simplify that deviation away.
- The default `SliceOptions::default().brim_options()` reports `brim_ears_detection_length_mm() == 1.0`.
- Invalid negative, non-finite, non-numeric, or boolean `brim_ears_detection_length` values return `SliceError::InvalidInput` naming the option.
- Pipeline/G-code tests prove the option reaches generated brim paths and emitted `;BRIM:` comments.
- Existing brim-ear angle tests continue to pass.
- `cargo test -p ares-core --lib`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the Rust LOC gate pass.
