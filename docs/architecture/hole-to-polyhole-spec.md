# SPEC: `hole_to_polyhole` — convert circular holes to polyholes

Approved proposal: `cfa2302b4` (`docs/architecture/hole-to-polyhole-discovery.md`).
Base: main `8f6ce1660`. All bounds are upstream-cited; no Ares-owned
pipeline design.

## Upstream source

| Piece | Location |
|---|---|
| Call site | `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp:847` — inside `PrintObject::slice()`, after `slice_volumes()` + cancel check and the `#if 0` warning block, **before** `groupingVolumesForBrim` (:849) and before the `lslices_bboxes`/`backup_untyped_slices` parallel pass (:851-867). |
| Transform | `OrcaSlicer/src/libslic3r/PrintObject.cpp:189-295` — `PrintObject::_transform_hole_to_polyholes()` |
| Polyhole builder | `OrcaSlicer/src/libslic3r/PrintObject.cpp:160-186` — free function `create_polyholes(center, radius, nozzle_diameter, multiple)` |
| Option defs | `PrintConfig.cpp:7093-7120` (`hole_to_polyhole` bool=false; `hole_to_polyhole_threshold` FloatOrPercent=0.01 mm, max_literal 10; `hole_to_polyhole_twisted` bool=true) |
| Invalidation | `PrintObject.cpp:1201-1203` (three keys invalidate the slice step); `Preset.cpp:1275-1277` (object-scope keys) |

## Upstream algorithm (included)

`_transform_hole_to_polyholes` (`PrintObject.cpp:189-295`):

1. **Detection** (:197-243): per layer, per `LayerRegion` — only when
   `region().config().hole_to_polyhole` — scan `slices.surfaces`
   `expolygon.holes`. A hole qualifies when **all** of:
   - `hole.convex_points(PI).empty()` — no concave vertex (hole is
     clockwise, so convexity test is inverted).
   - `hole.points.size() > 8`.
   - Vertex-radius variation: `diameter_max - diameter_min < 2 *
     max_variation`, where `diameter_*` are min/max distances from the
     hole `centroid()` to its vertices, and
     `max_variation = max(SCALED_EPSILON,
     scale_(threshold.get_abs_value(unscaled(mean_radius))))`.
   - Edge-midpoint-radius variation: same bound applied to distances
     from centroid to each `hole.lines()` midpoint — excludes rounded
     rectangles.
   - `threshold` semantics: `FloatOrPercent`; `get_abs_value(r)` yields
     the percent fraction of radius when `percent` else the literal mm
     value; result scaled to internal coords.
2. **Grouping** (:246-280): key each candidate by
   `(center, diameter_max, outer_wall_filament_id, max_variation,
   twist)`. For each ungrouped hole at layer `l`, walk upward
   collecting holes with matching extruder id, center distance <
   `max_variation`, diameter difference < `max_variation` across
   *contiguous* layers (break when `print_z - height - max_z >
   EPSILON`). Keep groups with `holes.size() >= 2` **or** a single
   layer-0 hole (first-layer-compensation exception,
   `min_nb_layers = 2`, :250/:278).
3. **Replacement** (:281-294): for each group call
   `create_polyholes(center, diameter_max,
   scale_(nozzle_diameter[outer_wall_filament_id - 1]), twist)`;
   assign `polyholes[layer_idx % polyholes.size()]` per layer, then
   write `polyhole.points` both into the region surface hole **and**
   into the matching `m_layers[z]->lslices` hole (identified by point
   equality).

`create_polyholes` (:160-186): `nb_edges = max(3, round(4.0 *
unscaled(radius) * 0.4 / unscaled(nozzle_diameter)))`; `new_radius =
radius / cos(PI / nb_edges)` (circumscribed, Hydraraptor polyhole).
When `twisted`: `nb_polyhole = 5`, `rotation = 2PI/(nb_edges*5)`, five
pre-rotated variants interleaved (`(i%2)==0` → first half else
second-half index) and assigned round-robin by layer index; otherwise a
single polygon. All output polygons `make_clockwise()` (hole
orientation).

## Ares destination boundary

Ares parses all three keys into `RegionOptions`
(`options/region_fields.rs`) but has **no consumer** — the transform
itself is entirely absent; this is new-port scope, not a compatibility
shell.

- **New module** `crates/ares-core/src/project_slice/hole_to_polyhole.rs`
  (plus `hole_to_polyhole/` split files if it nears 400 LOC; expected
  ≈250 LOC total).
- **Pipeline slot**: new `load_stages` stage run immediately after
  `prepare_post_regions` produces the composed `region_slices` (the
  Ares analogue of `slice_volumes()` output: per-layer region slices as
  typed `ExPolygon`s with holes) and **before**
  `prepare_post_top_empty_layers`/`prepare_post_conical_overhang`.
  Rationale: upstream runs the transform inside `slice()` after
  `slice_volumes()` and before brim regrouping; Ares's brim grouping
  consumes the same post-region-slice geometry, so mutating region
  slices in place at this point preserves upstream ordering.
- The stage iterates objects → layers → region slices; uses
  `regions[region_id].options.hole_to_polyhole*` from the resolved
  graph (region-scoped, matching `region().config()` at
  `PrintObject.cpp:201`), `outer_wall_filament_id` for nozzle selection
  (`PrintObject.cpp:282`), and writes the replacement polygon points
  into both the region surface hole and the layer `lslices`
  counterpart identified by point equality (:285-292).
- Helpers needed: `convex_points(PI)`-equivalent concavity check
  (existing `Polygon` geometry), `centroid`, `lines` midpoints — all
  present in `geometry/polygon`/`expolygon`; `FloatOrPercent`
  `get_abs_value` — existing `helpers::absolute` semantics.

## Included vs deferred

**Included (this milestone):**
- All three option keys consumed.
- Detection: convexity + `>8` points + vertex-radius and
  edge-midpoint-radius variation within `2 * max_variation`.
- `threshold` FloatOrPercent full semantics (mm or % of mean radius).
- Cross-layer grouping: extruder-id match, center/diameter tolerance
  `max_variation`, contiguous-layer requirement, `>=2` group minimum
  with layer-0 singleton exception.
- `twisted` = 5 rotated variants interleaved + round-robin by layer;
  untwisted = single polygon.
- Replacement into region surface holes and layer `lslices` (point-
  equality match).
- Parallelism optional — upstream uses `tbb`; sequential is acceptable
  first port (deterministic either way).

**Deferred (explicitly out of scope):**
- `hole.convex_points(PI)` exact upstream tolerance semantics beyond
  the concavity predicate — match the predicate result, not a
  reimplementation of the tolerance parameter.
- Multi-object/instance interactions beyond per-`PrintObject` slicing
  (upstream scopes the transform to `this` PrintObject — same in Ares:
  per prepared post-region object).
- UI/preset wiring (keys already parse and serialize).
- `fix_slicing_errors` warning block (`#if 0` upstream — dead code,
  never ported).

## Frozen RED witnesses (proposed)

Fixture root: `${LOCAL_EVIDENCE_ROOT}/worker4/polyhole-discovery/inputs/`
(generated; `gen.py` in same root — 30×30×8 block, centered r=5 mm
64-gon hole, fixture `project_settings.config`). One new
`orca_parity/option_coverage/hole_to_polyhole.rs` test:

| Case | Input | Expected on RED | Post-GREEN |
|---|---|---|---|
| `hole_to_polyhole_enabled` | `polyhole.3mf` (`683d7a1d`, `hole_to_polyhole=1`) | FAIL — Ares emits circular hole, native emits polyhole; first differing line `plate_1.gcode:107` | whole-output PASS |
| `hole_to_polyhole_twisted` | `polyhole-twisted.3mf` (`683d7a1d`, `hole_to_polyhole=1` + `hole_to_polyhole_twisted=1` default) | FAIL — same divergence; rotated variants visible from layer 2 | PASS |
| `hole_to_polyhole_threshold_miss` | `polyhole-threshold.3mf` (`hole_to_polyhole=1`, `hole_to_polyhole_threshold=0` → strict variation bound misses real tessellation noise? see note) | control-class: native keeps circle iff detection fails | PASS |
| `hole_to_polyhole_disabled` | `control.3mf` (`f9921c8a`, no override) | PASS control — already byte-equal modulo generator line | PASS (regression) |

Note on threshold-miss: with a perfect 64-gon the vertices are already
on-circle, so `threshold=0` may still detect (variation ≈ 0 <
`2*max(SCALED_EPSILON,0)`); the leg is only kept if the generated
fixture's mid-edge distance vs vertex distance actually exceeds the
bound — edge midpoints of a regular polygon sit at `r*cos(PI/n)`, so
for n=64 the spread is `r*(1-cos(2.8°))` ≈ 0.009 mm > 2*SCALED_EPSILON,
meaning `threshold=0` **does** reject (only vertex test passes at ~0,
midpoint test fails at ~0.009). Confirm at RED authoring: if native
still converts, swap to `hole_to_polyhole_threshold=0.0001` or mark the
leg as control-identical.

Witness style follows `region_speed_emission.rs`: `OrcaRunner`
`--slice 0 --outputdir`, `golden::compare_ordered_bytes_generator_only`
whole-output compare, control leg asserted PASS first, positive legs
asserted to FAIL at RED and pass post-GREEN; seeded + official modes.
