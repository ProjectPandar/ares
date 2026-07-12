# Consume Filament Flow Ratio Design

## Goal

Consume OrcaSlicer `filament_flow_ratio` as concrete Ares extrusion/G-code behavior for the current single-filament slicing pipeline, instead of adding more option metadata.

## Upstream Source Boundary

These anchors are for the repository-local `OrcaSlicer` checkout at commit `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24` on branch `main`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1301` declares `filament_flow_ratio` in `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2227-2237` defines `filament_flow_ratio` as nullable filament flow ratio values, default `1`, with tooltip text stating that it changes all extrusion flow of this filament in G-code proportionally.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10201-10205` rejects non-positive `filament_flow_ratio` values during config validation.
- `OrcaSlicer/src/libslic3r/Extruder.hpp:38-41` documents that Orca `e_per_mm3()` contains `filament_flow_ratio / filament cross-sectional area`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6397-6443` reads `FILAMENT_CONFIG(filament_flow_ratio)`, multiplies `_mm3_per_mm` by it before role flow ratios, then divides the final `e_per_mm` by the same filament flow ratio after using `m_writer.filament()->e_per_mm3()`. The resulting emitted extrusion length is still linearly proportional to `filament_flow_ratio` because `e_per_mm3()` already includes that ratio.

## Ares Destination Boundary

- `crates/ares-core` remains the only changed crate.
- `SliceOptions::extrusion_options()` consumes the first `filament_flow_ratio` value for the existing single-filament pipeline.
- `ExtrusionOptions` stores the parsed filament flow ratio and applies it when computing `extrusion_per_mm` and `extrusion_per_mm_for_layer`.
- Existing role flow ratios and `print_flow_ratio` remain multiplicative with `filament_flow_ratio`.
- Parser work stays in `crates/ares-core/src/options/flow_ratios.rs`; avoid growing `crates/ares-core/src/options.rs` beyond the 400 LOC gate.

## Behavior

When `filament_flow_ratio` is missing, Ares keeps current extrusion behavior with an implicit ratio of `1.0`.

When `filament_flow_ratio` is present, Ares consumes its first scalar/list/string numeric value for the current single-filament path and scales all generated extrusion lengths for that filament by that value. A `0.5` ratio halves generated E deltas relative to default; a `1.5` ratio multiplies generated E deltas by `1.5`.

The filament flow ratio composes with existing Ares flow modifiers. For example, with `print_flow_ratio = 1.5` and `filament_flow_ratio = 0.5`, generated E deltas equal the default E delta multiplied by `0.75`.

## Accepted Inputs

- Missing `filament_flow_ratio`: default `1.0`.
- Numeric scalar values greater than `0.0`.
- Numeric string values greater than `0.0`.
- Numeric arrays/lists and `;` or `,` separated strings accepted by existing numeric-vector parsing; Ares consumes the first value for the current single-filament pipeline.

Invalid inputs must return `SliceError::InvalidInput` naming `filament_flow_ratio`:

- values less than or equal to `0.0`,
- non-finite values,
- empty arrays or empty string-list members,
- non-numeric values,
- arrays whose parsed members include any invalid value.

## Testing

- Add options wiring tests proving:
  - missing `filament_flow_ratio` leaves `ExtrusionOptions` output equal to default,
  - scalar, numeric string, list, and string-list forms reach `ExtrusionOptions`,
  - invalid forms return `SliceError::InvalidInput` containing `filament_flow_ratio`.
- Add a generated G-code pipeline test proving `filament_flow_ratio` changes an external perimeter E delta linearly.
- Add a generated G-code pipeline test proving `filament_flow_ratio` composes multiplicatively with `print_flow_ratio`.
- Run at minimum:
  - `cargo fmt --check`
  - targeted `filament_flow_ratio` tests,
  - `cargo test -p ares-core --lib`,
  - `cargo clippy --workspace --all-targets -- -D warnings`,
  - `git diff --check`,
  - a LOC gate confirming edited `crates/ares-core/src/*.rs` files do not exceed 400 lines.

## Deferred Behavior

- Multi-filament/multi-extruder runtime selection is deferred; Ares consumes only the first value for the current single-filament pipeline.
- Orca's full `GCodeWriter`/`Extruder` state model is deferred; Ares implements the equivalent current-pipeline extrusion-length effect inside `ExtrusionOptions`.
- Filament override/update behavior for `filament_flow_ratio` is not changed in this slice.
- `filament_max_volumetric_speed`, adaptive volumetric speed, pressure equalizer, wipe tower, flush tower, and tool-change purge behavior are deferred.

## Docs Impact

No public API or user guide update is required in this slice. The behavior is covered by this source-cited spec and tests because Ares has not introduced end-user option documentation beyond rewrite milestone/spec artifacts for these runtime option-consumption slices.

## Safety

The change is local, deterministic extrusion math. It performs no file I/O in `ares-core`, adds no dependency, and remains WASM-safe.
