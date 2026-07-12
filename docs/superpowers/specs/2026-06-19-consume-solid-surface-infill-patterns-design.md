# Consume Solid Surface Infill Patterns Design

## Goal

Consume OrcaSlicer's concrete top, bottom, and internal solid infill pattern selection in Ares. The already registered `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` options must change generated solid infill paths and downstream G-code artifacts inside the current deterministic `sparse_infill_density == 100` scaffold, instead of only existing as option metadata or parser-only fields.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1090-1092` declares `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` as `ConfigOptionEnum<InfillPattern>` region options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1986-2025` registers those three options. Orca defaults `top_surface_pattern` to `ipMonotonicLine`, `bottom_surface_pattern` to `ipMonotonic`, and `internal_solid_infill_pattern` to `ipMonotonic`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:887-899` starts with `sparse_infill_pattern`, then overrides solid external top surfaces with `top_surface_pattern`, solid external bottom surfaces with `bottom_surface_pattern`, and internal solid infill with `internal_solid_infill_pattern`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:910-923` maps those surfaces to `erTopSolidInfill`, `erBottomSurface`, or `erSolidInfill`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:934-941` uses `solid_infill_direction` and `solid_infill_rotate_template` for non-sparse fill roles.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6247-6249` checks `bottom_surface_pattern`, `internal_solid_infill_pattern`, and `top_surface_pattern` against G-code path roles when deciding pattern-sensitive handling.

## Ares Boundary

- Extend `crates/ares-core/src/options/infill.rs` so `InfillOptions` stores parsed `top_surface_pattern`, `bottom_surface_pattern`, `internal_solid_infill_pattern`, and the shell layer counts needed to classify the current Ares dense-infill scaffold.
- Extend `crates/ares-core/src/options/infill/patterns.rs` with key-specific runtime parsers for top and bottom surface patterns. Supported runtime geometry is limited to Ares' existing line-scaffold patterns: `rectilinear`, `alignedrectilinear`, `monotonic`, and `monotonicline`. Unsupported Orca surface patterns such as `concentric`, `hilbertcurve`, `archimedeanchords`, and `octagramspiral` must return `SliceError::InvalidInput` naming the option key and saying the pattern is not implemented.
- Keep the current `generate_infills` public signature. Update `crates/ares-core/src/infills.rs` and `crates/ares-core/src/infills/rotation.rs` so dense solid infill chooses its effective pattern by layer role:
  - bottom shell layers use `bottom_surface_pattern`,
  - top shell layers use `top_surface_pattern`,
  - interior dense layers use `internal_solid_infill_pattern`,
  - bottom-layer precedence wins for a single-layer dense scaffold.
- Preserve existing sparse infill behavior for `sparse_infill_density < 100`, including `sparse_infill_pattern`, `infill_direction`, `sparse_infill_rotate_template`, symmetric Y-axis handling, anchors, combination, and downstream role strings.
- Do not add option registry metadata, new crates, dependencies, filesystem, terminal, UI, OpenGL, or viewer behavior.

## Included Behavior

- Missing `top_surface_pattern` defaults to `monotonicline`.
- Missing `bottom_surface_pattern` defaults to `monotonic`.
- Missing `internal_solid_infill_pattern` continues to default to `monotonic`.
- For `sparse_infill_density == 100`, Ares chooses the effective pattern using the same shell-layer role boundary that currently maps solid paths to `bottom_surface`, `solid_infill`, and `top_solid_infill`.
- For top and bottom dense layers, `solid_infill_direction` and `solid_infill_rotate_template` remain the direction/template source, matching Orca's non-sparse fill-role path.
- `alignedrectilinear` remains fixed to the configured direction on odd layers, while `rectilinear`, `monotonic`, and `monotonicline` keep the current Ares line-scaffold alternating-angle behavior. This makes top/bottom pattern choices visible in generated paths without claiming full Orca monotonic ordering parity.
- Pipeline and G-code comments must show changed coordinates for the affected `;INFILL:solid:`, `;PRINT_PATH:bottom_surface:`, `;PRINT_PATH:solid_infill:`, or `;PRINT_PATH:top_solid_infill:` entries when a selected pattern changes the generated passes.
- Invalid `top_surface_pattern` and `bottom_surface_pattern` values fail during `SliceOptions::infill_options()` parsing with `SliceError::InvalidInput` mentioning the offending key.

## Deferred Behavior

- Full Orca surface discovery from `SurfaceCollection`, sloped-surface shell expansion, bridge bottom surfaces, support surfaces, ironing surfaces, and multi-region fill batching.
- Full geometric implementations for `concentric`, `hilbertcurve`, `archimedeanchords`, `octagramspiral`, and true monotonic/monotonic-line ordering. This slice only uses the existing deterministic line scaffold for implemented pattern values.
- `top_surface_density`, `bottom_surface_density`, `min_width_top_surface`, ironing pattern behavior, bridge-specific pattern handling, object/region extruder routing, and travel optimization.
- Any change to registry metadata, validation metadata tables, CLI behavior, WASM adapter behavior, or Orca end-to-end binary comparison.

## Acceptance Criteria

1. Options tests prove `top_surface_pattern` defaults to `monotonicline`, `bottom_surface_pattern` defaults to `monotonic`, both parse `rectilinear`, `alignedrectilinear`, `monotonic`, and `monotonicline`, and both reject unsupported/unknown values with `SliceError::InvalidInput` naming the key.
2. Infill unit tests prove a dense bottom shell layer can use `bottom_surface_pattern = alignedrectilinear` to keep the configured solid direction on an odd bottom shell layer where the previous internal-solid pattern would have rotated.
3. Infill unit tests prove a dense top shell layer can use `top_surface_pattern = alignedrectilinear` independently of `internal_solid_infill_pattern`.
4. Infill unit tests prove an interior dense layer still uses `internal_solid_infill_pattern`, including the existing `grid` behavior, and sparse density below 100 remains controlled by `sparse_infill_pattern`.
5. Pipeline/G-code tests prove configured top and bottom surface patterns change real generated artifacts after print-path role classification, including `bottom_surface` and `top_solid_infill` print path comments.
6. Existing sparse infill pattern, sparse rotate template, 100% internal solid, top/bottom numeric solid-surface, shell-layer role, and internal-solid numeric tests continue to pass.
7. All touched Rust files under `crates/` remain at or below 400 LOC.

## LOC-Safe Implementation Guidance

- Do not add tests to `crates/ares-core/src/options/tests.rs`; it is already at the 400 LOC limit. Put parser tests in the existing `crates/ares-core/src/options/tests/internal_solid_infill.rs` module.
- Add infill behavior tests to `crates/ares-core/src/infills/tests/internal_solid.rs`, which is still small.
- Add pipeline coverage in a new `crates/ares-core/src/pipeline/tests/solid_surface_patterns.rs` module and register only that module in `crates/ares-core/src/pipeline/tests.rs`.
- Keep `crates/ares-core/src/infills.rs` under 400 LOC by making only the small layer-index/threading change there and putting role/pattern decisions on `InfillOptions`.

## Verification

- RED/GREEN targeted tests:
  - `cargo test -p ares-core --lib options::tests::internal_solid_infill`
  - `cargo test -p ares-core --lib infills::tests::internal_solid`
  - `cargo test -p ares-core --lib pipeline::tests::solid_surface_patterns`
- Regression tests:
  - `cargo test -p ares-core --lib sparse_infill_pattern`
  - `cargo test -p ares-core --lib sparse_infill_rotate_template`
  - `cargo test -p ares-core --lib internal_solid_infill`
  - `cargo test -p ares-core --lib top_bottom_solid_surface`
  - `cargo test -p ares-core --lib internal_solid_numeric`
- Full verification:
  - `cargo fmt --check`
  - `cargo test -p ares-core --lib`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `find crates -path '*/src/*.rs' -o -path '*/src/**/*.rs' | sort | xargs wc -l | awk '$1 > 400 && $2 != "total" {print; bad=1} END {exit bad}'`

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with the spec, reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

This spec and its implementation plan are the documentation artifacts for the slice. No CLI, WASM, or user-facing docs are required because the public byte-in/options-to-byte-output API remains unchanged.
