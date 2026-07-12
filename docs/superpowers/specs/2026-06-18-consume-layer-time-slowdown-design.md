# Consume Layer-Time Slowdown Design

## Goal

Consume OrcaSlicer's layer-time cooling slowdown options in Ares speed planning so `slow_down_for_layer_cooling`, `slow_down_layer_time`, and `slow_down_min_speed` change emitted print feedrates instead of remaining option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1510` declares `slow_down_for_layer_cooling` as a `ConfigOptionBools` print setting.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1542` declares `slow_down_min_speed` as a `ConfigOptionFloats` print setting.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1559` declares `slow_down_layer_time` as a `ConfigOptionFloats` print setting.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1772-1777` defines the slowdown enable option and its default `true`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4706-4713` defines the minimum print speed default `10`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5629-5637` defines the layer-time threshold default `5`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:346-348` copies the three options into per-extruder cooldown adjustments.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:425-438` starts layer-time accounting at the first extrusion speed block and marks eligible blocks adjustable, excluding external perimeters when `dont_slow_down_outer_wall` is enabled.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:440-474` calculates move lengths, current move time, and `time_max` capped by `slow_down_min_speed`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:547-550` excludes moves before the first extrusion from layer-time totals.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:560-689` calculates layer slowdown when the estimated layer time is shorter than `slow_down_layer_time`.

## Ares Destination Boundary

- `crates/ares-core/src/options/slow_down_layers.rs` parses the first configured value for the three options into Ares speed settings.
- `crates/ares-core/src/speeds/config.rs` carries the parsed settings in `SpeedOptions`.
- `crates/ares-core/src/speeds/volumetric.rs` applies the layer-time slowdown after configured role speeds, small-perimeter speed selection, and volumetric speed caps are known for a layer.
- Tests live in `crates/ares-core/src/options/tests/slow_down_layers.rs`, `crates/ares-core/src/speeds/slow_down_layers/tests.rs`, and `crates/ares-core/src/pipeline/tests/slow_down_layers.rs`.

## Included Behavior

- Missing `slow_down_for_layer_cooling` defaults to enabled, matching Orca's default.
- Missing `slow_down_layer_time` defaults to `5.0` seconds.
- Missing `slow_down_min_speed` defaults to `10.0` mm/s.
- Scalar and vector values are accepted for these Orca vector options; Ares consumes the first entry because the current slicer path is single-extruder.
- If slowdown is disabled, the speed pipeline keeps current behavior.
- If the estimated layer time from first extrusion onward is already at least `slow_down_layer_time`, the speed pipeline keeps current behavior.
- If slowdown is enabled and the estimated layer time is shorter than `slow_down_layer_time`, adjustable print moves are slowed to the requested time unless `slow_down_min_speed` is reached first.
- `slow_down_min_speed` is a lower bound for slowdown. If the requested target cannot be reached without dropping below it, adjustable print moves stop at that minimum.
- Existing `dont_slow_down_outer_wall` also exempts external perimeter print moves from this layer-time slowdown, matching Orca's `TYPE_EXTERNAL_PERIMETER` adjustable-line filter.
- Travel moves and non-adjustable print moves contribute to layer time after the first extrusion but are not slowed.
- The behavior is visible in emitted G-code feedrates through the existing `;SPEED:` comments and `G1 ... F...` commands.

## Slowdown Calculation

Ares ports the single-extruder form of Orca's non-proportional adjustable-feedrate slowdown from `CoolingBuffer.cpp:560-689`.

- Build one layer-local line record while scanning speed moves in order. Every move updates the current XY position. Moves before the first print extrusion keep their position effect but contribute zero layer time. Move time is `XY length / speed_mm_s` for moves with positive XY length and positive speed.
- The first counted move after excluded pre-extrusion moves uses the current position left by those excluded moves as its start point. The first move of a later layer uses the current position carried from the previous layer, matching the existing Ares extrusion and volumetric speed state model.
- Mark a line adjustable when it is a print move, has positive duration, and is not an external perimeter excluded by `dont_slow_down_outer_wall`.
- Compute the total layer time from all layer-local records. If no records are adjustable, or total time is at least `slow_down_layer_time * 1.001`, do not change speeds.
- Compute each adjustable record's maximum time at `slow_down_min_speed`. If the target layer time cannot be reached without crossing the minimum speed, slow every adjustable record to `slow_down_min_speed`.
- Otherwise, sort adjustable records by decreasing current speed and lower the fastest adjustable records first. For each speed band, lower records above the next band to that next band, or solve the exact equalized feedrate needed to consume the remaining time stretch before the next band.
- Apply the resulting speed only to the corresponding speed moves. Travel, pre-extrusion moves, and non-adjustable print moves keep their existing speed.

## Option Parsing Rules

- `slow_down_for_layer_cooling` accepts a scalar boolean or a non-empty boolean array and consumes the first entry. Missing defaults to `true`. Numbers, strings, null, objects, empty arrays, and arrays whose first entry is not boolean are invalid.
- `slow_down_layer_time` accepts a scalar number, numeric string, or non-empty array whose first entry is a number or numeric string. Missing defaults to `5.0`. Values must be finite and in `0.0..=1000.0`, matching Orca's configured max. A value of `0.0` disables additional layer-time stretching because no layer can be shorter than zero seconds. Negative values, values above `1000.0`, non-numeric strings, non-finite strings, null, objects, empty arrays, and arrays whose first entry is invalid are rejected.
- `slow_down_min_speed` accepts a scalar number, numeric string, or non-empty array whose first entry is a number or numeric string. Missing defaults to `10.0`. Values must be finite and non-negative. A value of `0.0` means unlimited slowdown for the adjustable records. Negative values, non-numeric strings, non-finite strings, null, objects, empty arrays, and arrays whose first entry is invalid are rejected.

## Pipeline Ordering

Speed composition remains source-shaped and deterministic:

1. Existing configured role speed selection and `slow_down_layers` first-layer interpolation choose the base speed.
2. Existing small-perimeter speed selection may replace eligible external perimeter speeds.
3. Existing volumetric caps lower speeds that exceed filament volumetric flow limits.
4. New layer-time slowdown stretches the capped layer speed plan when `slow_down_for_layer_cooling` is enabled.
5. Existing acceleration and jerk attachment stays unchanged and uses the final move kind, role, and first-layer status.

## Deferred Behavior

- Full Orca `CoolingBuffer` G-code post-processing is not ported in this slice.
- Multi-extruder per-range cooldown ordering is deferred until Ares has multi-extruder path ownership in the core pipeline.
- Fan interpolation from `fan_cooling_layer_time`, `fan_min_speed`, `fan_max_speed`, and `reduce_fan_stop_start_freq` is deferred.
- Support-interface, ironing, auxiliary fan, arc, wipe, and custom G-code cooldown marker behavior is deferred.
- This slice does not add new option registry metadata, crates, dependencies, CLI flags, WASM bindings, or independent Ares pipeline features.
- No additional roadmap, architecture, CLI, WASM, or registry documentation updates are required beyond this source-cited design and its implementation plan because the change is internal core speed-planning behavior for already documented Orca options.

## Acceptance Criteria

- Unit tests prove option parsing defaults, valid first-entry vector consumption, and invalid boundary values.
- Speed tests prove layer-time slowdown lowers feedrates, respects `slow_down_min_speed`, does nothing when disabled, and honors `dont_slow_down_outer_wall`.
- Speed tests prove fastest adjustable records are lowered before slower records for the single-extruder non-proportional algorithm.
- Speed tests prove no-op behavior when estimated layer time already meets `slow_down_layer_time`.
- Speed tests prove pre-first-extrusion moves update position but contribute zero layer time, while post-first-extrusion travel and non-adjustable print moves contribute time without being slowed.
- Speed tests prove `slow_down_min_speed = 0.0` uses Orca's unlimited proportional slowdown branch for adjustable records.
- Pipeline/G-code tests prove the options change emitted feedrates through the existing G-code formatting path.
- Existing `slow_down_layers`, small-perimeter, volumetric cap, acceleration, jerk, and fan tests continue to pass.
- All touched Rust files remain at or below 400 LOC.

## Verification

- Targeted red/green tests for option parsing, speed planning, and pipeline G-code.
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- Rust LOC gate: `find crates -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; bad=1 } END { exit bad }'`
