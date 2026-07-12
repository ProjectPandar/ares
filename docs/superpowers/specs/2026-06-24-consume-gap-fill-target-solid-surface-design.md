# Consume gap_fill_target Solid Surface Gap Fill Design

Consume OrcaSlicer's `gap_fill_target` option into concrete Ares solid-surface gap-fill behavior. This is a source-cited Rust rewrite slice of `libslic3r`; it does not turn `gap_fill_target` into a generic Ares gap-fill switch.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:241-244` declares `GapFillTarget` with `gftEverywhere`, `gftTopBottom`, and `gftNowhere`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:393-398` maps enum strings `everywhere`, `topbottom`, and `nowhere`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1141-1168` defines `gap_fill_target`, default `gftNowhere`, labels the three modes, and documents that classic perimeter-generator gap fill is not controlled by this setting.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:195-244` implements solid-surface gap fill by returning early for `gftNowhere`, returning early for `stInternalSolid` unless the value is `gftEverywhere`, deriving unextruded areas after solid infill, filtering short polylines with `filter_out_gap_fill`, and appending `erGapFill` for non-bridge solid surfaces.

## Ares Destination Boundary

- Parse `gap_fill_target` inside `crates/ares-core/src/options/gap_fill.rs` as an internal enum with the Orca strings and default `nowhere`.
- Add a solid-surface gap-fill generator under the gap-fill module boundary that appends `GapFillPath` entries to `LayerGapFills` after the existing wall/perimeter gap-fill generation. If touching `crates/ares-core/src/gap_fills.rs` would push it over 400 LOC, split the new solid-surface generator and/or existing tests into focused submodules instead of growing the file.
- Feed the parsed target through `crates/ares-core/src/pipeline.rs` and `crates/ares-core/src/pipeline/test_support.rs`. These files are already close to the 400 LOC limit, so the implementation must keep changes there to minimal call-site wiring or move wiring helpers into focused submodules before exceeding the limit.
- Use `crates/ares-core/src/options/infill/layer_role.rs` role classification so `topbottom` applies only to `BottomSurface` and `TopSurface`, while `everywhere` also applies to `InternalSolid`.
- Register new test modules without growing near-limit aggregators past 400 LOC; prefer standalone test files and compact module registration.

## Behavior

Add an internal `GapFillTarget` enum with these accepted values:

- missing or `"nowhere"`: do not add solid-surface gap fill.
- `"topbottom"`: add solid-surface gap fill for bottom and top solid layers only.
- `"everywhere"`: add solid-surface gap fill for bottom, top, and internal solid layers.

Invalid non-string or unknown values return `SliceError::InvalidInput` mentioning `gap_fill_target`.

The new solid-surface generator covers the rectangular contour approximation that Ares can currently express without importing Orca's full polygon/medial-axis stack. It uses the existing infill-generation width source, `InfillOptions::solid_line_width()` derived from `line_width` and nozzle diameter, for every eligible bottom, top, and internal solid role in this slice. It does not use `top_surface_line_width` or `internal_solid_infill_line_width` for path eligibility or inset geometry; those extrusion-width overrides remain downstream extrusion concerns for their own print-path roles, while gap-fill output continues to use the existing gap-fill extrusion width. For an eligible solid-surface layer, if a rectangular contour has a positive short span at most `2 * options.solid_line_width()` and a long span greater than `2 * options.solid_line_width()`, add one centered `GapFillPath` along the long axis, inset by one `options.solid_line_width()` from each end. These paths reuse the existing `PrintPathRole::GapFill`, G-code role comments, gap-fill speed, gap-fill flow ratio, extrusion, print-domain extras, and the existing downstream `filter_out_gap_fill` print-path filter.

Existing wall/perimeter gap fill remains independent. In particular, a narrow rectangular wall gap fill generated from `wall_loops` and `gap_infill_speed` must still be emitted when `gap_fill_target` is missing or `"nowhere"`, matching Orca's documented note that classic perimeter-generator gap fill is not controlled by this setting.

Solid-surface G-code tests must isolate the new generator from existing wall/perimeter gap fill by using fixtures with `wall_loops = 0` or by asserting against dedicated `LayerGapFills` source-path counts before print-path conversion. The old wall/perimeter regression fixture remains separate and intentionally keeps `wall_loops > 0`.

Bridge surfaces must not receive solid-surface gap fill in this slice. Use the existing Ares bridge classification available to the pipeline: suppress solid-surface gap-fill generation when `bridge_no_support` marks the current layer fully unsupported or when `enable_extra_bridge_layer` marks the current layer as an extra external bridge layer. Add a focused regression covering the existing `bridge_no_support` case; exact Orca polygon-level bridge-surface splitting remains deferred.

## Deferred Behavior

- Orca's exact `no_overlap_expolygons`, `polygons_covered_by_spacing`, polygon union/diff/intersection, opening/offset filtering, Douglas-Peucker simplification, medial-axis variable-width polylines, and partial unextruded-area subtraction.
- Non-rectangular contours, holes, multiple regions, partial solid-surface remnants, density-specific solid infill area subtraction, bridge-surface exclusion beyond existing whole-layer Ares bridge classification, and true variable-width extrusion.
- Changes to generated option metadata, UI/preset behavior, support generation, Arachne wall generation, crates, dependencies, file I/O, terminal behavior, OpenGL, or WASM-hostile code.

## Docs Impact

- Update `docs/roadmap.md` after implementation review approval to record this completed source-cited runtime slice and the explicitly deferred Orca medial-axis parity.
- Do not update generated option metadata, UI copy, preset docs, or milestone metadata in this slice.

## Acceptance Criteria

- Runtime option tests prove the default target is `nowhere`, all three enum strings parse, and invalid values fail with `SliceError::InvalidInput` naming `gap_fill_target`.
- Unit tests prove solid-surface gap fill is not generated for `nowhere`, is generated for top/bottom solid roles under `topbottom`, is not generated for internal solid roles under `topbottom`, and is generated for internal solid roles under `everywhere`.
- Pipeline/G-code tests prove a narrow top/bottom solid surface with wall/perimeter gap fill disabled emits `;PRINT_PATH:gap_fill:` only when `gap_fill_target = "topbottom"` or `"everywhere"`.
- Pipeline/G-code tests prove an internal solid layer created by existing Ares layer-role rules, again with wall/perimeter gap fill disabled, emits solid-surface gap fill only when `gap_fill_target = "everywhere"`.
- Pipeline/G-code tests prove an existing `bridge_no_support` bridge layer does not receive solid-surface gap fill even when `gap_fill_target = "everywhere"`.
- Regression tests prove existing wall/perimeter gap fill still reaches G-code when `gap_fill_target = "nowhere"`.
- Regression tests prove `filter_out_gap_fill` removes these new solid-surface gap-fill paths before print-domain extras and G-code.
- `docs/roadmap.md` records the completed `gap_fill_target` runtime behavior slice and the deferred exact Orca medial-axis behavior.
- Full verification must pass with `cargo fmt --check`, focused `cargo nextest run -p ares-core ...`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust file LOC checks.

## Safety

The change stays inside platform-neutral `ares-core` option parsing, contour-derived path generation, and existing G-code path plumbing. It adds no filesystem, terminal, UI, OpenGL, network, dependency, or native-only behavior. Rollback is reverting this spec, the plan, the option parser, the solid-surface gap-fill generator, pipeline wiring, tests, and the roadmap note for this slice.

## Self-Review

- No placeholders or TBD items remain.
- The scope is intentionally limited to solid-surface gap fill because Orca explicitly excludes classic perimeter gap fill from `gap_fill_target`.
- The rectangular approximation is explicit and bounded by existing Ares contour/path capabilities; full Orca medial-axis parity is deferred instead of being approximated silently.
