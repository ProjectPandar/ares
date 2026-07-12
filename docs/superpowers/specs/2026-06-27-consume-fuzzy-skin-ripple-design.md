# Consume Fuzzy Skin Ripple Design

## Scope

Consume OrcaSlicer's ripple fuzzy-skin options into Ares' existing external-perimeter fuzzy skin runtime. This is a concrete behavior slice, not option metadata: when `fuzzy_skin_noise_type` is `"ripple"`, Ares must generate deterministic sine-wave displaced external perimeter points, which then flow through print paths, moves, extrusion, speeds, and G-code coordinates.

The source boundary is:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:65-72` for `NoiseType::Ripple`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1112,1117-1119` for `fuzzy_skin_noise_type`, `fuzzy_skin_ripples_per_layer`, `fuzzy_skin_ripple_offset`, and `fuzzy_skin_layers_between_ripple_offset`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3491-3515,3545-3576` for option values, defaults, and ranges.
- `OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:70-220,296-300,434-441,507-513` for ripple phase, anchor, closed-polyline displacement, config wiring, and effect-equivalence fields.

## Current Behavior

Ares already consumes `fuzzy_skin`, `fuzzy_skin_thickness`, `fuzzy_skin_point_distance`, and `fuzzy_skin_first_layer` in `crates/ares-core/src/perimeters/fuzzy_skin.rs`. The current runtime always uses a deterministic classic random displacement shell. The registry already exposes the ripple-related option metadata, but the runtime ignores these option values.

## Required Behavior

- Parse `fuzzy_skin_noise_type` in `FuzzySkinConfig`.
  - Missing value defaults to `"classic"`.
  - `"classic"` keeps the existing deterministic random displacement behavior.
  - `"ripple"` selects the new ripple displacement behavior.
  - Other Orca noise strings remain deferred and must return `SliceError::InvalidInput` if supplied at runtime, because Ares does not yet implement their noise modules.
- Parse `fuzzy_skin_ripples_per_layer` as an integer with Orca default `15` and minimum `1`.
- Parse `fuzzy_skin_ripple_offset` as a percent value with Orca default `50%`, inclusive range `0..=100`, and numeric/string forms such as `50`, `"50"`, and `"50%"`.
- Parse `fuzzy_skin_layers_between_ripple_offset` as an integer with Orca default `1` and minimum `1`.
- For closed external perimeter point lists selected by the existing fuzzy-skin type/first-layer/thickness/point-distance gates, implement the Orca ripple branch:
  - compute total perimeter length,
  - find the leftmost `y = 0` crossing as the visual anchor, falling back to the vertex with smallest absolute y,
  - compute `anchor_arc_mm` as the cumulative arc length of the closest point on the perimeter to that visual anchor,
  - compute layer-group phase as `floor(layer_id / layers_between_ripple_offset) * ripple_offset / 100 * 2pi`,
  - resample along the closed perimeter at `fuzzy_skin_point_distance`,
  - for each sample at cumulative arc length `arc_mm`, compute `sample_phase = ripples_per_layer * 2pi * (arc_mm - anchor_arc_mm) / perimeter_mm + 2pi + phase_shift_rad`,
  - displace each sampled point along the segment perpendicular by `sin(sample_phase) * fuzzy_skin_thickness`.
- If Orca `FuzzySkin.cpp` comments diverge from the executable formula, follow the cited executable code for this slice. In particular, the current source code adds `2pi + phase_shift_rad` at the anchor even though an adjacent comment describes a peak-forming phase offset.
- Keep the existing classic behavior and existing effective-disable gates unchanged.
- Preserve platform neutrality: no file I/O, terminal behavior, threads, random devices, native UI, or OpenGL in `ares-core`.

## Non-Goals

- Do not implement Perlin, Billow, RidgedMulti, Voronoi, or other non-ripple coherent noise modules.
- Do not implement Arachne `FuzzySkinMode::Extrusion` or `FuzzySkinMode::Combined` width-changing behavior.
- Do not fuzzify holes or internal wall loops beyond Ares' existing external-path compatibility shell.
- Do not add dependencies.
- Do not attempt full Orca binary E2E geometry parity in this slice.

## Acceptance Criteria

- Focused RED/GREEN uses nextest:
  - `cargo nextest run -p ares-core fuzzy_skin`
- Tests prove ripple changes external perimeter coordinates differently from classic mode for the same fuzzy skin thickness and point distance.
- Tests prove `fuzzy_skin_ripples_per_layer` changes the ripple wave frequency along the same closed perimeter.
- Tests prove `fuzzy_skin_ripple_offset` and `fuzzy_skin_layers_between_ripple_offset` change coordinates across layers according to Orca's layer-group phase rule.
- Tests include at least one deterministic rectangle fixture with tightly bounded numeric assertions for the first generated ripple points, including an anchor sample that follows the executable `2pi + phase_shift_rad` formula.
- Tests prove invalid runtime ripple options fail with `SliceError::InvalidInput`.
- Existing fuzzy-skin behavior remains covered:
  - classic default still fuzzifies after the first layer,
  - first-layer gate still works,
  - disabled/effective-disabled cases still preserve geometry.
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run -p ares-core fuzzy_skin`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust files stay at or below 400 LOC.

## Documentation

Update `docs/roadmap.md` after implementation review to record that `fuzzy_skin_noise_type = ripple`, `fuzzy_skin_ripples_per_layer`, `fuzzy_skin_ripple_offset`, and `fuzzy_skin_layers_between_ripple_offset` now reach concrete external-perimeter/G-code behavior, while other noise modes and Arachne width modes remain deferred.
