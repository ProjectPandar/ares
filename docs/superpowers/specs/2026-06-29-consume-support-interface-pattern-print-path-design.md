# Consume `support_interface_pattern` in Support Interface Print Paths

## Source Boundary

Upstream OrcaSlicer boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:190-192`: `SupportMaterialInterfacePattern` declares `smipAuto`, `smipRectilinear`, `smipConcentric`, `smipRectilinearInterlaced`, and `smipGrid`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:333-340`: option strings map to support interface pattern variants: `auto`, `rectilinear`, `concentric`, `rectilinear_interlaced`, and `grid`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6158-6176`: `support_interface_pattern` is labeled "Interface pattern", defaults to `smipAuto`, and documents rectilinear as the default non-soluble interface pattern and concentric as the default soluble interface pattern.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:103-138`: support parameters derive `interface_angle` as `support_angle + 90`, map `smipGrid` to `ipGrid`, map `smipRectilinearInterlaced` to `ipRectilinear`, and otherwise choose concentric or rectilinear/support-base fill depending on `smipAuto`, `smipConcentric`, zero-gap interface, and top-interface density.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1563-1592,1694-1733`: normal support creates `filler_interface` from `contact_fill_pattern`, selects a support-interface angle, and emits top/bottom/interface contact paths with `ExtrusionRole::erSupportMaterialInterface` unless a layer is explicitly treated as base support.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:1497-1509,1554-1585`: tree support uses the same contact fill pattern concept; grid uses `base_support_angle`, while rectilinear interlaced fixes the angle by interface id.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp:67-77` and `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:3422-3438`: `FillGrid` is a rectilinear-derived self-crossing fill that emits two perpendicular multiline families.

Ares destination boundary:

- `crates/ares-core/src/print_paths/support_interface_spacing.rs`: consume the registered option during print-path finalization for current closed rectangular `SupportMaterialInterface` compatibility artifacts.
- `crates/ares-core/src/pipeline/tests/support_interface_pattern.rs`: cover option parsing, current rectangular line-family selection, G-code effects, ordering with adjacent support passes, and deferrals without growing the existing support-interface-spacing test file past the 400 LOC split threshold.

This is a source-cited rewrite slice over the current Ares support-interface path scaffold. It does not create a new Ares support pipeline and does not add new public options.

## Current Ares State

Ares already registers `support_interface_pattern` with Orca's default `auto`. Runtime slices currently consume support expansion, top-interface layer conversion, support base pattern spacing, support base pattern, support interface spacing, support angle, and support ironing. The current support-interface path finalizer ignores `support_interface_pattern`; every closed rectangular `SupportMaterialInterface` path becomes one family of support-interface lines at `support_angle + 90`, unless support ironing keeps the solid interface rectangle for ironing.

Ares does not yet have Orca's support-area generation, arbitrary polygon fill clipping, concentric fill, support-interface loop generation, soluble-interface material handling, raft/contact layer separation, tree/organic support interface generation, per-interface-id angle alternation, path chaining, or full `ipGrid` fill parity.

## Included Behavior

1. Parse `support_interface_pattern` from `SliceOptions` during print-path finalization.
2. Use Orca's default `auto` when the option is omitted.
3. Accept the Orca enum strings `auto`, `rectilinear`, `concentric`, `rectilinear_interlaced`, and `grid`.
4. Reject non-string values and unknown strings with `SliceError::InvalidInput` mentioning `support_interface_pattern`.
5. For closed rectangular `PrintPathRole::SupportMaterialInterface` paths, `auto` and `rectilinear` generate the existing single interface line family at `support_angle + 90`.
6. For closed rectangular `PrintPathRole::SupportMaterialInterface` paths, `grid` generates a bounded rectangular compatibility approximation of Orca `ipGrid`: two perpendicular line families over the current rectangle scaffold.
   - first family at the current interface angle, `support_angle + 90`;
   - second family at the base support angle, `support_angle`;
   - both families use the effective pitch already defined by `support_interface_spacing + support interface extrusion width`;
   - emitted order is all interface-angle lines, followed by all base-angle lines.
   - This angle choice follows the current Ares interface-line scaffold and the normal-support default interface-angle concept. It is not a claim of full parity with Orca's normal-support `raft_interface_angle(...)` branches or tree-support grid angle override.
7. For accepted but not-yet-representable `concentric` and `rectilinear_interlaced`, keep the current single-family rectangular interface compatibility output. These values are parsed and tracked but their exact upstream generators remain deferred in this slice.
8. Preserve generated path metadata: role, extrusion role if present, effective layer height, unsupported span, seam gap, layer id, and print Z.
9. Leave non-interface roles, `SupportMaterial` paths, non-rectangular interface paths, and non-closed interface paths unchanged.
10. Preserve pass ordering:
    - after support-interface top-layer conversion and support expansion;
    - after `support_angle` parsing;
    - after support base pattern spacing/base pattern;
    - before ordinary ironing and support ironing.
11. When `support_interface_top_layers = 0` converts interface paths to `SupportMaterial`, those converted paths do not consume `support_interface_pattern`; they flow to the existing support base pattern/base spacing behavior.
12. When `support_ironing = true`, keep the current solid interface rectangle behavior; parse and validate `support_interface_pattern`, but do not convert the interface rectangle to pattern lines before the support-ironing pass.

## Deferred Upstream Behavior

- Full `libslic3r` support-area generation from overhang/contact regions.
- Exact `ipGrid` fill generator parity, including Orca's `raft_interface_angle(...)` selection, tree-support `base_support_angle` override, internal path linking, fill sorting, layer-id reversal, multiline/trapezoidal behavior, and polygon clipping.
- Exact `smipAuto` soluble-interface resolution and zero-gap concentric selection.
- True `smipConcentric` / concentric support-interface path generation.
- True `smipRectilinearInterlaced` per-interface-id angle alternation and fixed-angle semantics.
- Raft contact, first-layer contact, base-interface, bottom-interface, bridge-flow, loop-interface, and support-style variants.
- Tree, slim, strong, hybrid, organic, and lightning support interface generation.
- Arbitrary polygon clipping, holes, path chaining, link generation, path ordering, and support/object contact splitting.
- Orca binary E2E support parity. This slice starts from Ares' current rectangular support compatibility artifacts; full Orca E2E parity belongs to later support-region and fill-generator rewrite slices.

## Acceptance Criteria

1. With `support_interface_pattern` omitted, closed rectangular `SupportMaterialInterface` paths keep the existing single-family default interface output.
2. With `support_interface_pattern = "auto"` or `"rectilinear"`, closed rectangular `SupportMaterialInterface` paths emit the same single-family interface output at `support_angle + 90`.
3. With `support_interface_pattern = "grid"`, closed rectangular `SupportMaterialInterface` paths emit the interface-angle family followed by a base-angle family, increasing support-interface line count and changing G-code coordinates.
4. Grid composes with `support_angle`; for example `support_angle = 90` produces horizontal interface-angle lines first and vertical base-angle lines second.
5. Grid composes with `support_interface_spacing`; denser spacing generates denser line families for both angles.
6. Generated grid lines preserve source metadata and support-interface extrusion role.
7. `support_interface_top_layers = 0` converts interface paths to `SupportMaterial` before interface-pattern selection; converted paths do not emit interface-pattern lines.
8. `support_ironing = true` keeps the solid support interface rectangle for support ironing even when `support_interface_pattern = "grid"`.
9. Non-rectangular interface paths, non-closed interface paths, and non-interface roles remain unchanged.
10. Accepted but deferred `concentric` and `rectilinear_interlaced` values do not error and keep the current rectangular single-family output for this compatibility shell.
11. Invalid values, including unknown strings, numbers, booleans, null, arrays, and objects, return `SliceError::InvalidInput` mentioning `support_interface_pattern`, including when `support_ironing = true`.
12. Existing support angle, support interface spacing, support base pattern, support base pattern spacing, support expansion, and support ironing regressions remain passing.

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
- Do not implement new concentric, interlaced, tree, organic, or arbitrary-polygon support generators in this slice.
- Keep touched Rust files under the project split threshold. Add the new pattern tests in `support_interface_pattern.rs` instead of expanding `support_interface_spacing.rs` if the existing file would exceed 400 LOC.
- No bounded OrcaSlicer binary E2E comparison is meaningful for this slice because current Ares only owns hand-constructed rectangular support-interface compatibility artifacts, while Orca's `support_interface_pattern` behavior depends on upstream support-region/contact generation and fill clipping that are explicitly deferred. Replacement verification is source-cited characterization plus targeted Rust finalization/G-code tests over the current Ares artifact boundary, with full Orca binary E2E deferred to the later support-region/fill-generator parity slices.

## Verification Plan

- `cargo nextest run -p ares-core support_interface_pattern`
- `cargo nextest run -p ares-core support_interface_spacing`
- `cargo nextest run -p ares-core support_base_pattern`
- `cargo nextest run -p ares-core support_base_pattern_spacing`
- `cargo nextest run -p ares-core support_angle`
- `cargo nextest run -p ares-core support_expansion`
- `cargo nextest run -p ares-core support_ironing`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
