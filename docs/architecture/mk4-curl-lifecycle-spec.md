# SPEC: Source-owned curled-perimeter lifecycle (`estimate_curled_extrusions`)

Originating witness: MK4 `make_overhang_printable=1` read-only pinpoint
capture.
This is a source-cited rewrite slice of `libslic3r`; it replaces Ares
scaffolding around an upstream concept and creates no Ares-owned pipeline.

## 1. Upstream owner and verbatim contract

Owner: `PrintObject::estimate_curled_extrusions` —
`OrcaSlicer/src/libslic3r/PrintObject.cpp:897-913`, gated by
`enable_overhang_speed` over print regions, calling
`SupportSpotsGenerator::estimate_malformations(this->layers(), params)`.

Producer: `OrcaSlicer/src/libslic3r/Support/SupportSpotsGenerator.cpp:141-196`
(`estimate_malformations`). Verbatim, per layer:

```cpp
LD prev_layer_lines{};
for (Layer *l : layers) {
    l->curled_lines.clear();
    std::vector<Linef> boundary_lines = l->lower_layer != nullptr ? to_unscaled_linesf(l->lower_layer->lslices) : std::vector<Linef>();
    AABBTreeLines::LinesDistancer<Linef> prev_layer_boundary{std::move(boundary_lines)};
    std::vector<ExtrusionLine>           current_layer_lines;
    for (const LayerRegion *layer_region : l->regions()) {
        for (const ExtrusionEntity *extrusion : layer_region->perimeters.flatten().entities) {
            if (extrusion->role() != Slic3r::erExternalPerimeter)
                continue;

            Points extrusion_pts;
            extrusion->collect_points(extrusion_pts);
            float flow_width       = get_flow_width(layer_region, extrusion->role());
```

and the degenerate-first-pair loop (same file, 166-183):

```cpp
const ExtendedPoint<2>& a = i > 0 ? annotated_points[i - 1] : annotated_points[i];
const ExtendedPoint<2>& b = annotated_points[i];
```

Role flow width (`SupportSpotsGenerator.cpp:92`, in `get_flow_width`):

```cpp
if (role == ExtrusionRole::erExternalPerimeter) return region->flow(FlowRole::frExternalPerimeter).width();
```

`LayerRegion::flow(role)` = `m_region->flow(*m_layer->object(), role,
m_layer->height, m_layer->id() == 0)` (`LayerRegion.cpp:21-29`) — a **nominal
region-role flow**, not the emitted path width.

Lifecycle placement (`Print.cpp:2303` non-sequential / `:2393` sequential):
`estimate_curled_extrusions` runs after `infill()` and
`generate_support_material()`/`detect_overhangs_for_lift()` but **before**
`simplify_extrusion_path()` (`Print.cpp:2571`,
`PrintObject::simplify_extrusion_path` `PrintObject.cpp:915-937`). Seam
placement happens far later at emit (`GCode.cpp:5752-5775`: the loop is copied,
`m_seam_placer.place_seam` runs, splitting/reordering points).

Consumer: `ExtrusionQualityEstimator::prepare_for_new_layer`
(`GCode/ExtrusionProcessor.hpp:296-308`) turns `layer->curled_lines` into the
`prev_curled_extrusions` distancer read by `estimate_extrusion_quality`
(`:381-420` artificial distance).

Ordered-input invariants the rewrite must keep:

1. Entities arrive in `l->regions()` then `perimeters.flatten()` order — the
   collection order, not emit/island print order.
2. `collect_points` concatenates the entity's sub-paths in stored order;
   `i == 0` yields a degenerate `ExtrusionLine` at the original first point.
3. `flow_width` is the region-role nominal width; only `l->height` (layer
   height) is per-layer.
4. Curl estimation input is pre-`simplify_extrusion_path` and pre-seam.

## 2. Ares boundary today (compatibility shell to be replaced)

Current Rust collection is emit-time: `seam_placement.rs::apply` (aligned
placement mutating loops via `placement.rs::place_loop` → `split_at`) runs in
`project_slice.rs` before g-code emit; during emit,
`gcode_emit/motion/perimeter.rs:81-84` / `multi_path.rs:18-22` call
`CurlTracker::collect_entity`, and `overhang.rs::estimate` collects open
external fragments. Consequences witnessed on MK4:

- Collector input is post-seam, post-simplify (L1 first point
  `(4500001,4657079)` vs upstream original `(4800001,4500001)`; L37 pre-seam
  entity has 20 points vs Native's original 22).
- Width is `paths[0].width` (actual path width, e.g. `0.39999938011169434`,
  L6 `0.492362380027771`) instead of nominal role flow
  (`0.4000000059604645`).
- Downstream: L36 prior-line selection diverges (curl heights
  `0.4041517674922943` vs `0.8476613759994507`), L37 artificial distances
  cross a speed band only on Ares, cooling input differs
  (7550 vs 7724 bytes), final F `1650` vs `1654`.

The `CurlTracker` module doc already claims `SupportSpotsGenerator.cpp:141-196`
semantics; this slice makes the module's *inputs* actually match that source.

## 3. Included behavior (the correction scope)

Create a source-owned pre-seam/pre-simplify curl estimation stage, ported from
`PrintObject::estimate_curled_extrusions` → `estimate_malformations`:

1. **Timing**: run once per object after perimeter/infill/support stages and
   before `path_simplification::apply` and `seam_placement::apply` in
   `project_slice.rs::slice_source` (mirrors `Print.cpp:2303/2393` before
   `:2571`). Gate: any region with `enable_overhang_speed`, mirroring
   `PrintObject.cpp:899-901`.
2. **Input capture — original ordered perimeters**: consume each layer's
   perimeters as ordered region/collection entities (the
   `PreparedLayerRegionPerimeterRecord::perimeters` /
   `ExtrusionIsland::perimeters` lineage before simplification rewrites
   polylines), flattening each entity's concatenated sub-path points exactly
   like `collect_points` (`ExtrusionEntity.hpp:507-512,431-435`). Filter
   `erExternalPerimeter` entities only. The degenerate `i == 0` pair and the
   closed-loop curvature walk semantics stay as already implemented in
   `curl.rs::collect`.
3. **Nominal role flow width**: feed `get_flow_width(layer_region,
   erExternalPerimeter)` equivalent —
   `perimeters/flow.rs::resolve_nominal_external_perimeter_flow(object,
   region, nozzle_diameters)` (`PrintRegion::flow(frExternalPerimeter,
   layer_height, first_layer)`) — as `flow_width`, with the physical layer's
   `height` and first-layer flag from the owning planned layer. The emitted
   path width no longer participates.
4. **State & consumer**: retain the existing `CurlTracker` layer-chaining
   (`previous`/`previous_curled` roll-over) and `LayerGeometry::
   previous_layer_boundary` (lower-layer `lslices`) inputs; emit-time
   `overhang.rs::estimate` continues reading `previous_curled` through the
   same `prepare_for_new_layer`-equivalent roll-over. Delete the emit-time
   `collect_entity`/fragment collection calls once the pre-emit stage feeds
   the tracker (`perimeter.rs:81-84`, `multi_path.rs:18-22`, the
   `overhang.rs::estimate` collect block) — no dual collection.
5. **Boundary trees**: `estimate_malformations` builds its own
   `prev_layer_boundary` from `l->lower_layer->lslices`; the Rust stage must
   use the same lower-layer slice polygons (existing
   `LineDistanceTree`/`FloatLineDistanceTree` inputs), not emit-time
   `previous_layer_boundary` if that differs in provenance.

Scaffolding replaced: emit-time collection is the temporary compatibility
shell; the source-owned stage becomes the only producer of curled state. No
new public API, no retained diagnostics, no formatter/M73/cooling changes.

## 4. Deferred behavior (explicitly out of scope)

- Support-material curl estimation additions and `Params` beyond what the
  object pass already consumes (`malformation_distance_factors`,
  `max_curled_height_factor`, `curling_tolerance_limit` stay as constants
  already ported).
- `simplify_extrusion_path` itself, arc fitting, and any change to what the
  emitter prints — simplification continues to run exactly as today over the
  same entities; only the curl stage reads the pre-simplify snapshot.
- Seam placement, `GCode::extrude_loop` copying/splitting, cooling buffer,
  M73, formatter — untouched.
- Multi-object/sequential-by-object `Print.cpp:2393` ordering nuances beyond
  per-object invocation order already present in `slice_source`.
- Any change to `ExtrusionQualityEstimator` speed-band math,
  `artificial_distance`, or `speed_for_distance`.

## 5. Expected corrections on the MK4 witness (0-based layers)

From the PINPOINT captures (same fixture):

| Layer | Native (upstream contract) | Ares today | Expected after slice |
| --- | --- | --- | --- |
| L1 | first point `(4800001,4500001)`, width `0.4000000059604645` | `(4500001,4657079)`, `0.39999938011169434` | collector input equals Native original ordered points; nominal width `0.4000000059604645` |
| L6 | 25 pts, 9 curled, nominal `0.4000000059604645` | 26 pts, 11 curled, `0.492362380027771` | 25 pts, curled set expected to match Native's 9 (verify; not assumed) |
| L33 | wing re-activation: `(4948061,0)→same`, height `0.13460035622119904` | no curled lines | the Native curled line is reproduced |
| L36 | prior height `0.06344743818044662` → curl `0.4041517674922943`; 10 curled | `0.5935096144676208` → `0.8476613759994507`; 8 | prior-line selection and curl height match Native; 10 curled |
| L37 | artificial distances `0.097…/0.132…/0.132…`, speeds `124.68559265136719`×3; cooling input 7550 B | `…/0.172…/0.187…`, speeds `…/103/94`; 7724 B | artificial distances/speeds match Native; cooling input equality is the acceptance signal |

The **L37 20-vs-22 point gap is INCLUDED**: it is exactly the pre-simplify
snapshot requirement (upstream reads unsimplified entities). Item 2 of §3
captures original ordered perimeters before `path_simplification::apply`
runs, so the collector receives the 22-point entity. Expected corrections are
verification targets, not guarantees; the slice is validated against Native,
and if a layer still diverges the run stops for re-localization rather than
tuning.

## 6. RED/GREEN and acceptance plan

1. RED first (test-only commit): MK4 `make_overhang_printable` fixture E2E
   (official + seeded) reproducing the stable whole-stream difference
   (byte 1710 `M73 P69/P68`, motion `F1650/F1654` at Z7.6), registered only
   in the group file; root `orca_parity.rs` unchanged.
2. GREEN: implement §3 only. Re-run exact RED in both modes (fresh official,
   fresh seeded) plus API==CLI raw equality, existing curl/overhang controls,
   `cargo nextest run` scoped core suite, fmt, clippy, LOC<400 module split.
3. Native outputs are never committed; a shared Orca cache directory via
   `ARES_ORCA_CACHE_DIR` with `ARES_REQUIRE_ORCA=1`.
4. Exit criteria: whole-stream NULL vs Native on the witness (or an explicitly
   re-localized next first difference reported, not masked); no regression on
   previously NULL machines/cases in the scoped control set.

### GREEN outcome (strict form)

The E2E is a strict whole-stream `native_stream_case::check` (no
pinned-mismatch asserts). The curl lifecycle stage lands the
`G1 F1650` slowdown set exactly (7 occurrences, no `G1 F1654`), and the
former byte-1710 `M73 P69/P68` divergence class is gone. Any remaining
whole-stream residuals are handled as follow-up RED/GREEN links on this
chain, each pinned by a read-only PINPOINT and fixed literally upstream,
until the witness is strict NULL.
