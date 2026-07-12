# Infill Wall Overlap Runtime Design

## Goal

Implement a concrete `libslic3r::PerimeterGenerator` rewrite slice for `infill_wall_overlap` and `top_bottom_infill_wall_overlap` so rectangular Ares slices generate infill paths that respect wall inset and then overlap back into the wall by the configured amount.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4028-4052` defines `infill_wall_overlap` and `top_bottom_infill_wall_overlap` as percent options with defaults `15` and `25`, both relative to `inner_wall_line_width`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1165-1178` stores `ConfigOptionPercent` as a percentage and converts it to an absolute distance with `get_abs_value(ratio_over)`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1632-1651` computes an infill inset from perimeter spacing, applies top/bottom overlap on first/top-like surfaces, applies sparse/internal overlap otherwise, converts the selected percentage over an inset/solid-spacing runtime base, and subtracts that overlap from the fill boundary inset.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1660-1670` appends the resulting enlarged infill area to fill surfaces.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:2520-2533` applies the same overlap choice in the alternate perimeter generation path.

## Ares Destination Boundary

- `crates/ares-core/src/options/infill.rs` parses the two runtime options into raw percentage values stored on `InfillOptions`.
- `crates/ares-core/src/options/infill.rs` also stores the small perimeter-boundary input needed by the infill stage: wall loop count, external wall line width, internal wall line width, `only_one_wall_first_layer`, `only_one_wall_top`, and `alternate_extra_wall`.
- `crates/ares-core/src/infills/overlap.rs` computes the rectangular clipping contour for the effective layer role.
- `crates/ares-core/src/infills.rs` uses the computed contour while clipping scanline infill for rectangular contours.
- `crates/ares-core/src/pipeline.rs` and pipeline test support keep calling the existing infill stage; no filesystem, terminal, UI, OpenGL, native viewer, or independent Ares pipeline behavior is introduced.

## Included Behavior

- Parse `infill_wall_overlap` and `top_bottom_infill_wall_overlap` as `ConfigOptionPercent`-style raw non-negative percentages. Numeric JSON `15`, string `"15"`, and string `"15%"` all store `15%`, not `15mm`.
- Default sparse/internal overlap to `15%` and top/bottom overlap to `25%`.
- Reject non-finite and negative overlap values.
- For rectangular contours with at least one effective wall loop, clip generated infill to the inner fill boundary produced by the wall loops, then expand that boundary outward by the role-specific overlap amount.
- Use `top_bottom_infill_wall_overlap` on the first layer, the topmost layer, and explicit `BottomSurface` / `TopSurface` roles, matching the upstream first/top-like branch.
- Use `infill_wall_overlap` for non-first, non-top sparse and internal solid roles.
- Clamp the overlap so the adjusted rectangle never expands beyond the original model contour.
- Keep `wall_loops = 0` behavior unchanged: infill remains clipped to the model contour because there is no perimeter boundary to overlap.
- Make the behavior visible through generated print paths and G-code comments, not just parsed option storage.

## Effective Rectangle Formula

For one rectangular contour with bounds `(min_x, min_y, max_x, max_y)`:

- `effective_wall_loops` matches `perimeters.rs` for currently supported modifiers: start from `wall_loops`; if `layer_id == 0 && only_one_wall_first_layer`, use `1`; if `alternate_extra_wall && layer_id` is odd and sparse infill density is positive, add `1`; if the layer is topmost and `only_one_wall_top`, use `1` when more than one loop would otherwise be generated.
- If `effective_wall_loops == 0`, use the original contour.
- For `effective_wall_loops == 1`, base inset is `external_wall_line_width / 2`.
- For `effective_wall_loops >= 2`, base inset is `(external_wall_line_width + internal_wall_line_width) / 2 + (effective_wall_loops - 2) * internal_wall_line_width + internal_wall_line_width / 2`.
- `overlap_reference = external_wall_line_width / 2 + solid_line_width / 2` when `effective_wall_loops == 1`, otherwise `internal_wall_line_width / 2 + solid_line_width / 2`. This is the Ares rectangular equivalent of `PerimeterGenerator.cpp:1645-1648`, where `ConfigOptionPercent::get_abs_value(...)` receives a runtime base derived from perimeter inset plus solid infill spacing, not the option metadata `ratio_over`.
- `role_overlap = selected_overlap_percent / 100 * overlap_reference`.
- `adjusted_inset = max(base_inset - role_overlap, 0)`.
- The rectangular clipping contour is `(min_x + adjusted_inset, min_y + adjusted_inset)` through `(max_x - adjusted_inset, max_y - adjusted_inset)`.
- If the adjusted rectangle collapses, that contour contributes no infill.

This formula mirrors the existing Ares perimeter placement for rectangular loops and the upstream PerimeterGenerator idea of converting a stored `ConfigOptionPercent` over a runtime geometry base, then subtracting that overlap from the fill boundary inset. It is intentionally limited to the classic rectangular path until full polygon offsetting is ported.

## Deferred Behavior

- Full `PerimeterGenerator` `fill_surfaces` and `fill_no_overlap` parity.
- Non-rectangular polygon offsetting, multiple islands, holes with exact offset geometry, Arachne-specific `add_infill_contour_for_arachne`, bridge-only surface expansion, and `top_fills` union behavior.
- The alternate `PerimeterGenerator.cpp:2520-2533` overlap base, which uses `get_abs_value(inset)` in that path, beyond preserving the same top/bottom versus sparse/internal overlap choice.
- Any generated Rust `PrintRegionConfig` class hierarchy or `PRINT_CONFIG_CLASS_DEFINE` expansion.
- Any new crate, dependency, UI, viewer, CLI-only behavior, or filesystem behavior in `ares-core`.

## Acceptance Criteria

- A focused unit test proves middle-layer sparse infill on a 4 mm rectangle with two 0.4 mm wall loops clips to the wall inner boundary at 0.6 mm with zero overlap, and to 0.54 mm when `infill_wall_overlap` is `15%` over the `0.4mm` runtime overlap reference.
- A focused unit test proves first-layer or topmost sparse infill uses `top_bottom_infill_wall_overlap`, producing a `0.5mm` boundary with the default `25%` over the same `0.4mm` runtime overlap reference.
- A focused unit test proves top/bottom solid infill uses `top_bottom_infill_wall_overlap`, producing a different boundary from sparse overlap under the same geometry.
- A focused option test proves defaults parse to raw percentages `15.0` and `25.0`.
- A focused option test proves numeric `20`, string `"20"`, and string `"20%"` all parse to raw percentage `20.0`, and negative/non-numeric values are rejected.
- A pipeline/G-code test proves `infill_wall_overlap` changes emitted middle-layer sparse infill print path comments for a three-layer rectangular pipeline from the zero-overlap boundary `;PRINT_PATH:sparse_infill:1.5,0.6 -> 1.5,3.4` to the default-overlap boundary `;PRINT_PATH:sparse_infill:1.5,0.54 -> 1.5,3.46` under two 0.4 mm wall loops and zero shell layers.
- Existing zero-wall behavior remains unchanged by a regression test.
- `cargo nextest run -p ares-core` passes.
- `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard pass before completion.

## Docs Impact

- Update `docs/roadmap.md` or the current behavior-tracking documentation if it lists `infill_wall_overlap` / `top_bottom_infill_wall_overlap` as metadata-only or deferred. If no such current runtime-status entry exists, no user-facing docs update is required beyond this SDD spec and plan.

## Safety And Simplicity

This is a small source-cited rewrite slice, not a new Ares-owned infill system. The implementation should reuse existing option parsing helpers, existing scanline clipping, and existing rectangle helpers or a tiny rectangle-local helper. It should not add a polygon offset dependency or refactor unrelated perimeter, gap-fill, extrusion, or registry code.
