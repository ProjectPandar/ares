# Consume Overhang Speed Behavior Design

## Goal

Consume already registered Orca overhang wall/speed options as concrete slicing and G-code behavior in `ares-core`, instead of adding more option metadata. This slice ports the first executable path from Orca's overhang perimeter detection and overhang speed selection into Ares' current rectangular contour pipeline.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1153` declares `detect_overhang_wall`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1171-1175` declares `enable_overhang_speed` and `overhang_1_4_speed` through `overhang_4_4_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1500-1570` defines default `enable_overhang_speed = true` and `overhang_*_speed = 0`, with each speed using `outer_wall_speed` as `ratio_over`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:107-201` and `373-460` split unsupported perimeter spans into `erOverhangPerimeter` when `detect_overhang_wall` is enabled and lower slices are available.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6418-6460` treats `erOverhangPerimeter` as bridge-speed fallback for base speed selection.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6587-6641` consumes `enable_overhang_speed` and the `overhang_*_speed` values for bridge/perimeter paths after the first layer.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5816-5836` excludes overhang/bridge paths from small-perimeter speed substitution.

## Ares Boundary

- `crates/ares-core/src/perimeters.rs`: add the smallest runtime overhang role assignment needed by current Ares geometry. When `detect_overhang_wall` is true, the current layer is not the first entry in the `generate_perimeters` input slice, and the current rectangular external perimeter has no positive-area axis-aligned rectangular overlap with any contour bounds from the immediately preceding input layer, emit it as an overhang perimeter role. Edge-only or point-only contact is not support.
- `crates/ares-core/src/print_paths.rs`, `crates/ares-core/src/moves.rs`, `crates/ares-core/src/extrusions/options.rs`, and `crates/ares-core/src/extrusion_entity.rs`: carry the overhang perimeter role through print paths, moves, extrusion role mapping, and extrusion width/flow as a wall/bridge-like role.
- `crates/ares-core/src/options/speed.rs` and `crates/ares-core/src/speeds/*`: parse `enable_overhang_speed` and `overhang_4_4_speed`, then apply the overhang speed to `PrintPathRole::OverhangPerimeter` on non-first layers. `overhang_4_4_speed` is selected because this slice only detects fully unsupported rectangular perimeter loops, matching the 75%-100% bucket. `0` preserves Orca's "use original wall speed" behavior by falling back to `bridge_speed`.
- `crates/ares-core/src/options/flow_ratios.rs`: parse `overhang_flow_ratio` with the same numeric-or-string, default `1.0`, inclusive `0.0..=2.0` contract as other Orca flow ratios. Validate it when present even if `set_other_flow_ratios` is false; apply it only when `set_other_flow_ratios` is true.
- `crates/ares-core/src/gcode.rs`: existing speed moves and role comments must show the changed feedrate in generated G-code; no separate G-code stage is added.

## Included Behavior

- `detect_overhang_wall: true` causes a second-or-later rectangular external perimeter that has no positive-area rectangular overlap with the immediately preceding layer's contour bounds to become `overhang_perimeter`.
- The immediately preceding layer is the previous `LayerContours` item in the `generate_perimeters` input slice, not the previous non-empty layer by search.
- Positive-area overlap means `min(current.max_x, lower.max_x) > max(current.min_x, lower.min_x)` and `min(current.max_y, lower.max_y) > max(current.min_y, lower.min_y)`. If either comparison is equality, the rectangles only touch at an edge or point and the current perimeter is unsupported for this slice.
- `detect_overhang_wall: false` preserves the existing `external_perimeter` role for the same geometry.
- `PrintPathRole::OverhangPerimeter` emits `overhang_perimeter` in comments and maps to `ExtrusionRole::OverhangPerimeter`.
- Overhang perimeter extrusion uses the external wall width, participates in `overhang_flow_ratio` only when `set_other_flow_ratios` is enabled, and otherwise preserves current extrusion behavior.
- `enable_overhang_speed: false` leaves overhang perimeter speed at the base bridge-speed fallback.
- `enable_overhang_speed: true` plus positive numeric or percent `overhang_4_4_speed` changes overhang perimeter feedrate on non-first layers; percent values resolve over `outer_wall_speed`, matching the source option's ratio base.
- `overhang_4_4_speed: 0` keeps Orca's fallback behavior and does not force an invalid zero feedrate.
- Small perimeter speed does not override `overhang_perimeter`.

## Deferred Behavior

- Partial perimeter clipping against lower slices, grown lower-slice series, Arachne extrusion splitting, fuzzy skin interactions, steep-overhang reversal, and segment-level overlap percentage estimation from `PerimeterGenerator.cpp` are deferred.
- `overhang_1_4_speed`, `overhang_2_4_speed`, and `overhang_3_4_speed` are parsed/validated only if needed by the plan, but not selected by the current fully unsupported rectangular detector.
- `slowdown_for_curled_perimeters`, curled perimeter quality estimation, bridge object/raft exceptions, scarf joint speed, volumetric-derived reference speed, support transition behavior, counterbore behavior, and full bridge/perimeter `estimate_extrusion_quality` parity are deferred.
- Full non-rectangular polygon intersection is deferred until Ares ports the corresponding upstream polygon clipping boundary.

## Docs Impact

Update `docs/roadmap.md` to record that M43 no longer defers all overhang speed behavior: this slice consumes the first `detect_overhang_wall` plus `enable_overhang_speed`/`overhang_4_4_speed` runtime path for fully unsupported rectangular perimeter loops. No user-facing CLI, WASM, or API documentation changes are required because the public byte-slicing API and option names do not change.

## Acceptance Criteria

- A focused perimeter test proves second-layer unsupported rectangular external walls become `PerimeterRole::Overhang` only when `detect_overhang_wall` is true.
- A focused perimeter test proves edge-only contact with the immediately preceding layer counts as unsupported and that an empty intervening previous layer is not skipped.
- Print-path, extrusion-role, speed, and pipeline/G-code tests prove the `overhang_perimeter` role reaches generated output.
- A G-code test proves `overhang_4_4_speed` changes the overhang perimeter feedrate on a non-first layer and that disabling `enable_overhang_speed` preserves bridge-speed fallback.
- A G-code test proves `overhang_flow_ratio` scales overhang perimeter extrusion only when `set_other_flow_ratios` is true and is parsed/validated even when the gate is false.
- A speed test proves small-perimeter speed does not override overhang perimeter speed.
- Invalid `detect_overhang_wall`, `enable_overhang_speed`, `overhang_4_4_speed`, or `overhang_flow_ratio` values produce `SliceError::InvalidInput`.
- No new crates, dependencies, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or WASM-incompatible logic are added.
- All edited Rust files remain at or below 400 LOC; if a touched file would exceed 400 LOC, split narrowly by existing module responsibility.
