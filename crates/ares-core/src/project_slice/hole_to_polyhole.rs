//! `PrintObject::_transform_hole_to_polyholes`
//! (`PrintObject.cpp:189-295`, SuperSlicer port): detect almost-circular
//! holes in the post-compensation region slices and replace them with
//! `create_polyholes` geometry (:160-186). Runs at the
//! `PrintObjectSlice.cpp:847` call order — after `slice_volumes` (Ares:
//! post `prepare_post_compensation`, which applies the upstream
//! `slice_volumes` XY compensation + elephant-foot steps) and before
//! brim grouping consumes the layer outline.
//!
//! Detection gate is per `LayerRegion` (`region().config()` at
//! `PrintObject.cpp:201`): `hole_to_polyhole`, `hole_to_polyhole_threshold`
//! (`FloatOrPercent`, `get_abs_value(mean radius)`), and
//! `hole_to_polyhole_twisted`. The polyhole builder is the free function
//! `create_polyholes(center, radius, nozzle_diameter, multiple)` where
//! the nozzle is `print()->config().nozzle_diameter.get_at(
//! outer_wall_filament_id - 1)` — indexed by filament id (:282).
//!
//! Grouping mirrors `std::map<std::tuple<Point,float,int,coord_t,bool>,
//! ...>` exactly (`PrintObject.cpp:246-280`): keys iterate in
//! lexicographical tuple order (Point `<` = x then y,
//! `Point.hpp:172-175`; `float`/`int`/`coord_t`/`bool` natural order);
//! `emplace` does not overwrite, so a second hole producing an
//! identical key is dropped. The upward scan breaks after the first
//! matching hole per layer and erases it from that layer's candidates.
//!
//! `convex_points(PI)` is the detection convexity predicate
//! (`Polygon.cpp:310-315` via `filter_convex_concave_points_by_angle_
//! threshold`, `filter_points_by_vectors` :235-258): each vertex is a
//! convex point iff `cross2(v1, v2) > 0` **and** `dot(normalized v1,
//! normalized v2) < cos(nextafter(PI, +inf))` — the second clause is
//! unreachable (normalized dot >= -1), so the predicate is exactly
//! "no vertex with positive cross2".

use std::collections::BTreeMap;

use super::{compensation::PostCompensationPrintObject, region_slices::PostRegionPrintObject};
use crate::{
    OrcaFloats,
    geometry::{CoordinateScale, Point, Polygon},
};

mod detect;
mod polyhole;
use detect::{convex_points_pi, detect, distance_to};
use polyhole::create_polyholes;

/// Upstream `coord_t` (`i64` scaled units).
type CoordT = i64;

/// `std::tuple<Point,float,int,coord_t,bool>` key — lexicographical
/// ordering identical to `std::tuple` element-wise `<` (`Point` first:
/// x, then y). Floats compare by `partial_cmp`; NaN is impossible for
/// in-bounds geometry, so `unwrap_or(Equal)` mirrors the
/// unspecified-but-total ordering of equal-ordinate ties.
#[derive(Clone, Copy)]
struct HoleId {
    center: Point,
    diameter_max: f32,
    extruder_id: i32,
    max_variation: CoordT,
    twist: bool,
}

impl PartialEq for HoleId {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}
impl Eq for HoleId {}
impl PartialOrd for HoleId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HoleId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.center
            .cmp(&other.center)
            .then_with(|| {
                self.diameter_max
                    .partial_cmp(&other.diameter_max)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| self.extruder_id.cmp(&other.extruder_id))
            .then_with(|| self.max_variation.cmp(&other.max_variation))
            .then_with(|| self.twist.cmp(&other.twist))
    }
}

/// `(region, surface, hole)` indices into
/// `regions[region].layers[layer].surfaces[surface].expolygon.holes[hole]`
/// — the `Polygon*` the upstream tuple carries.
#[derive(Clone, Copy)]
struct HoleRef {
    region: usize,
    surface: usize,
    hole: usize,
}

/// `PrintObject.cpp:189-295` — `_transform_hole_to_polyholes` applied
/// to one prepared post-compensation object. Mutates both sites the
/// upstream function writes: `object.regions[*].layers[*].surfaces`
/// expolygon holes and `lslices` (matched by point equality, :285-292).
pub(super) fn transform(
    object: &mut PostCompensationPrintObject,
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) {
    let (post_regions, lslices) = object.polyhole_parts_mut();
    transform_object(post_regions, lslices, nozzle_diameters, scale);
}

fn transform_object(
    object: &mut PostRegionPrintObject,
    lslices: &mut [Vec<crate::geometry::ExPolygon>],
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) {
    let layer_count = object.plan.layers.len();
    // PrintObject.cpp:195-243 — per-layer candidate detection.
    let mut layerid2center: Vec<Vec<(HoleId, HoleRef)>> =
        (0..layer_count).map(|_| Vec::new()).collect();
    for (layer_idx, candidates) in layerid2center.iter_mut().enumerate() {
        detect_layer_candidates(object, layer_idx, candidates, scale);
    }

    // PrintObject.cpp:246-280 — group holes spanning >= 2 contiguous
    // layers (or a single layer-0 hole); key = the upstream tuple.
    let mut id2layerz2hole: BTreeMap<HoleId, Vec<(HoleRef, usize)>> = BTreeMap::new();
    let layers: Vec<(f64, f64)> = object
        .plan
        .layers
        .iter()
        .map(|layer| (layer.print_z, layer.height))
        .collect();
    for layer_idx in 0..layer_count {
        for hole_idx in 0..layerid2center[layer_idx].len() {
            let (id, first) = layerid2center[layer_idx][hole_idx];
            let holes = group_span(&layers, &mut layerid2center, layer_idx, first, id);
            // :276-279 — min_nb_layers = 2, layer-0 exception.
            if holes.len() >= 2 || (holes.len() == 1 && holes[0].1 == 0) {
                // `emplace` drops a later hole whose key already exists.
                id2layerz2hole.entry(id).or_insert(holes);
            }
        }
    }

    // PrintObject.cpp:281-294 — one polyhole set per id; replace hole
    // points in lslices (point-equality match) and the region surface.
    for (id, group) in id2layerz2hole {
        // :282 — nozzle indexed by `outer_wall_filament_id - 1`
        // (filament id; `get_at` = `i < len ? [i] : front`).
        let nozzle = nozzle_at(nozzle_diameters, id.extruder_id);
        let polyholes = create_polyholes(
            id.center,
            // `diameter_max` is a `Point` distance — already in coord
            // units; upstream passes it verbatim as the `radius` arg
            // (:282, `coord_t` tuple → `coord_t` parameter).
            id.diameter_max as i64,
            // :282 — `scale_(print()->config().nozzle_diameter.get_at(
            // outer_wall_filament_id - 1))`.
            scale
                .checked_scale(nozzle)
                .expect("nozzle diameter must fit the coordinate scale"),
            id.twist,
            scale,
        );
        for (hole_ref, layer_idx) in group {
            let polyhole = polyholes[layer_idx % polyholes.len()].clone();
            replace_hole(object, lslices, hole_ref, layer_idx, polyhole);
        }
    }
}

/// `PrintObject.cpp:197-243` for one layer: per-region gate +
/// per-surface hole scan, pushing qualifying candidates.
fn detect_layer_candidates(
    object: &PostRegionPrintObject,
    layer_idx: usize,
    candidates: &mut Vec<(HoleId, HoleRef)>,
    scale: CoordinateScale,
) {
    for (region_idx, region) in object.regions.iter().enumerate() {
        if !region.options.hole_to_polyhole.0 {
            continue;
        }
        for (surface_idx, surface) in region.layers[layer_idx].surfaces.iter().enumerate() {
            let site = HoleRef {
                region: region_idx,
                surface: surface_idx,
                hole: 0,
            };
            detect_surface_candidates(surface, &region.options, scale, site, candidates);
        }
    }
}

/// Per-hole scan inside one surface (`PrintObject.cpp:206-243`).
fn detect_surface_candidates(
    surface: &super::region_slices::RegionSurface,
    options: &crate::RegionOptions,
    scale: CoordinateScale,
    site: HoleRef,
    candidates: &mut Vec<(HoleId, HoleRef)>,
) {
    for (hole_idx, hole) in surface.as_parts().1.holes().iter().enumerate() {
        // :208 — convex_points(PI) empty && > 8 points.
        if !convex_points_pi(hole).is_empty() || hole.points().len() <= 8 {
            continue;
        }
        let Some(id) = detect(hole, options, scale) else {
            continue;
        };
        candidates.push((
            id,
            HoleRef {
                hole: hole_idx,
                ..site
            },
        ));
    }
}

/// `PrintObject.cpp:250-280`: upward contiguous-layer scan that
/// collects same-id holes (`first match per layer`, erase on match)
/// and returns the candidate group for `id` seeded by `first`.
/// `layers` is `(print_z, height)` per planned layer.
fn group_span(
    layers: &[(f64, f64)],
    layerid2center: &mut [Vec<(HoleId, HoleRef)>],
    layer_idx: usize,
    first: HoleRef,
    id: HoleId,
) -> Vec<(HoleRef, usize)> {
    // `float max_z` upstream (`PrintObject.cpp:252`) — f32 truncation.
    let mut max_z = layers[layer_idx].0 as f32;
    let mut holes = vec![(first, layer_idx)];
    for (search_layer_idx, candidates) in layerid2center
        .iter_mut()
        .enumerate()
        .skip(layer_idx + 1)
        .take(layers.len())
    {
        let (print_z, height) = layers[search_layer_idx];
        // :258 — contiguous-layer adjacency (`print_z`/`height` are
        // double; `max_z` is upstream `float`).
        if print_z - height - f64::from(max_z) > EPSILON {
            break;
        }
        // :260-271 — first matching hole per layer, then erase.
        let mut search_hole_idx = 0;
        while search_hole_idx < candidates.len() {
            let search_id = candidates[search_hole_idx].0;
            let same_hole = search_id.extruder_id == id.extruder_id
                && distance_to(search_id.center, id.center) < id.max_variation as f64
                && f64::from((search_id.diameter_max - id.diameter_max).abs())
                    < id.max_variation as f64;
            if same_hole {
                max_z = layers[search_layer_idx].0 as f32;
                let removed = candidates.remove(search_hole_idx);
                holes.push((removed.1, search_layer_idx));
                // upstream `search_hole_idx--; break;` leaves the layer
                // after the first match; `remove` shifted the tail, so
                // re-checking index `search_hole_idx` mirrors the
                // decrement, and we break on the same layer.
                break;
            }
            search_hole_idx += 1;
        }
    }
    holes
}

/// `PrintObject.cpp:285-294`: replace the hole in the layer `lslices`
/// clones (point-equality match, :288 `poly_slice.points ==
/// poly_to_replace.first->points`) and in the region surface.
fn replace_hole(
    object: &mut PostRegionPrintObject,
    lslices: &mut [Vec<crate::geometry::ExPolygon>],
    hole_ref: HoleRef,
    layer_idx: usize,
    polyhole: Polygon,
) {
    let original = object.regions[hole_ref.region].layers[layer_idx].surfaces[hole_ref.surface]
        .as_parts()
        .1
        .holes()[hole_ref.hole]
        .points()
        .to_vec();
    for explo_slice in &mut lslices[layer_idx] {
        for poly_slice in explo_slice.holes_mut() {
            if poly_slice.points() == original.as_slice() {
                *poly_slice = polyhole.clone();
            }
        }
    }
    object.regions[hole_ref.region].layers[layer_idx].surfaces[hole_ref.surface]
        .expolygon_mut()
        .holes_mut()[hole_ref.hole] = polyhole;
}

/// `ConfigOption::get_at` (`Config.hpp`) — `i < len ? [i] : front`,
/// indexed by `outer_wall_filament_id - 1` (`PrintObject.cpp:282`).
fn nozzle_at(nozzle_diameters: &OrcaFloats, filament_id: i32) -> f64 {
    let index = usize::try_from(filament_id - 1).unwrap_or(usize::MAX);
    nozzle_diameters
        .0
        .get(index)
        .or_else(|| nozzle_diameters.0.first())
        .map(|value| value.0)
        .expect("nozzle_diameter list is non-empty (validated at config load)")
}

/// `libslic3r.h:52,96` — `EPSILON` in mm (adjacency) / `SCALED_EPSILON`.
const EPSILON: f64 = 1e-4;
