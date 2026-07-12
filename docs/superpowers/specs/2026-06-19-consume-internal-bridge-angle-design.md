# Consume Internal Bridge Angle Design

## Goal

Consume OrcaSlicer `internal_bridge_angle` as concrete Ares slicing behavior. The option is already present in Ares' source-cited PrintConfig metadata, and the previous slice makes non-default `internal_bridge_density < 100` generate `InternalBridge` infill paths. This slice must make positive `internal_bridge_angle` change the generated internal bridge scanline direction for that existing Ares internal bridge boundary instead of adding more option-only metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1081-1084` declares `bridge_angle`, `internal_bridge_angle`, `bridge_flow`, and `internal_bridge_flow` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1226-1235` registers `internal_bridge_angle` as "Internal bridge infill direction", default `0`, min `0`, and documents `0` as automatic bridge-angle detection while positive values are used for internal bridges.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3105-3106` applies positive `internal_bridge_angle` as an override by converting configured degrees to radians before constructing internal bridge geometry.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3199-3202` converts selected internal solid surfaces into `stInternalBridge` surfaces and copies the computed `bridge_angle` onto the generated surface.
- `OrcaSlicer/src/libslic3r/Surface.hpp:23-25,42` defines `stInternalBridge` / `stSecondInternalBridge` and stores `Surface::bridge_angle` in radians, with negative values meaning undefined.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:939-943` forwards normal infill angle and `surface.bridge_angle` into fill parameters.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:300-307` prefers a defined bridge angle over normal layer fill rotation.

## Current Ares State

- Ares has registry metadata for `internal_bridge_angle`, but `crates/ares-core/src/options/infill.rs` does not parse it into runtime infill options.
- `bridge_angle` is already parsed and consumed for Ares' external unsupported bottom bridge path.
- `internal_bridge_density` is already parsed and consumed for Ares' temporary internal bridge boundary: dense middle `InfillLayerRole::InternalSolid` layers become `InfillRole::InternalBridge` only when shell-layer settings create a shell/middle distinction and `internal_bridge_density < 100`.
- `crates/ares-core/src/infills/rotation.rs` already supports a fixed angle override through `InfillPasses::new(..., fixed_angle_degrees)`, currently used by external bridge overrides.
- Ares does not yet own full Orca `SurfaceCollection` partitioning or automatic internal bridge angle detection.

## Ares Destination Boundary

Implement the smallest source-cited runtime slice inside `ares-core`:

- Parse `internal_bridge_angle` into `InfillOptions` with Orca default `0.0`, lower bound `0.0`, and no finite upper bound beyond existing numeric parsing behavior.
- Expose an accessor and test helper for `internal_bridge_angle_degrees`.
- Treat `internal_bridge_angle == 0.0` as automatic detection deferred and preserve current internal bridge direction.
- Treat `internal_bridge_angle > 0.0` as a fixed scanline angle only when the same Ares predicate already classifies generated paths as `InfillRole::InternalBridge`.
- Reuse one internal bridge eligibility predicate for spacing, role mapping, and angle override:

```text
role == InternalSolid
and at least one shell-layer setting is nonzero
and internal_bridge_density < 100.0
```

- Keep dense middle layers with `internal_bridge_density == 100` as ordinary solid infill even when `internal_bridge_angle > 0`.
- Keep sparse middle layers, no-shell dense layers, top surfaces, bottom surfaces, external bridges, surface densities, pattern selection, and downstream speed/flow/fan behavior unchanged except through the intended internal bridge path direction.
- Keep implementation WASM-safe and platform-neutral: no filesystem, terminal, UI, OpenGL, native-only behavior, new crates, or new dependencies.

## Explicitly Deferred

- Full Orca `SurfaceCollection` ownership and true `stInternalBridge` / `stSecondInternalBridge` surface generation.
- Automatic internal bridge angle detection when `internal_bridge_angle == 0`.
- `enable_extra_bridge_layer`, `dont_filter_internal_bridges`, support-aware ownership, partial per-island internal bridge classification, and sparse-infill support interaction.
- Changes to external `bridge_angle`, `bridge_density`, top/bottom surface density, sparse infill density, surface patterns, gap fill, support, ironing, shell-thickness expansion, flow, speed, fan, or extrusion behavior.
- New registry metadata. This slice consumes an already recorded option in runtime slicing.

## Design

Add `internal_bridge_angle_degrees` to `InfillOptions` next to `bridge_angle_degrees` and the bridge density fields. Parse it with `options.range_f64("internal_bridge_angle", 0.0, 0.0, f64::INFINITY)?`, matching Orca's default and minimum.

Keep the current `InfillRole::InternalBridge` mapping from the internal bridge density slice. Rename or generalize the local predicate if useful, but keep its logic unchanged. Use that predicate to select a fixed angle for internal bridge output:

```text
if eligible_internal_bridge(role, options) and internal_bridge_angle > 0:
    fixed_angle_degrees = internal_bridge_angle
else:
    fixed_angle_degrees = None
```

Pass the resulting fixed angle into `InfillPasses::new(...)`. `rotation.rs` already makes a fixed angle take precedence over rotate templates and alternating layer rotation. That mirrors Orca's ordering where `Surface::bridge_angle` overrides normal fill rotation in `FillBase.cpp`.

Do not use `internal_bridge_angle` to classify a layer as an internal bridge. Classification stays owned by the existing `internal_bridge_density < 100` temporary boundary. This preserves default behavior and prevents angle-only options from creating new internal bridge roles before the upstream surface detector is ported.

## Tests

Use TDD with focused RED/GREEN coverage:

- Option parsing:
  - default parses `internal_bridge_angle` as `0.0`;
  - positive numeric and numeric string values parse;
  - below-zero, NaN/infinity strings, nonnumeric strings, booleans, and null are rejected with an error mentioning `internal_bridge_angle`.
- Infill geometry:
  - `internal_bridge_angle = 90` changes only eligible non-default internal bridge paths from the current solid direction to the fixed bridge direction;
  - `internal_bridge_angle = 0` preserves the current generated direction for eligible internal bridge paths;
  - `internal_bridge_angle > 0` does not change dense middle layers that remain ordinary solid infill because `internal_bridge_density == 100`;
  - `internal_bridge_angle > 0` does not change sparse middle layers or no-shell dense layers.
- Pipeline/G-code:
  - non-default `internal_bridge_density = 50` plus `internal_bridge_angle = 90` emits `;PRINT_PATH:internal_bridge:` comments with coordinates matching the fixed bridge direction;
  - `internal_bridge_angle = 0` preserves the current internal bridge G-code direction;
  - default-density output with `internal_bridge_angle > 0` still emits ordinary `solid_infill`, not `internal_bridge`.

## Acceptance Criteria

1. `internal_bridge_angle` has non-test runtime uses in `ares-core` infill generation.
2. Defaults preserve current generated internal bridge geometry direction and ordinary solid infill behavior.
3. Positive `internal_bridge_angle` affects only output that the existing Ares predicate already maps to `InfillRole::InternalBridge`.
4. `internal_bridge_angle` does not create internal bridge paths by itself.
5. Full Orca internal bridge detection and second internal bridge behavior remain explicitly deferred.
6. All touched Rust source files stay at or below 400 LOC.
7. No new crates, dependencies, platform-specific behavior, or Ares-owned pipeline design are introduced.

## Verification

- Targeted RED/GREEN tests:
  - `cargo test -p ares-core --lib internal_bridge_angle`
  - `cargo test -p ares-core --lib internal_bridge_density`
  - `cargo test -p ares-core --lib bridge_angle`
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

Update `docs/roadmap.md` after implementation to state that positive `internal_bridge_angle` now drives concrete internal bridge scanline direction within Ares' current non-default `internal_bridge_density` boundary, while automatic angle detection, true Orca internal bridge surface ownership, and second internal bridge layers remain deferred.
