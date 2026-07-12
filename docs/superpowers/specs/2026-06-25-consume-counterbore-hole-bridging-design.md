# Consume Counterbore Hole Bridging Design

## Goal

Consume OrcaSlicer's existing `counterbore_hole_bridging` option in a concrete Ares bridge-layer behavior slice instead of leaving it as registry metadata only.

## Upstream Source Boundary

The source boundary is pinned to the local OrcaSlicer checkout at commit `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:401-403` declares `CounterboreHoleBridgingOption` with `chbNone`, `chbBridges`, and `chbFilled`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1466-1484` defines `counterbore_hole_bridging` as enum strings `none`, `partiallybridge`, and `sacrificiallayer`, defaulting to `chbNone`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1544-1546` is the runtime slice for this change: when the region option is `chbFilled`, Orca unions bridge fill surfaces into `layerm_slices_surfaces` before top/bottom solid-surface detection.

## Ares Destination Boundary

- `crates/ares-core/src/bridges.rs` owns bridge-related option parsing and should add a small `CounterboreHoleBridging` enum plus validation.
- `crates/ares-core/src/infills.rs` owns Ares' current bridge override for bottom-surface infill generation via `InfillBridgeContext`.
- `crates/ares-core/src/gap_fills/solid_surface.rs` owns Ares' current solid-surface gap-fill skip for unsupported bridge layers.
- `crates/ares-core/src/print_paths.rs` owns Ares' current role mapping from solid infill to `bridge` or `bottom_surface`.
- `crates/ares-core/src/pipeline.rs` wires `BridgeOptions` into gap-fill, infill generation, and print-path role mapping.

## Included Behavior

1. Parse `counterbore_hole_bridging` values:
   - missing value defaults to `none`;
   - accepted strings are `none`, `partiallybridge`, and `sacrificiallayer`;
   - non-string or unknown values return `SliceError::InvalidInput` mentioning `counterbore_hole_bridging`.
2. Add runtime behavior only for `sacrificiallayer`, matching the `chbFilled` branch from `PrintObject.cpp:1544-1546` at Ares' current abstraction level.
3. When `bridge_no_support = true`, an unsupported bottom solid layer must continue to use bridge density and bridge G-code role under default `none`, preserving existing behavior.
4. When `bridge_no_support = true` and `counterbore_hole_bridging = sacrificiallayer`, that same unsupported bottom solid layer must remain a bottom-surface layer in Ares' current model. Concretely, this mode suppresses the Ares-local unsupported-bottom bridge override for that layer in infill generation, solid-surface gap-fill eligibility, and final print-path role mapping.
5. The `sacrificiallayer` mode must preserve bottom-surface speed/flow/role output rather than bridge speed/flow/role output for the affected layer, and solid-surface gap fills must remain eligible on that affected bottom-surface layer.
6. The `sacrificiallayer` mode must not apply bridge density or bridge angle to the affected bottom-surface infill. A test must distinguish this from `none` by setting bridge-specific density/angle values and proving that only the default path consumes them.
7. Existing `enable_extra_bridge_layer` behavior is unchanged; this slice only changes the fully unsupported bottom-surface bridge override.
8. `partiallybridge` is accepted and intentionally behaves like `none` in this slice because Orca's partial counterbore splitting is outside Ares' current rectangular bridge abstraction. Tests must lock this default-equivalent behavior for unsupported bottom-surface bridge classification.

## Deferred Behavior

- Full Orca counterbore hole detection and local partial bridge generation.
- Polygon union/diff between `fill_surfaces` and `slices.surfaces` beyond Ares' current fully-unsupported rectangular layer approximation.
- Region/object-specific option resolution beyond Ares' current global `SliceOptions`.
- `chbBridges` / `partiallybridge` geometry effects.
- Support-aware counterbore behavior, holes, Arachne, variable-width paths, UI/preset behavior, and exact Orca surface collections.

## Testing

- Add option parser tests for default, accepted enum values, and invalid values.
- Add print-path unit coverage showing `sacrificiallayer` keeps an unsupported bottom solid infill as `bottom_surface` while the default remains `bridge`.
- Add pipeline/G-code coverage showing `sacrificiallayer` changes the affected layer from `;PRINT_PATH:bridge:` and bridge feedrate to `;PRINT_PATH:bottom_surface:` and bottom-surface feedrate.
- Add pipeline/G-code coverage with `bridge_density = 50`, `bridge_angle = 90`, and `bottom_surface_pattern = alignedrectilinear` proving default `none` uses bridge density and bridge-angle direction while `sacrificiallayer` keeps the bottom-surface density/pattern direction.
- Add coverage showing `partiallybridge` behaves like `none` for unsupported bottom-surface bridge role, speed, density, and angle.
- Add solid-surface gap-fill coverage showing `sacrificiallayer` does not skip gap fill on the affected unsupported bottom-surface layer, while default `none` still skips it.
- Run focused RED first with the new tests and current implementation:
  - `cargo nextest run -p ares-core counterbore_hole_bridging`
- After implementation, run:
  - `cargo nextest run -p ares-core counterbore_hole_bridging bridge_no_support enable_extra_bridge_layer`
  - full workspace verification before commit.

## File Size Strategy

Current relevant Rust file sizes are:

- `crates/ares-core/src/bridges.rs`: 184 LOC.
- `crates/ares-core/src/gap_fills/mod.rs`: 84 LOC.
- `crates/ares-core/src/gap_fills/solid_surface.rs`: 256 LOC.
- `crates/ares-core/src/infills.rs`: 371 LOC.
- `crates/ares-core/src/print_paths.rs`: 381 LOC.
- `crates/ares-core/src/pipeline.rs`: 394 LOC.
- `crates/ares-core/src/options/tests.rs`: 400 LOC.
- `crates/ares-core/src/pipeline/tests.rs`: 400 LOC.
- `crates/ares-core/src/print_paths/tests.rs`: 305 LOC.
- `crates/ares-core/src/pipeline/tests/gap_fill_role_gcode.rs`: 357 LOC.

The implementation plan must keep every touched Rust file at or below the 400 LOC project limit. New tests should live in new focused test files plus existing module declarations only. `pipeline.rs` has only six spare lines, so policy logic belongs in `bridges.rs`, `gap_fills/solid_surface.rs`, `infills.rs`, and `print_paths.rs`; `pipeline.rs` may only wire parsed bridge options through existing calls. If any planned edit would push a file above 400 LOC, the implementation must first make a same-scope module split instead of adding more lines.

## Docs Impact

Update `docs/roadmap.md` after implementation to record that `counterbore_hole_bridging = sacrificiallayer` is consumed by runtime bridge, infill, and gap-fill classification. No architecture ADR is required because this does not introduce a new boundary or irreversible design decision.

## Safety And Rollback

This slice only changes `ares-core` option parsing and existing in-memory bridge classification. It adds no file I/O, terminal behavior, UI, OpenGL, platform-specific code, or dependency. Rollback is removing the enum field/parser, removing the new context flag from gap-fill/infill/print-path wiring, and deleting the focused tests/spec/plan.

## Self-Review

- No placeholders or TODOs remain.
- The behavior is source-cited and framed as an upstream Orca rewrite slice.
- The scope is intentionally narrow and does not claim complete counterbore hole parity.
