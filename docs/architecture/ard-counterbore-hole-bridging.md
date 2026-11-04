# ARD — counterbore Filled geometry and lifecycle ownership

Status: proposed. Implementation follows the SPEC and an independent public RED.
Upstream: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Context

[Alignment SPEC](counterbore-hole-bridging-alignment-spec.md) records the exact
rejection, line citations, included/deferred domains and public RED plan.
Classic partial bridging exists, but `classic/preflight.rs::validate_record`
still rejects Filled on holes. A second gap is the absence of Filled's
classification comparison-domain union in `surface_type_detection/stage.rs`.
Removing the guard alone cannot port the feature.

The owning upstream chain is:

- `libslic3r/LayerRegion.cpp:82–142`, `LayerRegion::make_perimeters`: input,
  flow and wall-generator dispatch, not counterbore geometry.
- `libslic3r/PerimeterGenerator.cpp:1214,1728–1930`,
  `process_classic` → `process_no_bridge`: common gates and Filled geometry
  (`:1777–1855`) before walls; generated fill and remaining surfaces.
- `libslic3r/Layer.cpp:248–279`, `Layer::make_perimeters`: group split and
  recovery of extra fill outside region slices into the first merged region.
- `libslic3r/PrintObject.cpp:1520,1579–1583,1690–1707,1912`,
  `PrintObject::detect_surfaces_type`: Filled comparison union, distinct
  original/internal domains, then typed fill clipping.
- `libslic3r/LayerRegion.cpp:63–80`,
  `LayerRegion::slices_to_fill_surfaces_clipped`: type-group intersection with
  the generated fill boundary.

## Non-negotiable decisions

1. **Rewrite the upstream geometry at its owner.** Extend
   `ares-core/project_slice/perimeters/classic/counterbore.rs` with the Filled
   branch, splitting a small sibling as needed below 400 LOC. It remains inside
   the existing prelude, not a post-wall or post-G-code hole patch. Preserve
   operation order, integer scaling, flow parameters, metadata and surface
   mutation order. Reuse the current detector and geometry implementation.
2. **Use existing lifecycle carriers, no duplicate pipeline.** Generated
   `fill_surfaces` / `fill_expolygons` pass through existing layer-region output,
   `groups/redistribute.rs::apply`, then
   `prepare_infill/surface_type_detection/stage.rs`. Port Filled's union for
   top/bottom comparison without incorrectly using that union for every
   original/internal slice operation. Do not mutate shared input slices merely
   to avoid passing the correct classification domain.
3. **Guard removal follows real support.** Filled rejection is a temporary
   compatibility shell. Replace it only when the complete chain is implemented
   and public tests prove it. No fallback to partiallybridge/classic, no skipped
   Filled work, no error-to-success adapter.
4. **Bound scope honestly.** This proposal includes classic Filled and preserves
   classic partial/None. Arachne's upstream `process_arachne:2123` call is
   deferred and stays explicitly unsupported where presently guarded. Keep
   adjacent unsupported domains separate; amend the scope if needed.
5. **Keep platform and test boundaries.** Core remains byte-oriented and
   platform-neutral on WASM, Windows, macOS, Linux; Native and filesystem
   evidence belong to the CLI test adapter. Public `slice_with_profiles` or the
   existing project/load-settings API is the acceptance seam. Private stage
   geometry alone is not E2E parity. Complete ordered bytes retain all commands
   except established generator identity normalization.
6. **Evidence precedes production.** SPEC → independent public RED →
   implementation.
   Keep raw failed/timeout attempts and use the existing
   Native Variant A protocol, never softened assertions or silent retries.
   The canonical local Native wrapper (Variant A SPEC `6becaa454`)
   invokes Native via `setarch -R` (verified SHA256
   `1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`).
   Record the actual wrapper SHA in every new run;
   report any new deterministic mismatch with that
   SHA, rather than presuming it is Variant A.

## Rejected alternatives

- Delete `UnsupportedProjectFeature` without adding geometry: accepts an option
  while producing ordinary holes, losing sacrificial fill.
- Implement only `PrintObject` union: there is no Filled geometry to preserve.
- Implement only `PerimeterGenerator` branch: region split or classification
  can discard the newly filled hole area.
- Make all surface classification consume the union: conflates comparison and
  original/internal domains, contrary to `PrintObject.cpp:1698`.
- Add an Ares-owned counterbore stage or Native fallback: obscures ownership,
  duplicates region state, and breaks browser portability/no-legacy policy.

## Consequences

The smallest correct slice crosses existing geometry, grouping and
classification boundaries. This is necessary upstream lifecycle parity, not
permission for unrelated refactoring. The scope is classic-only and requires
scope and require actual nested-island/extra-fill reachability evidence before
claiming those branches covered. No claim of Native reachability, Rust RED,
GREEN or full option support is made by these docs.
