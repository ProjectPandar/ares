# Consume Brim Ears Design

## Goal

Make the already-parsed `brim_type = "brim_ears"` option produce concrete first-layer brim-ear geometry instead of silently producing no brim paths.

## Upstream Boundary

This is a source-cited Rust rewrite slice of OrcaSlicer brim-ear option consumption:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:923-924` declares `PrintObjectConfig::brim_type` and `brim_width`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:925-926` declares `PrintObjectConfig::brim_ears_detection_length` and `brim_ears_max_angle`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1665-1693` in the Ares roadmap snapshot defines the FFF `brim_ears`, `brim_ears_max_angle`, and `brim_ears_detection_length` option metadata. Current Orca main has this block around `PrintConfig.cpp:1743-1772` after adjacent brim options moved.
- `OrcaSlicer/src/libslic3r/Brim.cpp:308-352` defines `make_brim_ears_auto(...)`, which decimates polygon points by `ear_detection_length`, selects convex or concave points using `brim_ears_max_angle`, and creates round local ear polygons.
- `OrcaSlicer/src/libslic3r/Brim.cpp:447-460` selects `btEar` into `use_auto_brim_ears`, treats it as both inner and outer brim-capable, and reads `brim_ears_detection_length` plus `brim_ears_max_angle`.
- `OrcaSlicer/src/libslic3r/Brim.cpp:536-554` uses `make_brim_ears_auto(...)` for outer and inner `btEar` brim area generation.

The existing project roadmap already records the metadata-only milestone for this upstream option set in `docs/roadmap.md` M45 and `docs/milestones/m45-print-config-brim-ear-registry.md`. This slice moves one step beyond that metadata boundary by giving `BrimType::BrimEars` executable first-layer geometry in the current Ares brim scaffold.

## Current Ares State

Ares already parses `brim_type` in `crates/ares-core/src/options/brim.rs` and exposes `BrimType::BrimEars`.

`crates/ares-core/src/brims.rs` currently generates geometry for:

- `auto_brim`, `outer_only`, and `outer_and_inner` as outer rectangular brim loops.
- `inner_only` and `outer_and_inner` as inner rectangular brim loops for holes.
- `no_brim` as no paths.

`BrimType::BrimEars` and `BrimType::Painted` are currently treated as deferred and produce no paths. That means an already-parsed user-visible option is not consumed into concrete slicing behavior.

## Ares Destination Boundary

Implement a conservative temporary Ares rectangle-scaffold approximation of the cited `Brim.cpp` `btEar` path inside the existing `brims` module:

- `brim_type = "brim_ears"` generates first-layer local square ear loops at the four corners of each outer rectangular contour.
- Ear loops use the existing `brim_width` and effective brim line width to determine loop count, matching the existing brim loop count behavior.
- The first ear loop around a corner is a square centered on the corner with radius equal to `brim_object_gap + effective_line_width`, clamped by `brim_width` in the same way as outer brim loops.
- Additional ear loops expand by the effective line width until `brim_width` is reached.
- Ears are emitted only for outer rectangular contours because the current Ares perimeter/brim scaffold is rectangle-based.
- Ears are emitted only on the first layer, consistent with existing brim generation.
- `brim_type = "painted"` remains deferred and produces no paths because Ares has no painted-brim input geometry boundary yet.
- `brim_ears`, `brim_ears_max_angle`, and `brim_ears_detection_length` option values remain preserved option values in this slice. They do not affect the scaffold until the full `Brim.cpp:308-352` sharp-corner detection boundary is ported.

This adds executable behavior for a currently empty parsed mode without adding new option metadata milestone files.

## Included Behavior

- `brim_type = "brim_ears"` with positive `brim_width` produces local ear brim paths on the first layer.
- Ear geometry reaches print paths, extrusion moves, speed moves, and G-code via the existing brim path pipeline.
- `brim_type = "painted"` continues to produce no brim paths.
- `brim_type = "no_brim"` continues to produce no brim paths.
- Existing `outer_only`, `inner_only`, `outer_and_inner`, `auto_brim`, object-gap, line-width, first-layer-only, and excessive-loop behavior remains unchanged.

## Deferred Behavior

This slice does not implement:

- Exact Orca `Brim.cpp` round mouse-ear polygon generation.
- `Brim.cpp:308-352` sharp-corner detection using `brim_ears_max_angle`.
- `Brim.cpp:321-329` detection radius/edge decimation using `brim_ears_detection_length`.
- `Brim.cpp:332-333` outer convex versus inner concave ear-point selection.
- `Brim.cpp:536-554` full inner-ear handling for hole contours.
- Per-object `brim_ears` enable/disable behavior beyond `brim_type = "brim_ears"`.
- Painted brim input geometry.
- Elephant-foot-compensated brim outline behavior.
- Brim combining behavior.
- Multi-object overlap clipping or object-by-object scheduling.
- New crates, dependencies, filesystem, UI, OpenGL, terminal, or native-only behavior.

## File Size Constraints

- Keep all touched Rust files at or below 400 LOC.
- `crates/ares-core/src/brims.rs` and `crates/ares-core/src/brims/tests.rs` are currently below the limit and can absorb this slice without splitting if the implementation stays compact.
- `crates/ares-core/src/pipeline/tests.rs` is already above 300 LOC. Register the pipeline/G-code regression as a new focused child module, `crates/ares-core/src/pipeline/tests/brim_ears.rs`, and keep the parent file change to a single `mod brim_ears;` declaration.
- Do not add option metadata milestone files.

## Test Strategy

- Add a focused `brims` test proving `BrimType::BrimEars` generates four local ear paths for a first-layer unit square with one brim loop.
- Add a focused `brims` test proving multi-loop ear generation uses `brim_width` and effective line width.
- Update the existing deferred brim test so `BrimEars` is no longer expected to be empty while `Painted` remains empty.
- Add a pipeline/G-code regression proving `brim_type = "brim_ears"` produces brim print-path and G-code output while `brim_type = "painted"` stays empty.
- Keep existing brim tests passing.

## Acceptance Criteria

- `brim_type = "brim_ears"` affects concrete generated brim geometry.
- Brim-ear paths reach downstream print path/G-code surfaces through existing pipeline stages.
- `painted` and `no_brim` remain explicit no-output modes.
- Existing brim modes retain their current output.
- No option metadata milestone files are added.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- All touched Rust source files stay at or below 400 LOC.
