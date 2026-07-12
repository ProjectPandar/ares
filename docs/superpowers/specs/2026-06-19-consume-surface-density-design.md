# Consume Top And Bottom Surface Density Design

## Goal

Consume OrcaSlicer `top_surface_density` and `bottom_surface_density` as concrete Ares slicing behavior. The options are already present in the Ares registry metadata; this slice must make non-default values change generated top and bottom surface scanline spacing and emitted G-code line counts.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1088-1089` declares `top_surface_density` and `bottom_surface_density` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6586-6596` registers `top_surface_density` as a percent option, default `100`, min `0`, max `100`, and documents that `0%` leaves only walls on the top layer.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6598-6607` registers `bottom_surface_density` as a percent option, default `100`, min `10`, max `100`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2767-2769` asserts fill density is positive and computes rectilinear line spacing as `spacing / params.density`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2787-2794` keeps full solid infill on adjusted solid spacing but uses the density-derived spacing for non-full or density-adjusted fill.

## Current Ares State

- `crates/ares-core/src/options/registry/...` already records `top_surface_density` and `bottom_surface_density` metadata from OrcaSlicer, but `crates/ares-core/src/options/infill.rs` does not parse either value into `InfillOptions`.
- `crates/ares-core/src/options/infill/layer_role.rs` already classifies generated infill layers as `Sparse`, `BottomSurface`, `InternalSolid`, or `TopSurface`.
- `crates/ares-core/src/infills.rs` currently gives every non-sparse, non-bridge role the same `solid_line_width()` spacing. As a result, changing `top_surface_density` or `bottom_surface_density` cannot affect generated geometry or G-code.
- The previous `bridge_density` slice added a bridge spacing override for fully unsupported bottom bridges. That override must remain higher precedence than ordinary bottom surface density.

## Ares Destination Boundary

Implement the smallest source-cited runtime slice:

- Parse `top_surface_density` into `InfillOptions` with Orca default `100.0`, range `0.0..=100.0`.
- Parse `bottom_surface_density` into `InfillOptions` with Orca default `100.0`, range `10.0..=100.0`.
- For `InfillLayerRole::TopSurface`, compute scanline spacing as `solid_line_width / (top_surface_density / 100.0)` when density is positive.
- For `top_surface_density == 0`, generate no top-surface infill paths for that layer, matching Orca's documented "only walls" top-layer behavior inside Ares' current wall/infill split.
- For `InfillLayerRole::BottomSurface`, compute scanline spacing as `solid_line_width / (bottom_surface_density / 100.0)`.
- Keep `InfillLayerRole::InternalSolid` at `solid_line_width()` and `InfillLayerRole::Sparse` at existing sparse spacing.
- Keep the existing bridge override first: a fully unsupported bottom bridge using `bridge_no_support` and `bridge_density` must ignore `bottom_surface_density` for spacing.
- Keep implementation inside `ares-core`; add no dependency, crate, filesystem behavior, terminal behavior, UI behavior, OpenGL behavior, or platform-specific path.

## Explicitly Deferred

- Full Orca `Surface` and `SurfaceCollection` ownership for top/bottom surface partitioning.
- Shell-thickness expansion beyond the existing Ares `top_shell_layers` and `bottom_shell_layers` role classifier.
- Per-island or mixed-density surface partitioning.
- Density-driven flow changes. This slice changes generated scanline geometry/count, while existing extrusion width and flow-ratio slices keep owning extrusion math.
- `internal_bridge_density`, support interface density, ironing, gap-fill density, min-width top surface behavior, and full bridge detector parity.
- New option registry metadata. The options already exist in registry metadata; this slice consumes them in runtime slicing.

## Design

Add `top_surface_density_percent` and `bottom_surface_density_percent` fields to `InfillOptions` beside the existing surface pattern and bridge density fields. Expose small accessors so option tests and spacing code can assert parsed values without reaching into private fields.

In `generate_infills_with_bridge_context`, keep sparse-density global behavior unchanged. After selecting the layer role and bridge override, ask `spacing_for_role(...)` for an optional spacing:

- `Some(spacing)` means generate scanlines as today.
- `None` means this role intentionally emits no infill paths for the layer. This should only occur for `TopSurface` with `top_surface_density == 0` and no bridge override.

`spacing_for_role(...)` remains the single local place where density becomes spacing:

- bridge override: `solid_line_width / (bridge_density / 100.0)`;
- sparse: existing sparse spacing;
- bottom surface: `solid_line_width / (bottom_surface_density / 100.0)`;
- top surface with positive density: `solid_line_width / (top_surface_density / 100.0)`;
- top surface with zero density: no infill paths;
- internal solid: `solid_line_width`.

This keeps Ares' current role pipeline intact while giving already existing surface-density options a concrete geometric effect.

## Tests

Use TDD with focused RED/GREEN coverage:

- Option parsing:
  - defaults parse both densities as `100.0`;
  - top density accepts `0`, `100`, and numeric strings within range;
  - bottom density accepts `10`, `100`, and numeric strings within range;
  - invalid top values below `0`, above `100`, NaN/infinity strings, nonnumeric strings, booleans, and null fail with an error mentioning `top_surface_density`;
  - invalid bottom values below `10`, above `100`, NaN/infinity strings, nonnumeric strings, booleans, and null fail with an error mentioning `bottom_surface_density`.
- Infill geometry:
  - `top_surface_density = 50` increases top-surface spacing and reduces top-surface scanline count while preserving bottom and sparse layers;
  - `bottom_surface_density = 50` increases bottom-surface spacing and reduces bottom-surface scanline count while preserving top and sparse layers;
  - `top_surface_density = 0` emits no top-surface infill paths while preserving other layers;
  - a fully unsupported bottom bridge keeps using `bridge_density` spacing even when `bottom_surface_density` is lower.
- Pipeline/G-code:
  - lowering top density reduces `;PRINT_PATH:top_solid_infill:` lines without changing `;PRINT_PATH:bottom_surface:` lines;
  - lowering bottom density reduces `;PRINT_PATH:bottom_surface:` lines without changing `;PRINT_PATH:top_solid_infill:` lines;
  - top density `0` emits no `;PRINT_PATH:top_solid_infill:` lines in the deterministic rectangular scaffold.

## Acceptance Criteria

1. Both density options have non-test runtime uses in `ares-core` infill generation.
2. Defaults preserve current generated top/bottom surface spacing.
3. Non-default top density affects only `InfillLayerRole::TopSurface`.
4. Non-default bottom density affects only `InfillLayerRole::BottomSurface` when no bridge override applies.
5. `top_surface_density = 0` suppresses top-surface infill paths instead of dividing by zero or producing invalid geometry.
6. Sparse infill, internal solid infill, bridge angle, bridge density, surface patterns, speed, acceleration, jerk, extrusion width, and flow-ratio behavior remain unchanged except through changed surface path counts.
7. All touched Rust source files stay at or below 400 LOC.
8. No new crates, dependencies, platform-specific behavior, or Ares-owned pipeline design are introduced.

## Verification

- Targeted RED/GREEN tests:
  - `cargo test -p ares-core --lib surface_density`
  - `cargo test -p ares-core --lib bridge_density`
- Regression and full checks:
  - `cargo test -p ares-core --lib`
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `find crates -name '*.rs' ! -path '*/target/*' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; found = 1 } END { exit found }'`

## SDD Gates

- Do not write implementation code until this spec/design and its implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with this spec, the reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

Update `docs/roadmap.md` after implementation to state that `top_surface_density` and `bottom_surface_density` now drive concrete Ares top/bottom surface spacing and G-code line counts, while full Orca surface ownership and adjacent density features remain deferred.
