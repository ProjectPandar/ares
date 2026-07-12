# Consume Support Ironing Pattern Design

## Objective

Consume OrcaSlicer's existing `support_ironing_pattern` option into concrete support-interface Ironing path geometry in `ares-core`. This slice must move an already registered support option from metadata/config parsing into observable slicing behavior; it must not add new option metadata or invent an Ares-owned support-ironing pipeline.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:997-1000` declares the support ironing option group, including `ConfigOptionEnum<InfillPattern> support_ironing_pattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406-6446` defines `support_ironing`, `support_ironing_pattern`, `support_ironing_flow`, and `support_ironing_spacing`; `support_ironing_pattern` accepts `rectilinear` and `concentric`, defaulting to `ipRectilinear`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:58-61` copies `object_config.support_ironing_pattern` into `SupportParameters::ironing_pattern` with support ironing enablement, flow, and spacing.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1877-1886` creates the support ironing filler with `Fill::new_from_type(support_params.ironing_pattern)` and assigns spacing and angle.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1899-1907` emits support contact-layer Ironing paths using `ExtrusionRole::erIroning` and support ironing flow.

## Ares Destination Boundary

- `crates/ares-core/src/options/ironing_flow.rs` owns support ironing runtime config for `support_ironing_flow` and `support_ironing_spacing`.
- `crates/ares-core/src/options/ironing_flow.rs` will also own a crate-visible `SupportIroningPattern` enum plus a private parser, exposed through `SupportIroningConfig`, so support pattern parsing stays with the rest of the support ironing runtime config and `support_ironing.rs` can branch without private-type leakage.
- `crates/ares-core/src/print_paths/support_ironing.rs` owns the current support-interface Ironing compatibility shell that duplicates support-interface paths and expands closed rectangular support-interface paths into rectilinear Ironing lines.
- `crates/ares-core/src/pipeline/tests/support_ironing_spacing.rs` currently covers support spacing geometry and metadata preservation.
- `crates/ares-core/src/pipeline/tests/support_ironing_pattern.rs` will cover support pattern parsing, support pattern geometry, metadata preservation, and ordinary/support independence.

## Included Behavior

1. Parse `support_ironing_pattern` in support ironing runtime config, defaulting to `rectilinear` when omitted.
2. Represent parsed support pattern as a `pub(crate) SupportIroningPattern` enum in `crates/ares-core/src/options/ironing_flow.rs`, store it on `SupportIroningConfig`, expose it through `SupportIroningConfig::pattern()`, and branch support path geometry in `support_ironing.rs` through that crate-visible enum.
3. Accept exactly Orca's current support ironing enum values `rectilinear` and `concentric` after existing legacy normalization has already run.
4. Preserve the existing legacy `support_ironing_pattern = "zig-zag"` migration to `rectilinear`; this compatibility migration happens before runtime support pattern parsing and is not treated as an invalid runtime value in this slice.
5. Reject non-string or unknown post-normalization `support_ironing_pattern` values with `SliceError::InvalidInput` whose message includes `support_ironing_pattern`.
6. Preserve current support rectilinear behavior: a closed four-corner axis-aligned rectangular support-interface path with positive spacing generates open horizontal Ironing line paths.
7. For `support_ironing_pattern = "concentric"`, a closed four-corner axis-aligned rectangular support-interface path with positive spacing generates closed rectangular concentric Ironing loops. The outer loop uses the source rectangle, and each subsequent loop steps inward by `support_ironing_spacing` while both X and Y extents remain positive.
8. Preserve support ironing metadata on generated paths: public role `PrintPathRole::Ironing`, extrusion role `PrintPathRole::SupportMaterialInterface`, generated closed flag, support flow-scaled effective layer height, unsupported span, and seam gap.
9. Preserve existing zero-spacing and unsupported-shape behavior: zero spacing or non-eligible geometry duplicates the original support-interface geometry once without pattern-specific fill expansion.
10. Keep ordinary `ironing_pattern` out of scope; support `support_ironing_pattern` must not alter ordinary Ironing geometry.

## Deferred Behavior

- Full Orca support contact-layer polygon discovery, `polys_to_iron` generation, union/diff clipping, and `FillConcentric` / `FillRectilinear` parity.
- Support ironing angle selection, non-rectangular concentric clipping, holes, island chaining, path ordering, `link_max_length`, and exact extrusion-width calculation from filler spacing.
- Multi-extruder support ownership beyond Ares' current single-active support-interface path.
- Distinct user-visible `support ironing` G-code label beyond the existing public `ironing` role.
- Ordinary `ironing_pattern` behavior, already consumed by the ordinary Ironing runtime slice.
- Orca binary E2E geometry parity.

## Acceptance Criteria

- Omitting `support_ironing_pattern` with `support_ironing = true` and `support_ironing_spacing = 1.0` over the existing 4 mm by 3 mm rectangular support-interface fixture still emits four open rectilinear Ironing lines at Y coordinates `0.0`, `1.0`, `2.0`, and `3.0`.
- Explicit `support_ironing_pattern = "rectilinear"` emits the same four open lines for that fixture.
- `support_ironing_pattern = "concentric"` and `support_ironing_spacing = 1.0` over the same fixture emits two closed loops:
  - `(0.0,0.0) -> (4.0,0.0) -> (4.0,3.0) -> (0.0,3.0)`
  - `(1.0,1.0) -> (3.0,1.0) -> (3.0,2.0) -> (1.0,2.0)`
- `support_ironing_pattern = "concentric"` with `support_ironing_spacing = 0` keeps the existing single closed support-interface duplicate.
- Invalid `support_ironing_pattern` values fail before path output with an error mentioning `support_ironing_pattern`.
- `support_ironing_pattern = "zig-zag"` remains covered by the existing legacy migration tests and is normalized to `rectilinear`, so invalid-value tests use non-normalized unsupported values such as `monotonic`, `concentric `, booleans, arrays, objects, and null.
- Support concentric generated paths preserve support flow scaling and source metadata.
- Support `support_ironing_pattern` does not change ordinary Ironing duplicate points.

## Verification

- TDD RED: `cargo nextest run -p ares-core support_ironing_pattern` fails before implementation because the support pattern parser/path behavior is absent.
- Focused GREEN after implementation:
  - `cargo nextest run -p ares-core support_ironing_pattern`
  - `cargo nextest run -p ares-core support_ironing_spacing`
  - `cargo nextest run -p ares-core ironing_pattern`
  - `cargo nextest run -p ares-core legacy_pattern_migrations`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC guard confirms every touched Rust file is at or below 400 LOC.

## Safety And Documentation

This slice is local to `ares-core` option parsing and print-path generation. It adds no dependencies, no filesystem access, no terminal behavior, no UI behavior, and no non-WASM APIs. `docs/roadmap.md` must be updated after implementation to record the consumed runtime slice and keep deferred upstream behavior explicit.
