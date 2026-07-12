# Consume fuzzy skin coherent noise options

## Source boundary

This slice is a source-cited Rust rewrite step for OrcaSlicer fuzzy-skin noise selection, not a new Ares pipeline feature.

Upstream source:
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:65-72` declares `NoiseType::{Classic, Perlin, Billow, RidgedMulti, Voronoi, Ripple}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1112,1114-1116` stores `fuzzy_skin_noise_type`, `fuzzy_skin_scale`, `fuzzy_skin_octaves`, and `fuzzy_skin_persistence` in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:202-210` maps the noise enum string values.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3491-3543` defines the UI/default/range metadata for `fuzzy_skin_noise_type`, `fuzzy_skin_scale`, `fuzzy_skin_octaves`, and `fuzzy_skin_persistence`.
- `OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:41-64` selects a libnoise module for Perlin, Billow, RidgedMulti, and Voronoi using `1 / cfg.noise_scale`, `cfg.noise_octaves`, and `cfg.noise_persistence` where applicable.
- `OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:296-333` applies non-ripple noise in `fuzzy_polyline` by sampling points along the closed path and displacing them by `noise->GetValue(x, y, slice_z) * cfg.thickness`.
- `OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:434-441` copies the coherent-noise parameters from print-region config into `FuzzySkinConfig`.

Rust destination:
- `crates/ares-core/src/perimeters.rs`
- `crates/ares-core/src/perimeters/fuzzy_skin.rs`
- `crates/ares-core/src/perimeters/fuzzy_skin_noise.rs`
- Create `crates/ares-core/src/perimeters/fuzzy_skin_coherent_noise.rs` for deterministic no-dependency coherent-noise helpers if this keeps `fuzzy_skin_noise.rs` focused and under the 400 LOC policy.
- Tests in `crates/ares-core/src/perimeters/tests/fuzzy_skin.rs`, a new `crates/ares-core/src/perimeters/tests/fuzzy_skin_coherent.rs`, `crates/ares-core/src/perimeters/tests.rs`, and `crates/ares-core/src/tests/fuzzy_skin_gcode.rs`.
- `docs/roadmap.md`.

## Current Ares state

Ares already parses and applies classic fuzzy skin and ripple fuzzy skin to current closed rectangular perimeter loops. `fuzzy_skin_noise_type = "perlin"`, `"billow"`, `"ridgedmulti"`, and `"voronoi"` are registered in the option metadata but currently rejected by the perimeter option parser. `fuzzy_skin_scale`, `fuzzy_skin_octaves`, and `fuzzy_skin_persistence` are registered in the option metadata but are not parsed into `FuzzySkinConfig` and do not affect generated geometry.

The current Ares perimeter boundary owns closed-polyline point displacement for external walls and the recently consumed `allwalls` internal rectangular wall loops. It does not own Arachne variable-width extrusion junctions, libnoise binary parity, painted-region polygon splitting, or hole topology.

## Runtime behavior

1. `fuzzy_skin_noise_type` must accept all already-registered Orca enum values: `classic`, `ripple`, `perlin`, `billow`, `ridgedmulti`, and `voronoi`.
2. `classic` and `ripple` behavior must remain unchanged except for any internal refactor needed to share dispatch.
3. `perlin`, `billow`, `ridgedmulti`, and `voronoi` must use the existing closed-polyline resampling path used by classic noise, then choose displacement from a deterministic coherent-noise module instead of classic uniform random noise.
4. Coherent noise must consume:
   - `fuzzy_skin_scale`, default `1.0`, valid range `0.1..=500.0`, used as the base feature size by evaluating noise at frequency `1 / scale`.
   - `fuzzy_skin_octaves`, default `4`, valid range `1..=10`, used by Perlin, Billow, and RidgedMulti.
   - `fuzzy_skin_persistence`, default `0.5`, valid range `0.01..=1.0`, used by Perlin and Billow.
5. `voronoi` must consume `fuzzy_skin_scale` and may ignore octaves and persistence, matching the upstream module selection where Voronoi only sets frequency and displacement.
6. `ridgedmulti` must consume `fuzzy_skin_scale` and `fuzzy_skin_octaves` and may ignore persistence, matching the upstream module selection.
7. The generated coherent displacement must be deterministic for the same input geometry/options/layer and must vary when the consumed coherent-noise parameters vary.
8. Coordinate sampling must follow the upstream `fuzzy_polyline` shape: evaluate at the sampled point's millimeter `x` and `y` coordinates plus Ares' `LayerContours::print_z()` as the `slice_z` coordinate. Do not use `layer_id` as the coherent-noise Z coordinate.
9. `generate_perimeters` / `perimeters_for_contour` must thread `LayerContours::print_z()` into fuzzy-skin dispatch. `FuzzySkinConfig::external_points`, `internal_wall_points`, and the shared fuzzification path must receive `print_z` in addition to `layer_id`.
10. The existing first-layer gate, disabled/effective-disabled gate, `all`/`external` behavior, and `allwalls` internal-wall behavior must continue to apply before noise dispatch.
11. No new option keys, registry entries, crates, dependencies, filesystem behavior, terminal behavior, UI behavior, OpenGL/viewer behavior, or WASM-incompatible APIs may be added.

## Deterministic coherent-noise shell

Exact libnoise parity is not part of this slice, but the compatibility shell must be concrete and reviewable:

1. Work in millimeters. For every sampled point `p`, compute `frequency = 1.0 / fuzzy_skin_scale` and evaluate the coherent-noise function at `(p.x * frequency, p.y * frequency, print_z * frequency)`.
2. Use deterministic integer-lattice value noise as the base signal:
   - Split the sample coordinate into integer cell coordinates and fractional offsets on each axis.
   - Hash integer cell coordinates plus a fixed per-noise-type seed with a local 64-bit integer hash.
   - Convert hash values to `[-1.0, 1.0]`.
   - Interpolate the eight cell-corner values with `smoothstep(t) = t * t * (3.0 - 2.0 * t)` and trilinear interpolation.
3. `Perlin` compatibility: sum `fuzzy_skin_octaves` value-noise octaves, doubling frequency each octave and multiplying amplitude by `fuzzy_skin_persistence`; normalize by the amplitude sum and clamp to `[-1.0, 1.0]`.
4. `Billow` compatibility: use the same octave loop as Perlin, but transform each octave's base signal with `2.0 * abs(signal) - 1.0` before amplitude accumulation; normalize and clamp.
5. `RidgedMulti` compatibility: use the same octave loop and frequency doubling as Perlin, ignore persistence, and transform each octave's base signal with `2.0 * (1.0 - abs(signal)).powi(2) - 1.0`; average over the octave count and clamp.
6. `Voronoi` compatibility: receive the same normalized coordinates from step 1, search unit lattice cells in that normalized coordinate space, derive one deterministic feature point and one deterministic patch value per cell from the local hash, choose the nearest feature point to the sample coordinate, and return that cell's patch value in `[-1.0, 1.0]`. Do not apply `fuzzy_skin_scale` a second time inside the Voronoi path.
7. Multiply the selected coherent-noise value by `fuzzy_skin_thickness` and displace along the same path normal used by the existing classic closed-polyline displacement path.

## Non-goals and deferred behavior

- Exact libnoise algorithm/seed parity is deferred. This slice ports the source boundary into a deterministic Rust coherent-noise compatibility shell with the same option ownership and parameter wiring.
- Arachne `fuzzy_extrusion_line` width behavior for `fuzzy_skin_mode = extrusion|combined` remains deferred.
- `fuzzy_skin_mode` parsing/runtime behavior remains deferred except where existing behavior already depends on displacement-only closed polylines.
- Painted fuzzy regions, fuzzy hole topology, arbitrary polygon clipping/splitting, multi-region fuzzy-effect merging, non-rectangular ownership beyond current closed-polylines, and Orca binary E2E geometry parity remain deferred.
- UI labels/tooltips/modes remain metadata-only and are not reimplemented in runtime code.

## Acceptance criteria

- Parser tests prove all six `fuzzy_skin_noise_type` values are accepted and invalid values are still rejected.
- Parser tests prove `fuzzy_skin_scale`, `fuzzy_skin_octaves`, and `fuzzy_skin_persistence` enforce the upstream numeric ranges.
- Perimeter tests prove each coherent noise type changes external perimeter geometry from the unfuzzified rectangle when fuzzy skin is enabled.
- Perimeter tests prove coherent noise is deterministic for identical inputs and differs from classic/ripple for the same fixture.
- Perimeter tests prove changing `fuzzy_skin_scale`, `fuzzy_skin_octaves`, and `fuzzy_skin_persistence` affects at least one coherent noise output that consumes that parameter.
- Perimeter tests prove changing `print_z` changes at least one coherent noise output for identical XY geometry and options.
- A G-code integration test proves a coherent noise type reaches perimeter diagnostics/print paths/G-code output.
- Existing fuzzy skin tests for classic, ripple, `all`, `external`, `allwalls`, first-layer gating, and disabled/default behavior continue to pass.
- Touched Rust files remain at or below 400 LOC.
- `docs/roadmap.md` gets a new `2026-06-29 Fuzzy skin coherent noise runtime slice` entry that cites the upstream fuzzy-noise source boundary, says Perlin/Billow/RidgedMulti/Voronoi plus scale/octaves/persistence now reach Ares' closed-polyline fuzzy skin runtime, and leaves exact libnoise parity, Arachne extrusion/combined width modes, painted regions, holes, and full Orca E2E parity deferred. The earlier fuzzy ripple entry must no longer list Perlin/Billow/RidgedMulti/Voronoi modules as deferred runtime ownership.
- Verification includes targeted fuzzy-skin tests, perimeter tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo nextest run --workspace`, and diff whitespace checks before commit/push.

## Self-review

- No placeholders or TBDs remain.
- The scope consumes already-registered fuzzy skin options and does not add new options.
- The exact upstream source boundary and Rust destination boundary are named.
- The deterministic no-dependency coherent-noise shell is defined enough for implementation and review without asking for algorithm choices.
- `print_z` threading through `perimeters.rs` is explicit.
- Voronoi uses the same normalized coordinate convention as the other coherent noise paths, avoiding double-scaling.
- The test split is explicit so LOC stays within policy.
- The roadmap update is specified as an acceptance criterion.
- Deferred behavior is explicit where exact libnoise/Arachne/painted/hole parity is outside the current Ares ownership boundary.
