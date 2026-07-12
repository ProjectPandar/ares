# Consume Internal Bridge Density Design

## Goal

Consume OrcaSlicer `internal_bridge_density` as concrete Ares slicing behavior. The option is already present in Ares' source-cited PrintConfig metadata and the downstream `InternalBridge` path role already affects speed, flow, fan, extrusion, and G-code comments when a path is constructed manually. This slice must make non-default `internal_bridge_density < 100` change generated dense middle-layer scanline spacing and emit `;PRINT_PATH:internal_bridge:` in Ares' current deterministic layer pipeline while preserving default solid infill behavior.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:987-991` declares the internal thick bridge and internal bridge density fields on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1252-1263` registers `internal_bridge_density` as a percent option, default `100`, min `10`, max `100`, and documents internal bridge spacing control.
- `OrcaSlicer/src/libslic3r/Surface.hpp:23-25` defines `stInternalBridge` and `stSecondInternalBridge` as dense infill bridge surface types above sparse infill.
- `OrcaSlicer/src/libslic3r/Surface.hpp:107-112` treats `stInternalBridge` as bridge, internal bridge, and solid.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1331-1337` applies `bridge_density` to external bridge surfaces and `internal_bridge_density` to internal bridge surfaces, with `dont_adjust = true`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2767-2769` computes rectilinear line spacing as `spacing / params.density`.

## Current Ares State

- Ares has metadata for `internal_bridge_density` under the source-cited `PrintConfig.hpp` milestones, but `crates/ares-core/src/options/infill.rs` does not parse the value into runtime infill options.
- `crates/ares-core/src/surface.rs` already includes `SurfaceType::InternalBridge`, but the active rectangular infill pipeline does not yet own Orca `SurfaceCollection` partitioning.
- `crates/ares-core/src/print_paths.rs` already defines `PrintPathRole::InternalBridge`, and downstream extrusion, speed, fan, and G-code code consume constructed internal bridge paths.
- `crates/ares-core/src/options/infill/layer_role.rs` classifies dense middle layers as `InfillLayerRole::InternalSolid` when `sparse_infill_density == 100`, but generated paths from that role currently become ordinary `PrintPathRole::SolidInfill`.
- `crates/ares-core/src/infills.rs` currently spaces `InfillLayerRole::InternalSolid` at `solid_line_width()` regardless of any internal bridge density option.

## Ares Destination Boundary

Implement the smallest source-cited runtime slice inside `ares-core`:

- Parse `internal_bridge_density` into `InfillOptions` with Orca default `100.0` and range `10.0..=100.0`.
- Add an `InfillRole::InternalBridge` variant for generated infill paths that should become `PrintPathRole::InternalBridge`.
- Treat `InfillLayerRole::InternalSolid` as Ares' current temporary stand-in for Orca `stInternalBridge` only when it appears as a dense middle layer from `sparse_infill_density == 100`, at least one shell-layer setting creates a shell/middle distinction, and `internal_bridge_density < 100.0`.
- For that non-default internal-bridge role, compute scanline spacing as `solid_line_width / (internal_bridge_density / 100.0)`, mirroring Orca's density-to-spacing path.
- Keep `internal_bridge_density = 100` behavior-equivalent to current full solid middle-layer output, including the generated `solid_infill` role, so existing `internal_solid_*` runtime behavior remains intact by default.
- Keep top surface, bottom surface, sparse infill, external bridge, no-shell dense infill, and ordinary non-dense sparse middle layers unchanged.
- Keep implementation WASM-safe and platform-neutral: no filesystem, terminal, UI, OpenGL, native-only behavior, new crates, or new dependencies.

## Explicitly Deferred

- Full Orca `SurfaceCollection` ownership and true `stInternalBridge` / `stSecondInternalBridge` surface generation.
- `enable_extra_bridge_layer`, `dont_filter_internal_bridges`, `internal_bridge_angle`, and complete bridge detector parity.
- Partial per-island internal bridge classification, support-aware bridge ownership, and sparse-infill support interaction.
- Internal bridge flow or speed parsing changes; those existing downstream options remain owned by current bridge/speed/extrusion modules.
- Changes to external `bridge_density`, `bridge_angle`, top/bottom surface density, sparse infill density, surface patterns, gap fill, support, ironing, or shell-thickness expansion.
- New registry metadata. This slice consumes an already recorded option in runtime slicing.

## Design

Add `internal_bridge_density_percent` to `InfillOptions` next to the existing bridge and surface density fields. Expose a const accessor so option tests and infill spacing code can assert parsed values.

Extend `InfillRole` with `InternalBridge`. Keep `InfillLayerRole::infill_role(...)` returning `InfillRole::Solid` for ordinary `InternalSolid` layers. In `infills.rs`, choose `InfillRole::InternalBridge` only when the selected layer role is `InfillLayerRole::InternalSolid`, at least one shell-layer setting is nonzero, and `internal_bridge_density_percent < 100.0`; otherwise keep the existing role mapping. This keeps the non-default internal bridge conversion local and lets `PrintPathRole::InternalBridge` flow through the already implemented downstream path without introducing a new internal bridge detector.

Use one internal bridge eligibility predicate for both spacing and role mapping:

```text
role == InternalSolid
and at least one shell-layer setting is nonzero
and internal_bridge_density < 100.0
```

When that predicate is true, `spacing_for_role(...)` returns:

```text
solid_line_width / (internal_bridge_density / 100.0)
```

When the predicate is false, `InfillLayerRole::InternalSolid` keeps `solid_line_width()`. This mirrors Orca's `Fill.cpp` internal bridge density assignment and `FillRectilinear.cpp` spacing calculation for the non-default Ares temporary internal-bridge boundary while preserving existing no-shell and default solid infill behavior.

In `generate_print_paths(...)`, map generated `InfillRole::InternalBridge` to `PrintPathRole::InternalBridge`. Existing `InfillRole::Solid` continues to use `solid_print_path_role(...)` for bottom/top/external bridge role classification.

## Tests

Use TDD with focused RED/GREEN coverage:

- Option parsing:
  - default parses `internal_bridge_density` as `100.0`;
  - accepts `10`, `100`, and numeric strings in range;
  - rejects below `10`, above `100`, NaN/infinity strings, nonnumeric strings, booleans, and null with an error mentioning `internal_bridge_density`.
- Infill geometry:
  - default dense middle layer generated between bottom and top shells preserves the current scanline count and `InfillRole::Solid`;
  - `internal_bridge_density = 50` increases dense middle-layer spacing, reduces only that layer's generated scanline count, and uses `InfillRole::InternalBridge`;
  - `internal_bridge_density` does not change sparse middle-layer spacing when `sparse_infill_density < 100`;
  - `internal_bridge_density` does not reclassify no-shell dense infill as internal bridge and does not change its line count;
  - top and bottom shell surface counts remain unchanged when internal bridge density changes.
- Pipeline/G-code:
  - default dense middle layer still emits `;PRINT_PATH:solid_infill:` lines;
  - non-default `internal_bridge_density = 50` emits `;PRINT_PATH:internal_bridge:` lines rather than `;PRINT_PATH:solid_infill:` lines for the affected dense middle layer;
  - lowering `internal_bridge_density` reduces internal bridge print path count;
  - downstream speed/flow/G-code existing behavior remains visible through `InternalBridge` role comments.

## Acceptance Criteria

1. `internal_bridge_density` has non-test runtime uses in `ares-core` infill generation.
2. Defaults preserve current generated middle-layer geometry count and `solid_infill` role.
3. Non-default internal bridge density below `100%` affects only eligible dense middle `InfillLayerRole::InternalSolid` output where shell-layer settings create a shell/middle distinction.
4. Sparse infill, top surface, bottom surface, external bridge, bridge angle, bridge density, surface densities, pattern selection, speed parsing, extrusion width, and flow-ratio behavior remain unchanged except through the intended internal bridge path count and role.
5. Full Orca internal bridge detection and extra-layer behavior remain explicitly deferred.
6. All touched Rust source files stay at or below 400 LOC.
7. No new crates, dependencies, platform-specific behavior, or Ares-owned pipeline design are introduced.

## Verification

- Targeted RED/GREEN tests:
  - `cargo test -p ares-core --lib internal_bridge_density`
  - `cargo test -p ares-core --lib internal_bridge`
  - `cargo test -p ares-core --lib surface_density`
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

Update `docs/roadmap.md` after implementation to state that non-default `internal_bridge_density` now drives concrete dense middle-layer internal bridge spacing and `InternalBridge` G-code path counts within Ares' temporary rectangular infill boundary, while defaults preserve existing `solid_infill` output and full Orca internal bridge surface ownership plus adjacent filters/extra-layer behavior remain deferred.
