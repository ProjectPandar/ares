# Consume Small Area Infill Flow Compensation Design

## Goal

Consume OrcaSlicer `small_area_infill_flow_compensation` and
`small_area_infill_flow_compensation_model` into concrete Ares extrusion
behavior. This slice must change generated E values for currently supported
short solid infill moves instead of only preserving the options as registry
metadata.

## Source Boundary

Upstream source slice:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1211`: `PrintRegionConfig`
  tuple field `small_area_infill_flow_compensation`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1464`: `GCodeConfig` tuple field
  `small_area_infill_flow_compensation_model`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4352-4371`: option definitions,
  defaults, labels, and the default compensation model.
- `OrcaSlicer/src/libslic3r/GCode/SmallAreaInfillFlowCompensator.cpp:27-81`:
  model parsing and validation.
- `OrcaSlicer/src/libslic3r/GCode/SmallAreaInfillFlowCompensator.cpp:91-110`:
  line-length flow multiplier and role filter for `erSolidInfill`,
  `erTopSolidInfill`, and `erBottomSurface`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6234-6251`: runtime gate that enables the
  compensator only for supported rectilinear/monotonic solid infill patterns.
- `OrcaSlicer/src/libslic3r/GCode/PchipInterpolatorHelper.cpp:26-99`: PCHIP
  data setup, derivative calculation, and interpolation used by the
  compensation model.

Rust destination boundary:

- Create or extend `crates/ares-core/src/options/small_area_infill_flow.rs`:
  parse the bool and model strings, validate the model, resolve supported
  pattern gates, and build the runtime compensation config. Wire it into
  `crates/ares-core/src/options.rs` without growing that file above 400 LOC by
  extending the existing compact module-registration macro line, and call it
  from `SliceOptions::extrusion_options()` through the existing
  `crates/ares-core/src/options/flow_ratios.rs::parse_extrusion_options`
  construction path. It must parse `bottom_surface_pattern`,
  `internal_solid_infill_pattern`, and `top_surface_pattern` directly through
  the existing infill pattern parsers rather than calling
  `SliceOptions::infill_options()`, because `parse_infill_options` already
  depends on `SliceOptions::extrusion_options()`. The parser reuse strategy is
  to make `crates/ares-core/src/options/infill.rs` expose its `patterns` module
  as `pub(crate)` and widen only `parse_bottom_surface_pattern`,
  `parse_internal_solid_infill_pattern`, and `parse_top_surface_pattern` from
  `pub(super)` to `pub(crate)`; do not add public API or duplicate pattern
  parsing.
- Create or extend `crates/ares-core/src/extrusions/small_area.rs`: hold the
  PCHIP-backed compensation model and role/layer gate. Wire it through
  `crates/ares-core/src/extrusions.rs` module declarations and re-export only
  what `ExtrusionOptions` needs.
- Extend `crates/ares-core/src/extrusions/options.rs`: store the parsed
  compensation config and expose a segment extrusion-delta helper. Because the
  PCHIP model is vector-backed and not `Copy`, `ExtrusionOptions` should stop
  deriving `Copy`; existing builder-style methods should continue to be
  ergonomic by returning a cloned modified value instead of relying on implicit
  copy semantics.
- Extend `crates/ares-core/src/extrusions.rs`: apply the helper to each print
  segment using the actual segment length before rounding cumulative E.
- Focused tests:
  - `crates/ares-core/src/options/tests/small_area_infill_flow.rs`, registered
    in `crates/ares-core/src/options/tests.rs` by extending the compact
    `option_test_modules!` macro line without growing the file above 400 LOC.
  - `crates/ares-core/src/extrusions/tests/small_area.rs`, registered from
    `crates/ares-core/src/extrusions/tests.rs`.
  - `crates/ares-core/src/pipeline/tests/small_area_infill_flow.rs`,
    registered from `crates/ares-core/src/pipeline/tests.rs`.

## Included Behavior

- Missing `small_area_infill_flow_compensation` defaults to `false`, matching
  Orca's `ConfigOptionBool(false)`, and leaves all extrusion E values unchanged.
- Explicit `small_area_infill_flow_compensation` accepts JSON booleans only and
  rejects non-boolean values with `SliceError::InvalidInput`.
- Missing `small_area_infill_flow_compensation_model` uses Orca's default
  string list:
  `0,0`, `0.2,0.4444`, `0.4,0.6145`, `0.6,0.7059`, `0.8,0.7619`,
  `1.5,0.8571`, `2,0.8889`, `3,0.9231`, `5,0.9520`, `10,1`.
- Explicit model values accept a JSON string list or a newline/semicolon
  separated JSON string. Each non-empty entry is parsed as `length, factor`
  after trimming whitespace around both fields. JSON string-list entries must be
  non-empty after trimming; empty fragments produced by splitting a serialized
  newline/semicolon string are ignored.
- The model validator follows the upstream invariants:
  - at least two parsed points;
  - the first extrusion length must be `0`;
  - only the first extrusion length may be `0`;
  - subsequent extrusion lengths must strictly increase;
  - flow compensation factors must strictly increase;
  - the final compensation factor must be `1.0`;
  - every parsed length and factor must be finite.
- The runtime multiplier uses the upstream PCHIP formula from
  `PchipInterpolatorHelper`: segment slopes, endpoint derivatives from adjacent
  slopes, weighted harmonic mean internal derivatives when adjacent slopes share
  sign, zero derivative otherwise, and cubic Hermite interpolation.
- `line_length == 0` and `line_length > max_model_length` return multiplier
  `1.0`, matching Orca's `flow_comp_model`.
- When enabled, Ares changes only the E delta for existing print segments. It
  does not alter geometry, move ordering, role generation, feedrate, fan,
  acceleration, or jerk.
- The role/layer gate mirrors Orca's two-stage `_needSAFC` plus `modify_flow`
  behavior where Ares has matching data. A segment receives the multiplier only
  when its role is one of `PrintPathRole::SolidInfill`,
  `PrintPathRole::TopSolidInfill`, or `PrintPathRole::BottomSurface`, and the
  path gate is true. The path gate is true when any of these conditions holds:
  - the segment is on the first layer and `bottom_surface_pattern` is one of
    `Rectilinear`, `AlignedRectilinear`, `Monotonic`, or `MonotonicLine`;
  - the role is `SolidInfill` and `internal_solid_infill_pattern` is one of
    those supported patterns;
  - the role is `TopSolidInfill` and `top_surface_pattern` is one of those
    supported patterns.
- Existing flow-ratio options still compose with the small-area multiplier:
  Ares first computes the normal extrusion delta from line width, layer height,
  filament/print flow, and role flow ratios, then multiplies that delta by the
  small-area factor.

## Deferred Behavior

This slice does not port the full Orca G-code writer around the compensator:

- No new infill geometry generation, path merging, path splitting, or path
  collection machinery.
- No support-material, ironing, bridge, sparse infill, perimeter, scarf, seam,
  wipe tower, multi-region, multi-object, or multi-extruder behavior.
- No UI behavior, CLI option presentation, filesystem access, terminal behavior,
  OpenGL/viewer behavior, or platform-specific code.
- No Orca binary E2E parity fixture is added in this slice; focused Rust
  regression tests prove the parsed option changes emitted Ares G-code E values.
- No new dependency is introduced for interpolation.

## Acceptance Criteria

- Option tests prove default disabled behavior, explicit enabled behavior,
  default model parsing, accepted string-list/string model forms, and invalid
  bool/model values.
- PCHIP unit tests prove these default-model interpolation points within
  `1e-9`: `0.1 -> 0.246996362897`, `0.3 -> 0.545340214541`,
  `1.0 -> 0.797266684388`, `4.0 -> 0.939899413511`, and
  `7.5 -> 0.977423853211`; they also prove zero-length multiplier `1.0`,
  above-max multiplier `1.0`, and monotonic short-line reduction below `1.0`.
- Extrusion tests prove enabling the option lowers the E delta for a short
  `SolidInfill` segment and leaves `SparseInfill`, perimeter roles, disabled
  config, and above-max solid infill segments unchanged.
- Role-gate tests prove `SolidInfill`, `TopSolidInfill`, and `BottomSurface`
  compose Orca's path gate and role filter: supported bottom pattern affects all
  three supported roles on the first layer, supported internal pattern affects
  `SolidInfill`, supported top pattern affects `TopSolidInfill`, unsupported
  patterns disable the multiplier, and roles outside the three supported roles
  remain unchanged.
- Pipeline/G-code tests prove generated Ares G-code contains a lower E delta for
  an existing short solid-surface infill move when the option is enabled.
- The change does not introduce dependencies, file I/O, terminal/UI behavior, or
  platform-specific logic in `ares-core`.
- Every touched Rust file remains at or below 400 LOC.

## Verification

Use `cargo nextest run`, not `cargo test`.

Required RED/GREEN and final verification:

- Focused RED/GREEN: `cargo nextest run -p ares-core small_area_infill_flow`
- Focused extrusion regression: `cargo nextest run -p ares-core extrusion`
- Focused pipeline regression: `cargo nextest run -p ares-core small_area`
- Full workspace: `cargo nextest run --workspace`
- Formatting: `cargo fmt --check`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- WASM compile gate: `cargo check -p ares-core --target wasm32-unknown-unknown`
- Diff hygiene: `git diff --check`
- Rust LOC guard for touched Rust files.

## Documentation

Update `docs/roadmap.md` to record this as a completed source-cited runtime
slice for small-area infill flow compensation while still deferring the full Orca
G-code/path writer context around it.
