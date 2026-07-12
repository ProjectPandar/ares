# Consume reduce_infill_retraction Design

## Intent

Consume the existing OrcaSlicer `reduce_infill_retraction` option in Ares G-code output instead of leaving it as metadata-only configuration. This is a source-cited Rust rewrite slice of Orca `libslic3r` retraction behavior, focused on the ordinary same-layer travel retraction decision that Ares already emits.

## Upstream Source Boundary

All upstream citations below refer to the repo-local `OrcaSlicer` git checkout pinned at commit `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1544` declares `ConfigOptionBool reduce_infill_retraction`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4829-4835` defines the option, labels it "Reduce infill retraction", describes skipping retraction for travel entirely within an infill area, and defaults it to `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7280-7289` passes the travel target extrusion role into `GCode::needs_retraction`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7458-7578` implements `GCode::needs_retraction`; the relevant gate skips retraction only when the travel is long enough for ordinary retraction, the target role is not perimeter, `reduce_infill_retraction` is true, the current layer exists, sparse infill density is positive, and `m_retract_when_crossing_perimeters.travel_inside_internal_regions(*m_layer, travel)` returns true.

## Ares Destination Boundary

- Parse the option in `crates/ares-core/src/options/layer_change_retraction.rs` because Ares already groups travel and layer-change retraction settings there.
- Consume it in `crates/ares-core/src/gcode_travel_retraction.rs`, the current owner of ordinary travel retract/unretract decisions.
- Pass the current move role and sparse-infill-density gate from `crates/ares-core/src/gcode.rs` without adding file I/O, UI behavior, terminal behavior, OpenGL, or platform-specific code.
- `crates/ares-core/src/gcode.rs` is already exactly 400 LOC. The implementation must keep this file line-neutral or line-negative, for example by compacting the repeated `crate::gcode_move_buffer::flush(...)` calls around role-fan buffering while adding only the required command fields.
- `crates/ares-core/src/options/infill.rs` is already 393 LOC. Extract the effective-density helper by replacing the existing local density block, keeping the file at or below 400 LOC.
- Test the emitted G-code in a new `crates/ares-core/src/tests/travel_retraction_gcode/reduce_infill.rs` submodule; `crates/ares-core/src/tests/travel_retraction_gcode.rs` should only gain `mod reduce_infill;` so it stays well below 400 LOC.

## Included Behavior

- Default `reduce_infill_retraction` to `false`, matching Orca.
- Accept Orca-style bool input using the same first-value parsing shape used by other single-extruder retraction options.
- Reject invalid `reduce_infill_retraction` values with a `SliceError::InvalidInput` that names the option key.
- When the option is enabled and effective `sparse_infill_density > 0`, suppress ordinary same-layer travel retraction and the paired unretraction for travel from a previous internal infill print role to a target internal infill print role.
- Effective sparse infill density must come from the same semantics as `SliceOptions::infill_options()`: `spiral_mode=true` normalizes the effective sparse density to `0.0`, so this option cannot suppress retraction in spiral mode. The concrete implementation should extract a small helper from the existing `parse_infill_options` density calculation rather than re-parse `sparse_infill_density` independently in `gcode.rs`.
- The exact Ares internal-infill role predicate for both previous and target roles is `SparseInfill | SolidInfill | TopSolidInfill | BottomSurface | Bridge | InternalBridge`. It excludes `ExternalPerimeter`, `OverhangPerimeter`, `InternalPerimeter`, `GapFill`, `Skirt`, and `Brim`.
- Keep existing ordinary travel retraction behavior when the option is false, when sparse infill density is zero, when the target role is a perimeter, when the previous printed role is a perimeter, when travel is below `retraction_minimum_travel`, or when retraction is already suppressed by an existing Ares rule.
- Keep z-hop tied to retraction: skipped retraction must also skip travel z-hop, matching the Orca tooltip note that z-hop is not performed where retraction is skipped.

## Deferred Behavior

- Full Orca `m_retract_when_crossing_perimeters.travel_inside_internal_regions(*m_layer, travel)` geometry is deferred until Ares has the corresponding internal-region containment data structure.
- Orca support-material island and tree-support retraction suppression remain deferred; this slice is only for the `reduce_infill_retraction` gate.
- Avoid-crossing-perimeters rerouting, wipe disabling, multi-extruder-specific filament config differences, wipe tower behavior, and complete `LiftType` parity remain deferred.
- Existing metadata-only generated files for the `PrintConfig.hpp` tuple line remain historical source records; this slice adds runtime behavior through handwritten Ares slicing/G-code modules.

## Acceptance Criteria

- A test proves `reduce_infill_retraction=true` suppresses the retract/unretract pair before a sparse-infill-to-sparse-infill ordinary travel with positive sparse infill density.
- A test proves default or explicit false preserves the same ordinary travel retract/unretract pair.
- A test proves `sparse_infill_density=0` preserves the ordinary travel retract/unretract pair even when `reduce_infill_retraction=true`.
- A test proves `spiral_mode=true` preserves ordinary travel retract/unretract even when raw `sparse_infill_density` is positive, because the effective density gate is normalized to zero.
- A test proves a perimeter target preserves ordinary travel retract/unretract even when `reduce_infill_retraction=true`.
- A test proves a previous perimeter print role preserves ordinary travel retract/unretract before an internal infill target even when `reduce_infill_retraction=true`.
- A test proves skipped reduce-infill retraction also skips the travel z-hop lift/restore pair.
- A test proves invalid option values are rejected with `reduce_infill_retraction` in the error.
- Verification uses `cargo nextest run`, not `cargo test`, and includes `cargo fmt --check`, targeted `cargo nextest run`, full `cargo nextest run --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- Touched Rust files stay at or below 400 LOC.

## Documentation

Update `docs/roadmap.md` to record that `reduce_infill_retraction` has moved from option metadata into a concrete, source-cited G-code retraction behavior slice, and list the full internal-region geometry parity work as deferred.
