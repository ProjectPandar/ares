# Consume `support_angle` in Support Print-Path Direction

## Source Boundary

Upstream OrcaSlicer boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:952`: `support_angle` is a `PrintObjectConfig` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5949-5957`: the option is labeled "Pattern angle", uses degrees, has minimum `0`, maximum `359`, and defaults to `0`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:103-104`: Orca derives `base_angle` from `support_angle` and `interface_angle` from `support_angle + 90`.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1589-1592`, `1711-1717`, and `1767`: support interface and base fill assign filler angles from the support parameters while generating support extrusion paths; this slice maps the current Ares rectangular scaffold to the `smsGrid`/`smipRectilinear` branch that uses `support_params.interface_angle`.

Ares destination boundary:

- `crates/ares-core/src/print_paths`: consume the existing option during print-path finalization for the current rectangular `SupportMaterial` and `SupportMaterialInterface` compatibility artifacts.
- `crates/ares-core/src/pipeline/tests`: cover option parsing, base/interface path orientation, emitted G-code, and ordering against adjacent support finalization passes.

This is a source-cited rewrite slice over the current Ares support path scaffold. It does not create a new Ares support pipeline and does not add new public options.

## Current Ares State

Ares already registers `support_angle` and migrates legacy `support_material_angle` to it, but the runtime print-path finalization does not consume the value. Current `support_base_pattern_spacing` and `support_interface_spacing` passes convert closed rectangular support regions into horizontal open line paths. Ares does not yet have Orca's full support polygon generation, filler pattern selection, arbitrary polygon clipping, or layer-alternating support fill machinery.

## Included Behavior

1. Parse `support_angle` from `SliceOptions` during print-path finalization.
2. Accept finite JSON numbers and numeric strings as degrees.
3. Reject negative values, values greater than `359`, non-finite values, unit strings, booleans, nulls, arrays, and objects with `SliceError::InvalidInput` mentioning `support_angle`.
4. Use Orca's default value `0` when the option is omitted.
5. For closed rectangular `PrintPathRole::SupportMaterial` paths converted by the base pattern spacing pass, generate open support-material line paths at `support_angle` degrees.
6. For closed rectangular `PrintPathRole::SupportMaterialInterface` paths converted by the interface spacing pass, generate open support-interface line paths at `support_angle + 90` degrees.
7. Preserve the existing pitch calculation from the spacing passes:
   - base support pitch remains `support_base_pattern_spacing + extrusion_options.width_for_role(PrintPathRole::SupportMaterial)`;
   - interface pitch remains `support_interface_spacing + extrusion_options.width_for_role(PrintPathRole::SupportMaterialInterface)`.
8. Preserve source path metadata on generated lines: role, extrusion role if present, effective layer height, unsupported span, seam gap, layer id, and print Z.
9. Leave non-support roles, non-rectangular support paths, and non-closed support paths unchanged.
10. Preserve support pass ordering:
    - run after support-interface top-layer conversion and support expansion;
    - run before support ironing;
    - if `support_interface_top_layers = 0` converts interface paths to `SupportMaterial`, those paths use the base `support_angle`, not `support_angle + 90`.
11. When `support_ironing = true`, keep the current interface-spacing skip behavior; `support_angle` must not independently convert the solid interface rectangle before support ironing.
12. With the default `support_angle = 0`, base support lines remain horizontal and interface support lines become vertical because Orca defines interface angle as base angle plus 90 degrees.

## Rotated Rectangle Contract

The current Ares support artifacts are rectangles, so this slice defines a deterministic rectangle-only line generator:

1. Normalize the effective angle with Euclidean modulo `360` before trigonometry. `support_angle` itself still validates against Orca's configured range `0..=359`; the modulo exists for derived interface angles such as `support_angle + 90`.
2. Use direction vector `d = (cos(theta), sin(theta))`.
3. Use normal vector `n = (-sin(theta), cos(theta))`, then flip `n` when `n.y < 0` or when `abs(n.y) <= EPSILON && n.x < 0`. This keeps `0` degrees anchored from `min_y` upward and `90` degrees anchored from `min_x` rightward.
4. Project the four rectangle corners onto `n` and iterate offsets from the minimum projection to the maximum projection, inclusive, by effective pitch.
5. For each offset, intersect the infinite line `dot(point, n) = offset` with the four rectangle edges.
6. Deduplicate intersection points within `EPSILON`; emit a line only when the clipped chord has positive length and at least two unique points.
7. Sort each emitted chord's two endpoints by increasing projection onto `d`.
8. Emit chords in increasing offset order.

Representative non-axis-aligned acceptance case: for rectangle `(1,1)..(3,2)`, base `support_angle = 45`, and effective pitch `1.0`, the first corner-only offset is skipped and the emitted diagonal support lines are approximately:

- `(1.5857864376, 1.0) -> (2.5857864376, 2.0)`
- `(1.0, 1.8284271247) -> (1.1715728753, 2.0)`

Tests may use tolerant floating-point assertions for rotated coordinates.

## Deferred Upstream Behavior

- Full `libslic3r` support-area generation from overhang regions.
- Exact Orca fill pattern machinery, including `support_base_pattern`, `support_interface_pattern`, grid, honeycomb, rectilinear interlaced, lightning, hollow, and first-layer/raft variants.
- Support layer angle alternation beyond the cited `support_angle` and `support_angle + 90` mapping for current rectangular artifacts.
- Filling arbitrary polygons, holes, clipping against non-rectangular regions, chaining, path ordering, and support/object contact region splitting.
- `support_ironing_angle` behavior, which belongs to support ironing slices rather than this support base/interface direction slice.
- Tree and organic support behavior.
- Orca end-to-end support parity tests for generated support regions. This slice starts from Ares' current rectangular support compatibility artifacts, not Orca-generated support polygons, so a direct Orca E2E comparison would conflate support-region generation gaps with this option-consumption change. The next-best source-cited regression checks are the cited `SupportParameters` angle mapping, deterministic rectangle clipping tests, and G-code coordinate assertions over finalized Ares support paths. Full Orca E2E parity belongs to the later support-area generation rewrite slice that owns `SupportCommon.cpp` region generation.

## Acceptance Criteria

1. With `support_angle` omitted, closed rectangular `SupportMaterial` paths still emit horizontal base-support lines.
2. With `support_angle` omitted, closed rectangular `SupportMaterialInterface` paths emit vertical interface-support lines, matching Orca's `support_angle + 90` interface angle.
3. With `support_angle = 90`, closed rectangular `SupportMaterial` paths emit vertical base-support lines.
4. With `support_angle = 90`, closed rectangular `SupportMaterialInterface` paths emit horizontal interface-support lines at the effective 180-degree interface angle.
5. With an arbitrary non-axis-aligned angle such as `45`, generated support lines are clipped to the rectangular bounds and include diagonal open paths.
6. Generated support lines preserve source metadata and role.
7. `support_interface_top_layers = 0` converts support interface paths to `SupportMaterial` before spacing; those converted paths then use the base support angle.
8. `support_ironing = true` leaves closed support-interface rectangles available for the existing support-ironing pass and does not apply support-interface spacing or the interface angle first.
9. Non-rectangular support paths, non-closed support paths, and non-support paths are unchanged.
10. Invalid `support_angle` values return `SliceError::InvalidInput` mentioning `support_angle`.
11. Existing support interface spacing and support expansion tests are updated where the new upstream-correct default interface angle intentionally changes finalized interface path orientation and G-code coordinates.
12. Existing support base pattern spacing tests continue to prove default base support remains horizontal and are extended or adjusted to cover explicit base-angle rotation.

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
- Do not add backwards-compatibility shims beyond the already-existing legacy `support_material_angle` migration.

## Verification Plan

- `cargo nextest run -p ares-core support_angle`
- `cargo nextest run -p ares-core support_interface_spacing`
- `cargo nextest run -p ares-core support_base_pattern_spacing`
- `cargo nextest run -p ares-core support_expansion`
- `cargo nextest run -p ares-core support_ironing`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
