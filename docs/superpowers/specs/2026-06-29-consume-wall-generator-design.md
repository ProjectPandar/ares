# Consume `wall_generator` Runtime Option

## Goal

Consume Orca's existing `wall_generator` option as a typed Ares perimeter option without implementing Arachne variable-width geometry in this slice. The slice must reject invalid enum values, preserve the current classic-style perimeter geometry, and make the remaining Arachne rewrite boundary explicit.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:294-300`: `PerimeterGeneratorType` has `Classic` and `Arachne` variants.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1020`: `wall_generator` is a `PrintObjectConfig` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:520-524`: enum serialization maps `Classic` to `classic` and `Arachne` to `arachne`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6989-7001`: `wall_generator` is a `coEnum`, accepts `classic` and `arachne`, and defaults to `Arachne`.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:121-124`: Orca routes to `process_arachne()` when `wall_generator == Arachne && !spiral_mode`; otherwise it routes to `process_classic()`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:30-65,482-553,684-708`: Arachne owns variable-width wall input parameters, outline simplification, beading strategy, transition filtering, minimum bead width, and path simplification.

## Current Ares Boundary

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs` already registers `wall_generator` with default `arachne` and source citations.
- `crates/ares-core/src/options/tests/registry_lookup_relative_e_wall_generator.rs` verifies registry lookup metadata only.
- `crates/ares-core/src/options/overhang_reverse.rs` builds `PerimeterOptions` but does not parse `wall_generator`.
- `crates/ares-core/src/perimeters/options.rs` has no typed wall-generator field or getter.
- `crates/ares-core/src/perimeters.rs` generates constant-width rectangular perimeter shells and thin-wall centerlines. It does not implement Orca `process_arachne()`, variable-width lines, beading strategies, or Voronoi/skeletal trapezoidation.

Because Orca's default is Arachne but Ares currently has no source-cited Arachne path generator, this slice must not claim geometry parity. The only behavior consumed here is option parsing, validation, storage, and observability through `PerimeterOptions`.

## Included Behavior

1. Add a typed `WallGenerator` enum to Ares perimeter options with `Classic` and `Arachne` variants.
2. Parse `wall_generator` from `SliceOptions::perimeter_options()`.
3. Use Orca's default `arachne` when the option is omitted.
4. Accept exactly the string values `classic` and `arachne`.
5. Reject non-string values and unknown strings with `SliceError::InvalidInput` mentioning `wall_generator`.
6. Store the parsed value in `PerimeterOptions` and expose it through a getter.
7. Preserve current perimeter geometry for both `classic` and `arachne`; the current generator remains a classic-style compatibility shell until the source-cited Arachne rewrite is implemented.

## Deferred Behavior

- Orca `LayerRegion::make_perimeters()` generator routing beyond recording the selected mode.
- `process_arachne()` parity.
- Variable-width wall lines and extrusion widths.
- `Arachne::WallToolPaths`, beading strategy, skeletal trapezoidation, transition filtering, wall distribution, min-feature/min-bead behavior, outline simplification, and wall path simplification.
- Spiral-mode-specific generator fallback.
- Geometry differences between `classic` and `arachne`.
- Orca binary E2E geometry parity.

## Acceptance Criteria

1. `SliceOptions::default().perimeter_options().unwrap().wall_generator()` returns `WallGenerator::Arachne`.
2. Setting `wall_generator = "classic"` returns `WallGenerator::Classic`.
3. Setting `wall_generator = "arachne"` returns `WallGenerator::Arachne`.
4. Unknown strings and non-string JSON values fail `perimeter_options()` with `SliceError::InvalidInput` mentioning `wall_generator`.
5. Existing default perimeter geometry is unchanged after adding the option parser.
6. Explicit `classic` and explicit `arachne` currently produce the same Ares perimeter path coordinates, documenting that geometry parity is deferred.

## Verification

- Add focused tests in `crates/ares-core/src/options/tests/wall_generator.rs` and `crates/ares-core/src/perimeters/tests/wall_generator.rs`.
- Register the options test through the existing compact `option_test_modules!` line because `crates/ares-core/src/options/tests.rs` is already at the 400-line guideline.
- `cargo nextest run -p ares-core wall_generator`
- `cargo nextest run -p ares-core wall_sequence wall_direction`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that `wall_generator` is now parsed, validated, and exposed as a perimeter option, while all Arachne/classic geometry routing differences remain deferred to a future source-cited `LayerRegion`/`Arachne::WallToolPaths` rewrite slice.
