# Consume Small Perimeter Speed Design

## Goal

Consume OrcaSlicer `small_perimeter_threshold` and `small_perimeter_speed` in Ares speed planning so existing external perimeter paths can emit slower G-code feedrates when their path length is within the configured small-perimeter threshold.

This slice adds concrete slicing/G-code behavior. It does not add new option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2039-2067`
  - `outer_wall_speed` default is `60` mm/s.
  - `small_perimeter_speed` is a float-or-percent over `outer_wall_speed`, defaults to `50%`, and `0` means auto.
  - `small_perimeter_threshold` is a non-negative float in mm and defaults to `0`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1191-1192`
  - `PrintRegionConfig` option tuple entries for `small_perimeter_speed` and `small_perimeter_threshold`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5734-5746`
  - Orca computes a small perimeter speed when `speed == -1` and `loop.length() <= SMALL_PERIMETER_LENGTH(m_config.small_perimeter_threshold.value)`.
  - If configured `small_perimeter_speed == 0`, Orca uses `outer_wall_speed * 0.5`; otherwise it resolves the float-or-percent over `outer_wall_speed`.
- `OrcaSlicer/src/libslic3r/libslic3r.h:84`
  - `SMALL_PERIMETER_LENGTH(LENGTH)` converts the configured threshold into a maximum loop length with `((LENGTH) / SCALING_FACTOR) * 2 * PI`.

## Current Ares Boundary

Ares already has:

- `SliceOptions::speed_options()` building `SpeedOptions` from existing option values.
- `generate_toolpath_moves()` producing one travel move to a path start followed by print moves for that path, with closed external perimeters returning to the first point.
- `generate_speed_moves()` and `speeds/volumetric.rs` assigning configured speed and applying filament volumetric caps before G-code formatting.
- G-code `;SPEED:print:external_perimeter:<feedrate>` comments that make feedrate changes observable in tests.

This slice belongs in `ares-core` speed planning, not option metadata generation.

## Design

Use concrete module splits so the repo's 400 LOC rule remains satisfied:

- Move the existing `SpeedOptions` struct and its impl from `crates/ares-core/src/speeds.rs` into a new `crates/ares-core/src/speeds/config.rs`, then re-export it from `speeds.rs`. This is a mechanical ownership split before adding fields.
- Add small perimeter span logic under `crates/ares-core/src/speeds/small_perimeter.rs`.
- Add option parsing for these two runtime options under `crates/ares-core/src/options/small_perimeter.rs`.
- Keep `crates/ares-core/src/options.rs` as the public aggregation point. It may add only the new module declaration and a compact call from the existing `speed_options()` path.

`SpeedOptions` will carry two new resolved values:

- `small_perimeter_threshold_mm`, default `0.0`.
- `small_perimeter_speed_mm_s`, default `outer_wall_speed * 0.5`.

`SliceOptions::speed_options()` will parse:

- `small_perimeter_threshold` as a non-negative numeric option with default `0.0`.
- `small_perimeter_speed` as a non-negative number or percent over resolved `outer_wall_speed`, with default `outer_wall_speed * 0.5`; explicit `0` also resolves to `outer_wall_speed * 0.5`.

During speed generation, Ares will infer path spans from the existing flattened extrusion moves:

- A travel move starts a new span at its point.
- Following print moves up to the next travel belong to the same span.
- Print moves that appear before the first travel keep their existing speeds and are not eligible for the small-perimeter rule. Existing speed unit tests construct this print-only shape, but Ares' path generator emits a travel before each generated path, so generated pipeline/G-code behavior is unaffected.
- Span length is the sum of distances from the travel point through all print moves.
- Ares stores path coordinates in mm, so it implements Orca's `SMALL_PERIMETER_LENGTH` conversion as `small_perimeter_threshold_mm * 2.0 * PI`.
- If every print move in the span has `PrintPathRole::ExternalPerimeter` and the span length is `<= small_perimeter_threshold_mm * 2.0 * PI`, those print moves use `small_perimeter_speed_mm_s` before the existing volumetric cap is applied.
- Travel moves, internal perimeters, sparse infill, skirt, brim, bridge, and internal bridge keep their existing configured speeds.

`small_perimeter_threshold = 0` preserves current behavior for ordinary non-zero perimeter paths.

### Speed Ordering

For an eligible external perimeter span, Ares will select the small perimeter speed before the existing volumetric cap is applied.

The order is:

1. Select the existing base speed for the move using current Ares rules, including first-layer speeds and `slow_down_layers`.
2. If the move is an external perimeter print move inside a small-perimeter span, replace that base speed with `small_perimeter_speed_mm_s`.
3. Apply the existing filament volumetric cap to the selected speed.
4. Preserve existing acceleration and jerk selection.

This means small perimeter speed intentionally overrides first-layer external perimeter speed and `slow_down_layers` for matching external perimeter spans, while the volumetric cap can still reduce it. Non-matching spans keep the existing configured external perimeter speed path.

### Orca `speed == -1` Mapping

In the cited Orca `GCode.cpp` boundary, `speed == -1` means the extrusion loop has not already been given an explicit per-loop speed override, so region/config speed selection is allowed. Ares does not currently carry explicit per-loop speed overrides in `ExtrusionMove` or `SpeedMove`; all current external perimeter speed selection comes from `SpeedOptions` at speed-planning time. Therefore, every current Ares external perimeter span is equivalent to Orca's configurable-speed path for this slice. If Ares later adds explicit per-path speed overrides, those overrides must become the equivalent exclusion from this small-perimeter rule.

## Included Behavior

- Existing external perimeter paths can emit a lower feedrate when the threshold is above their path length.
- Percent values such as `"25%"` are resolved over `outer_wall_speed`.
- Numeric values such as `20` are accepted as mm/s.
- Explicit `small_perimeter_speed: 0` uses auto speed `outer_wall_speed * 0.5`.
- Negative threshold or negative speed is rejected by `SliceOptions::speed_options()`.
- Existing volumetric cap still applies after the small perimeter speed is selected.

## Deferred Behavior

- Full Orca `ExtrusionLoop` clipping, seam behavior, and loop ordering.
- Applying this setting to generated support loops or bridge/support pipeline behavior Ares does not yet own.
- New option metadata, generated config classes, UI behavior, or additional crates.
- Retrofitting path identifiers into extrusion moves. This slice uses the existing travel-delimited span structure.

## Acceptance Criteria

- Unit or pipeline tests prove default threshold `0` leaves ordinary external perimeter feedrate unchanged.
- Tests prove a rectangular external perimeter of length `16.0` mm is not affected by a threshold below `16.0 / (2.0 * PI)` and is affected by a threshold above `16.0 / (2.0 * PI)`.
- Tests prove `"25%"` resolves over `outer_wall_speed`.
- Tests prove `small_perimeter_speed: 0` resolves to half `outer_wall_speed`.
- Tests prove internal perimeter or other non-external roles are not affected.
- Tests prove print moves before the first travel preserve their existing speed.
- Tests prove invalid negative values are rejected.
- `cargo test -p ares-core --lib`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repo LOC check pass.

## Docs Impact

No user-facing docs are required beyond this source-cited spec and the implementation plan. The change consumes existing Orca option keys in runtime behavior and does not add CLI flags, WASM APIs, public command syntax, or roadmap changes.

## Safety

The change is local to option parsing and speed planning. It does not alter geometry generation, extrusion amounts, file I/O, CLI behavior, WASM bindings, or G-code formatting syntax beyond existing speed feedrate values.
