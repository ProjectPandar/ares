# Consume Infill Combination Design

## Goal

Consume OrcaSlicer `infill_combination` and `infill_combination_max_layer_height` in Ares sparse infill generation so existing option metadata changes layer infill paths, print paths, and extrusion output instead of remaining registry-only data.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1132-1134` declares `infill_combination` and `infill_combination_max_layer_height` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3853-3860` registers `infill_combination` as a boolean, defaults it to `false`, and describes automatically combining sparse infill over several layers while walls keep the original layer height.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3972-3984` registers `infill_combination_max_layer_height` as `FloatOrPercent`, defaults it to `100%`, and describes deriving the number of combined sparse-infill layers from max combined height divided by layer height.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3914-4050` implements `PrintObject::combine_infill()`: skip first print layer, skip disabled combination or zero sparse infill density, cap the combined sparse-infill height by nozzle diameter and `infill_combination_max_layer_height`, remove combined sparse infill from lower layers, and append combined infill with accumulated thickness to the upper target layer.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:58-63` notes that later infill preparation must tolerate `combine_infill()` turning some fill surfaces into void surfaces.

## Ares Destination Boundary

- Runtime option parsing belongs in `crates/ares-core/src/options.rs::SliceOptions::infill_options()` because sparse density, pattern, direction, line width, and minimum area are already parsed there.
- `infill_combination_max_layer_height` parsing belongs in `crates/ares-core/src/options/infill.rs` as a focused FloatOrPercent helper over nozzle diameter.
- Typed storage belongs in `crates/ares-core/src/options/infill.rs::InfillOptions`.
- Layer-level behavior belongs in `crates/ares-core/src/infills.rs`, where Ares already creates `LayerInfills` from `LayerContours`.
- Pipeline integration belongs in `crates/ares-core/src/pipeline.rs` and `crates/ares-core/src/pipeline/test_support.rs`, because combination needs `Layer::height()` in addition to contours.
- Extrusion-height propagation belongs in `LayerInfills` and `InfillPath`, then `print_paths`, `moves`, and `extrusions`, so only combined sparse infill uses combined layer height while perimeters and other print roles keep the physical layer height.

## Included Behavior

- Parse `infill_combination` as a boolean with Orca default `false`.
- Parse `infill_combination_max_layer_height` as a non-negative FloatOrPercent over the first nozzle diameter, with Orca default `100%`.
- Treat a parsed max height of `0` as "use nozzle diameter", matching the upstream `> 0 ? min(max, nozzle) : nozzle` branch.
- When `infill_combination` is disabled, preserve current sparse infill paths and extrusion math.
- When sparse infill density is zero, preserve current empty sparse infill output.
- When enabled and sparse infill density is positive, skip layer `0` and combine later sparse infill layers into the upper target layer using the same layer-height accumulation rule as `PrintObject::combine_infill()`.
- Clear sparse infill paths from lower layers that are combined into a later target layer.
- Keep the target layer's sparse infill geometry as the Ares scaffold's current paths for that layer, but carry the accumulated sparse-infill thickness so downstream extrusion amount reflects the combined height.
- Preserve current sparse infill pattern behavior for the target layer: rectilinear/line/zigzag/crosshatch still rotate by Ares layer parity, aligned rectilinear/grid keep their existing layer-angle behavior.
- Keep wall, skirt, brim, bridge, travel, speed, acceleration, and G-code header behavior unchanged except for changed sparse-infill path counts and sparse-infill extrusion amounts.

## Deferred Behavior

- Full Orca multi-region `SurfaceCollection` intersection, `stInternalVoid` surfaces, solid infill combination at 100% sparse density, region-specific sparse/solid filament nozzle selection, area-threshold erasure beyond Ares' existing minimum sparse infill area, clearance offsets around combined surfaces, monotonic/honeycomb/lateral lattice behavior, adaptive layer-height interaction, raft-layer offset, support interaction, and UI behavior remain deferred.
- This slice does not add option registry metadata, new crates, dependencies, filesystem behavior, terminal behavior, OpenGL/viewer behavior, or an Ares-owned pipeline concept.

## Acceptance Criteria

- `SliceOptions::default().infill_options()` reports `infill_combination() == false` and `infill_combination_max_layer_height_mm() == 0.4` for the default 0.4 mm nozzle.
- `infill_combination: true` parses into `InfillOptions`, and invalid non-boolean values return `SliceError::InvalidInput` naming `infill_combination`.
- `infill_combination_max_layer_height` accepts absolute numbers and percent strings over nozzle diameter, rejects negative or non-numeric values, and maps `0` to the effective nozzle-diameter cap for combination.
- Direct infill tests prove enabled combination with 0.2 mm layers and a 0.4 mm cap clears layer `1` sparse infill, preserves layer `0`, and emits layer `2` sparse infill with 0.4 mm combined infill height.
- Direct infill tests prove disabled combination preserves per-layer sparse infill and each sparse path's physical layer height.
- Direct infill tests prove zero sparse density remains empty even when combination is enabled.
- Pipeline tests prove combined sparse infill is absent from lower combined layers, present on the target layer, and reaches print paths.
- Pipeline or G-code tests prove combined target-layer sparse infill extrusion is larger than the disabled equivalent because extrusion uses the combined sparse-infill height.
- Existing infill pattern, minimum sparse area, perimeter, extrusion, speed, and layer-aware G-code tests continue to pass.
- Verification must include focused red/green tests, `cargo test -p ares-core --lib`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository Rust LOC gate.
