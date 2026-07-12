# Consume Volumetric Rate Slope Design

## Goal

Consume the existing OrcaSlicer `max_volumetric_extrusion_rate_slope`, `max_volumetric_extrusion_rate_slope_segment_length`, and `extrusion_rate_smoothing_external_perimeter_only` options in Ares speed generation so they change concrete print feedrates instead of remaining metadata-only options.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4610-4648` defines the three extrusion-rate smoothing options, defaults, units, and bounds.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1362-1364` declares the options in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/GCode/PressureEqualizer.cpp:61-66` enables pressure-equalizer slope limits only when `max_volumetric_extrusion_rate_slope > 0`, reads segment length, and reads the external-perimeter-only gate.
- `OrcaSlicer/src/libslic3r/GCode/PressureEqualizer.cpp:819-830` limits positive extrusion-rate transitions and skips non-external roles when `extrusion_rate_smoothing_external_perimeter_only` is enabled.

## Ares Destination Boundary

- Parse option values in `crates/ares-core/src/options/volumetric_speed.rs` and wire them through `crates/ares-core/src/options/speed.rs`.
- Store them on `crates/ares-core/src/speeds/config.rs` / `crates/ares-core/src/speeds/config/accessors.rs`.
- Apply the first concrete runtime behavior in `crates/ares-core/src/speeds/volumetric.rs`, after existing volumetric cap selection and before layer-time slowdown.
- Add focused option tests in `crates/ares-core/src/options/tests/filament_max_volumetric_speed.rs`, split speed-stage tests under `crates/ares-core/src/speeds/tests/volumetric_rate_smoothing.rs`, and G-code-facing pipeline tests in `crates/ares-core/src/pipeline/tests/filament_max_volumetric_speed.rs`.

## Included Behavior

- `max_volumetric_extrusion_rate_slope` defaults to `0.0`; `0.0` disables the new smoothing behavior.
- `max_volumetric_extrusion_rate_slope` accepts finite JSON numbers and numeric strings in `[0.0, +inf)` in `mm3/s2`. Explicit JSON `0`, `"0"`, and `"0.0"` are accepted and disable smoothing. Negative values, non-finite strings, non-numeric strings, arrays, objects, booleans, null, or empty strings are rejected at option parsing.
- `max_volumetric_extrusion_rate_slope_segment_length` defaults to Orca's `3.0` and accepts finite JSON numbers and numeric strings in `[0.5, 5.0]`. It is parsed and stored now so the option is consumed, but this slice does not split G-code moves into smoothing subsegments.
- `extrusion_rate_smoothing_external_perimeter_only` defaults to `false` and accepts booleans only.
- When smoothing is enabled, Ares limits only positive adjacent print-flow jumps: a print move's effective volumetric rate may not exceed `previous_print_rate + slope * previous_print_duration`.
- The first print move has no previous-print baseline, so it bypasses smoothing and initializes the smoothing state with its effective post-cap speed and volumetric rate. Travel-only prefixes do not create a smoothing baseline.
- The limiter uses the same filament-area and move-distance math as the existing volumetric cap path, so lower extrusion-per-mm or longer previous-duration moves allow higher next speeds.
- Travel moves never receive smoothing. They may separate print moves spatially, but the previous print rate remains the smoothing baseline for the next print move.
- With `extrusion_rate_smoothing_external_perimeter_only = true`, smoothing applies only to `ExternalPerimeter` and `OverhangPerimeter` roles. Other print roles retain their configured/capped speed and do not update the smoothing baseline, matching the external-feature-only intent for this forward-only Ares slice.
- Existing `filament_max_volumetric_speed`, adaptive volumetric speed, small-perimeter speed, `slow_down_layers`, layer-time slowdown, acceleration, jerk, and fan behavior remain composed with the new limiter.

## Deferred Behavior

- Full Orca `PressureEqualizer` G-code parsing and rewriting is deferred.
- Negative-rate deceleration smoothing is deferred.
- Splitting one G-code movement into acceleration, steady, and deceleration subsegments using `max_volumetric_extrusion_rate_slope_segment_length` is deferred.
- Per-role future/backward passes, bridge/ironing exclusions beyond Ares' current roles, opened extrude-set-speed blocks, absolute/relative E post-processing, arc-fitting disabling, multi-extruder slope state, and full Orca binary E2E parity are deferred.
- No new crate, dependency, UI, filesystem, terminal, OpenGL, or WASM-incompatible behavior is included.
- Any touched Rust source or test file that would exceed 400 LOC must be split before or during implementation.

## Acceptance Criteria

- A RED nextest run proves `max_volumetric_extrusion_rate_slope` and `extrusion_rate_smoothing_external_perimeter_only` do not yet reach `SpeedOptions`.
- A RED nextest run proves `max_volumetric_extrusion_rate_slope_segment_length` does not yet reach `SpeedOptions`.
- A RED nextest run proves a high-flow print move after a low-flow print move is not yet slowed by the slope option.
- After implementation, focused nextest runs prove parsing, speed-stage smoothing, and G-code feedrate behavior.
- `SpeedOptions` exposes `max_volumetric_extrusion_rate_slope_mm3_s2`, `max_volumetric_extrusion_rate_slope_segment_length_mm`, and `extrusion_rate_smoothing_external_perimeter_only`.
- Default `max_volumetric_extrusion_rate_slope` resolves to `0.0`; explicit numeric and numeric-string zero values are accepted; positive numeric and numeric-string values are accepted; negative, non-finite, non-numeric, empty, array, object, boolean, and null values return `SliceError::InvalidInput`.
- Default `max_volumetric_extrusion_rate_slope_segment_length` resolves to `3.0`, valid numeric and numeric-string values in `[0.5, 5.0]` are accepted, and invalid values outside that range or with non-numeric/non-finite forms return `SliceError::InvalidInput`.
- With smoothing disabled, existing feedrates are unchanged.
- With smoothing enabled, a high-flow external perimeter after a low-flow external perimeter has a lower feedrate than the disabled case.
- A first print move without a previous print baseline keeps its configured/capped speed and initializes smoothing state.
- With external-only enabled, a high-flow sparse infill move keeps the disabled feedrate while an external perimeter move is smoothed.
- With external-only enabled, skipped non-external print moves do not update the smoothing baseline used by the next external or overhang print move.
- Invalid smoothing option values produce `SliceError::InvalidInput` before byte output.
- `cargo fmt --check`, `cargo nextest run -p ares-core volumetric`, `cargo nextest run -p ares-core filament_max_volumetric_speed`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust file LOC checks pass.

## Safety And Rollback

The change is confined to `ares-core` option parsing and speed generation. It does not introduce file I/O or platform-specific behavior. Rollback removes the new option fields and smoothing pass while leaving existing volumetric cap and slowdown behavior intact.
