# Consume Slowdown For Curled Perimeters Design

## Goal

Consume OrcaSlicer `slowdown_for_curled_perimeters` into concrete Ares overhang-perimeter speed behavior. This slice must change generated speed moves and G-code for Ares' existing fully unsupported rectangular overhang path instead of only preserving the option as registry metadata.

## Source Boundary

Upstream source slice:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1201`: `PrintRegionConfig` tuple field `slowdown_for_curled_perimeters`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1507-1528`: option definition, label, tooltip, and default `true`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1567-1577`: `bridge_speed` tooltip stating that disabled curled slowdown uses bridge speed for overhang walls supported by less than 13%.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6587-6641`: dynamic overhang speed table branch. When `slowdown_for_curled_perimeters` is true, the final `0%` overlap speed repeats `overhang_4_4_speed`; when false, the final speed uses `bridge_speed`.

Rust destination boundary:

- `crates/ares-core/src/options/overhang_speed.rs`: parse `slowdown_for_curled_perimeters` as a boolean defaulting to Orca's `true`, and wire the final severe-overhang fallback speed into the parsed overhang speed bands.
- `crates/ares-core/src/options/speed.rs`: pass resolved `bridge_speed` into overhang band parsing.
- `crates/ares-core/src/speeds/config/overhang.rs`: represent the final beyond-one-line-width speed separately from the existing four configured overlap bands.
- `crates/ares-core/src/speeds/volumetric.rs`: keep the current Ares speed-generation ordering and consume the updated band lookup.
- Focused tests in `crates/ares-core/src/options/tests/overhang_speed.rs`, `crates/ares-core/src/speeds/tests/overhang.rs`, and `crates/ares-core/src/pipeline/tests/overhang_speed.rs`.

## Included Behavior

- `slowdown_for_curled_perimeters` accepts only JSON booleans and returns `SliceError::InvalidInput` for non-boolean values.
- Missing `slowdown_for_curled_perimeters` defaults to `true`, matching Orca's `ConfigOptionBool{ true }`.
- `enable_overhang_speed = false` still disables dynamic overhang bands and keeps the existing overhang role bridge-speed fallback.
- For Ares' current unsupported-span approximation, spans with `unsupported_span_mm / external_line_width_mm <= 0.25`, `<= 0.50`, `<= 0.75`, and `<= 1.00` continue to use `overhang_1_4_speed`, `overhang_2_4_speed`, `overhang_3_4_speed`, and `overhang_4_4_speed` respectively when those bands are configured.
- For spans with `unsupported_span_mm / external_line_width_mm > 1.00`, Ares models Orca's final `0%` overlap slot:
  - when `slowdown_for_curled_perimeters = true`, use the configured `overhang_4_4_speed` if it is finite and at least `0.5 mm/s`;
  - when `slowdown_for_curled_perimeters = false`, use the resolved `bridge_speed`.
- Runtime speed selection still clamps dynamic overhang speed to no more than the move's current base overhang perimeter speed, preserving the existing cap behavior and speed-generation order before volumetric caps, smoothing, and layer-time slowdown.
- Existing first-layer behavior remains unchanged: first-layer overhang perimeters ignore dynamic overhang speed bands.

## Deferred Behavior

This slice does not port the full Orca overhang quality estimator:

- No `ExtrusionQualityEstimator`, AABB previous-layer distance calculation, per-point overlap estimation, or path subdivision.
- No segment-level threshold at Orca's exact `13%` overlap slot; Ares uses the existing whole-path unsupported-span estimate and a `> 1.00` line-width ratio as the final severe-overhang bucket for now.
- No bridge-infill dynamic speed table, raft/object-layer gate, scarf/sloped interaction, multi-region/object estimator state, or UI behavior.
- No changes to option registry metadata beyond tests that prove the runtime parser consumes the existing option key.

## Acceptance Criteria

- A default slice with `overhang_4_4_speed` configured and no explicit `slowdown_for_curled_perimeters` uses `overhang_4_4_speed` for the existing fully unsupported second-layer overhang G-code path.
- Setting `slowdown_for_curled_perimeters = false` for that same path changes the generated overhang perimeter feedrate to `bridge_speed`.
- Speed unit tests prove the final severe-overhang bucket chooses `overhang_4_4_speed` when enabled and `bridge_speed` when disabled.
- Option parser tests prove default `true`, explicit `false`, and invalid non-boolean values.
- The change does not introduce dependencies, file I/O, terminal/UI behavior, or platform-specific logic in `ares-core`.
- Every touched Rust file remains at or below 400 LOC.

## Verification

Use `cargo nextest run`, not `cargo test`.

Required RED/GREEN and final verification:

- Focused RED/GREEN: `cargo nextest run -p ares-core slowdown_for_curled_perimeters`
- Focused regression: `cargo nextest run -p ares-core overhang_speed`
- Full workspace: `cargo nextest run --workspace`
- Formatting: `cargo fmt --check`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- WASM compile gate: `cargo check -p ares-core --target wasm32-unknown-unknown`
- Diff hygiene: `git diff --check`
- Rust LOC guard for touched Rust files.

## Documentation

Update `docs/roadmap.md` to move curled-perimeter slowdown from deferred overhang speed work into the completed runtime slice list while still deferring the full Orca estimator and exact segment-level overlap behavior.
