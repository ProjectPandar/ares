# Consume `support_base_pattern` in Support Base Print Paths

## Source Boundary

Upstream OrcaSlicer boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:172-177`: `SupportMaterialPattern` declares `smpDefault`, `smpRectilinear`, `smpRectilinearGrid`, `smpHoneycomb`, `smpLightning`, and `smpNone`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:969`: `support_base_pattern` is a `PrintObjectConfig` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:312-320`: option strings map to support material pattern variants, including `default`, `rectilinear`, `rectilinear-grid`, `honeycomb`, `lightning`, and `hollow`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6133-6156`: the option is labeled "Base pattern", defaults to `smpDefault`, and documents that non-tree default resolves to rectilinear while tree/organic/lightning cases have separate support-style rules.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:122-128`: normal support maps `smpHoneycomb` to `ipHoneycomb`; otherwise current support density and sheath state select rectilinear or support-base fill.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1430-1432`: normal support starts with `base_angle` and adds `interface_angle` only when `support_base_pattern == smpRectilinearGrid`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp:153-163`: `FillSupportBase` is the support-base rectilinear-derived fill used by normal support.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7931-7932`: legacy `support_base_pattern = none` normalizes to `hollow`.

Ares destination boundary:

- `crates/ares-core/src/print_paths`: consume the existing registered option during print-path finalization for current closed rectangular `SupportMaterial` compatibility artifacts.
- `crates/ares-core/src/pipeline/tests`: cover option parsing, rectilinear-grid line families, G-code effects, ordering with adjacent support passes, and deferrals.

This is a source-cited rewrite slice over the current Ares support path scaffold. It does not create a new Ares support pipeline and does not add new public options.

## Current Ares State

Ares already registers `support_base_pattern` with Orca's default `default`, preserves the existing legacy normalization from `none` to `hollow`, and has runtime slices for support expansion, support top-interface role conversion, support base spacing, support interface spacing, support angle, and support ironing. The current base-support path finalizer ignores `support_base_pattern`; every closed rectangular `SupportMaterial` path becomes one family of support lines at `support_angle`.

Ares does not yet have Orca's support-area generation, arbitrary polygon fill clipping, honeycomb fill, lightning/tree support generation, organic support hollow wall handling, raft/first-layer variants, path chaining, or support density/sheath selection beyond the current rectangular compatibility shell.

## Included Behavior

1. Parse `support_base_pattern` from `SliceOptions` during print-path finalization.
2. Use Orca's default `default` when the option is omitted.
3. Accept the Orca enum strings `default`, `rectilinear`, `rectilinear-grid`, `honeycomb`, `lightning`, and `hollow`.
4. Also accept the already-preserved legacy string `grid` as an alias for `rectilinear-grid`, because existing Ares legacy normalization intentionally leaves `support_base_pattern = grid` unchanged.
5. Reject non-string values and unknown strings with `SliceError::InvalidInput` mentioning `support_base_pattern`.
6. For closed rectangular `PrintPathRole::SupportMaterial` paths, `default` and `rectilinear` generate the existing single base line family at `support_angle`.
7. For closed rectangular `PrintPathRole::SupportMaterial` paths, `rectilinear-grid` and `grid` generate two line families:
   - first family at `support_angle`;
   - second family at `support_angle + 90`;
   - both families use the effective pitch already defined by `support_base_pattern_spacing + support material extrusion width`;
   - emitted order is all base-angle lines, followed by all perpendicular lines.
8. For accepted but not-yet-representable `honeycomb`, `lightning`, and `hollow`, keep the current single-family rectangular support-base compatibility output. These values are parsed and tracked but their exact upstream generators remain deferred in this slice.
9. Preserve generated path metadata: role, extrusion role if present, effective layer height, unsupported span, seam gap, layer id, and print Z.
10. Leave non-support roles, `SupportMaterialInterface` paths, non-rectangular support material paths, and non-closed support material paths unchanged.
11. Preserve pass ordering:
    - after support-interface top-layer conversion and support expansion;
    - after `support_angle` parsing;
    - before support-interface spacing, ordinary ironing, and support ironing.
12. When `support_interface_top_layers = 0` converts interface paths to `SupportMaterial`, those converted paths consume `support_base_pattern`.
13. When `support_ironing = true`, remaining `SupportMaterialInterface` paths stay outside this pass and remain available to the existing support-ironing pass.

## Deferred Upstream Behavior

- Full `libslic3r` support-area generation from overhang/contact regions.
- Exact `SupportParameters` density/sheath selection between `ipRectilinear` and `ipSupportBase` beyond the existing rectangular line compatibility shell.
- True `smpHoneycomb` / `FillHoneycomb` path generation.
- Tree, slim, strong, hybrid, organic, and lightning support generators.
- Hollow organic support wall-only behavior and tree default-to-hollow behavior.
- Raft, first-layer, base-interface, support-style, wall-count, and sheath variants.
- Arbitrary polygon clipping, holes, path chaining, link generation, path ordering, and support/object contact splitting.
- Orca binary E2E support parity. This slice starts from Ares' current rectangular support compatibility artifacts; full Orca E2E parity belongs to later support-region and fill-generator rewrite slices.

## Acceptance Criteria

1. With `support_base_pattern` omitted, closed rectangular `SupportMaterial` paths keep the existing single-family default base support output.
2. With `support_base_pattern = "default"` or `"rectilinear"`, closed rectangular `SupportMaterial` paths emit the same single-family base support output.
3. With `support_base_pattern = "rectilinear-grid"`, closed rectangular `SupportMaterial` paths emit the base-angle family and a perpendicular family, increasing support-material line count and changing G-code coordinates.
4. With `support_base_pattern = "grid"`, Ares treats the legacy-preserved alias as rectilinear grid.
5. Rectilinear grid composes with `support_angle`; for example `support_angle = 90` produces vertical lines first and horizontal lines second.
6. Rectilinear grid composes with `support_base_pattern_spacing`; denser spacing generates denser line families for both angles.
7. Generated rectilinear-grid lines preserve source metadata and support-material extrusion role.
8. `support_interface_top_layers = 0` converts interface paths to `SupportMaterial` before base-pattern selection; converted paths then consume `support_base_pattern`.
9. Remaining `SupportMaterialInterface` paths are unchanged by this pass and still flow to support-interface spacing or support ironing.
10. Non-rectangular support material paths, non-closed support material paths, and non-support roles remain unchanged.
11. Accepted but deferred `honeycomb`, `lightning`, and `hollow` values do not error and keep the current rectangular single-family output for this compatibility shell.
12. Invalid values, including unknown strings, numbers, booleans, null, arrays, and objects, return `SliceError::InvalidInput` mentioning `support_base_pattern`.
13. Existing support angle, support interface spacing, support base pattern spacing, support expansion, and support ironing regressions remain passing.

## Docs Impact

- This spec/design and the matching implementation plan are the required design artifacts for the slice.
- Update `docs/roadmap.md` with the completed runtime slice after implementation approval.
- No architecture decision record is required because this slice follows the existing `crates/ares-core/src/print_paths` support-finalization boundary and does not introduce a new crate, dependency, or architectural boundary.

## Safety and Constraints

- No new dependencies.
- `ares-core` remains platform-neutral and WASM-compatible: no direct file I/O, terminal behavior, UI, OpenGL, native viewer runtime, or OS-specific APIs.
- Keep changes scoped to support print-path finalization, tests, and required docs.
- Use `cargo nextest`, not `cargo test`, for Rust test execution.
- Do not change option registration semantics or add new options.
- Do not implement new honeycomb, lightning, tree, organic, or arbitrary-polygon support generators in this slice.

## Verification Plan

- `cargo nextest run -p ares-core support_base_pattern`
- `cargo nextest run -p ares-core support_base_pattern_spacing`
- `cargo nextest run -p ares-core support_angle`
- `cargo nextest run -p ares-core support_interface_spacing`
- `cargo nextest run -p ares-core support_expansion`
- `cargo nextest run -p ares-core support_ironing`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
