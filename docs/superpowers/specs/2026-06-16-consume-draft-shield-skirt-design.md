# Consume Draft Shield Skirt Behavior Design

## Purpose

Consume the existing Orca `draft_shield` option into concrete Ares skirt generation behavior before adding more option metadata. This is a source-cited Rust rewrite slice of OrcaSlicer skirt enablement and draft-shield height semantics, scoped to the current Ares rectangular skirt approximation.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:290-292`
  - `enum DraftShield { dsDisabled, dsEnabled }`
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1512`
  - `((ConfigOptionEnum<DraftShield>,  draft_shield))`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5573-5586`
  - `draft_shield` option definition, serialized values `disabled` and `enabled`, default `dsDisabled`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11306-11313`
  - `has_skirt(const DynamicPrintConfig&)`: skirt exists when `skirt_height > 0 && skirt_loops > 0`, or when `draft_shield != dsDisabled`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11315-11323`
  - `get_real_skirt_dist`: enabled draft shield with `skirt_loops == 0` behaves as one loop for real skirt distance only
- `OrcaSlicer/src/libslic3r/Print.cpp:572-577`
  - `Print::has_infinite_skirt`: infinite skirt is true only when `draft_shield == dsEnabled && skirt_loops > 0`
- `OrcaSlicer/src/libslic3r/Print.cpp:2593-2669`
  - `_make_skirt` chooses all object layers when `has_infinite_skirt()` is true.
- `OrcaSlicer/src/libslic3r/Print.cpp:2686-2695`
  - Combined skirt generation loops over `m_config.skirt_loops`; if loops are zero, no skirt loop is emitted.

## Rust Destination Boundary

- `crates/ares-core/src/skirts.rs`
  - Add a `DraftShield` enum and extend `SkirtOptions` so skirt generation knows whether draft shield is enabled.
  - Keep the existing rectangular bounds approximation and layer-preserving `LayerSkirts` output.
- `crates/ares-core/src/options.rs`
  - Delegate `draft_shield` parsing from `SliceOptions` into a small option submodule so `options.rs` remains at or below 400 LOC.
  - Accept Orca serialized strings `disabled` and `enabled`.
- `crates/ares-core/src/lib.rs`
  - Export the new skirt draft-shield enum if needed by tests and downstream adapters.
- Pipeline consumers stay unchanged except for receiving richer `SkirtOptions`.

## Included Behavior

1. Missing `draft_shield` defaults to disabled.
2. `draft_shield = "disabled"` preserves current Ares skirt behavior:
   - `skirt_loops = 0` emits no skirt paths.
   - `skirt_height` limits generated skirt layers by current `layer_id < skirt_height` behavior.
3. `draft_shield = "enabled"` with `skirt_loops = 0` emits no skirt paths in this artifact generator, matching `Print::_make_skirt` combined-skirt loop generation.
4. `draft_shield = "enabled"` with `skirt_loops > 0` consumes the source-cited infinite-skirt height behavior in the current layer model:
   - Every represented layer with contour bounds gets skirt paths.
   - Empty represented layers remain present with no paths.
   - The configured loop count is preserved on every generated layer.
5. Invalid `draft_shield` values return `SliceError::InvalidInput` with an option-specific message.
6. G-code output naturally includes skirt comments and moves on later layers when enabled draft shield generates those skirt paths.

## Deferred Behavior

- True Orca convex hull offsetting, round joins, simplification, and flow-derived spacing.
- `single_loop_draft_shield` after-first-layer loop limiting.
- `skirt_type = perobject` and per-object skirt ownership.
- `min_skirt_length` loop extension.
- Applying `get_real_skirt_dist`'s `skirt_loops == 0` one-loop distance override to any Ares spacing or arrangement calculation. Ares does not yet have a real skirt-distance arrangement surface.
- Wipe tower, support-layer, brim trimming, and full `Print::_make_skirt` scheduling interactions.
- Orca E2E parity for draft shield geometry. This slice only makes existing Ares rectangular skirt artifacts honor the source-cited enablement and height semantics.

## Docs Impact

No architecture or roadmap document update is required for this slice. The work implements an already documented rewrite direction inside `ares-core`, stays within existing crate boundaries, and records the detailed source boundary in this spec and the paired implementation plan.

## Acceptance Criteria

- Unit tests prove disabled draft shield preserves current `skirt_loops`/`skirt_height` behavior.
- Unit tests prove enabled draft shield with `skirt_loops = 0` emits no skirt paths.
- Unit tests prove enabled draft shield with multiple loops uses the configured loop count on later layers.
- `SliceOptions::skirt_options()` parses missing, `disabled`, `enabled`, and invalid `draft_shield`.
- A pipeline/G-code test proves enabled draft shield emits skirt artifacts or G-code comments beyond the first layer.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test -p ares-core --lib` pass.
- Modified Rust files stay at or below 400 LOC; split tests or helpers if needed.

## Safety and Rollback

The change is isolated to skirt option parsing and skirt artifact generation. Rollback is a single commit revert. The default remains disabled, so existing callers that do not set `draft_shield` keep the current behavior.
