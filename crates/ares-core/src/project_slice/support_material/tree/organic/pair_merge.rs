//! `TreeSupport3D.cpp::merge_influence_areas_two_elements`.

mod state;

use super::super::model_volumes::TreeModelVolumes;
use super::{
    TreeSettings,
    area_growth::{Expansion, safe_offset_inc, safe_union},
    layer_growth::SupportElementMerging,
    move_inside::move_inside_if_outside,
};
use crate::geometry::{
    BoundingBox, ClipperError, Coord, CoordinateScale, JoinType, Polygon,
    intersection_polygons_paths, offset_paths,
};

pub(super) fn merge_two(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    layer: usize,
    nodes: (&mut SupportElementMerging, &mut SupportElementMerging),
    scale: CoordinateScale,
) -> Result<bool, ClipperError> {
    let (dst, src) = nodes;
    if dst.state.bits.to_model_gracious != src.state.bits.to_model_gracious
        || dst.state.bits.use_min_xy_dist != src.state.bits.use_min_xy_dist
    {
        return Ok(false);
    }
    let dst_bigger = dst.state.collision_radius(config) > src.state.collision_radius(config);
    let (small, big) = if dst_bigger {
        (&*src, &*dst)
    } else {
        (&*dst, &*src)
    };
    let radius_delta = (big.state.actual_radius(config) - small.state.actual_radius(config)).abs();
    let small_bbox = small
        .bbox
        .expect("grown merge node has a bbox")
        .inflated(radius_delta);
    let big_bbox = big.bbox.expect("grown merge node has a bbox");
    if small_bbox.max().x() < big_bbox.min().x()
        || big_bbox.max().x() < small_bbox.min().x()
        || small_bbox.max().y() < big_bbox.min().y()
        || big_bbox.max().y() < small_bbox.min().y()
    {
        return Ok(false);
    }
    let mut increased_to_model_radius = 0;
    let to_bp = dst.state.bits.to_buildplate && src.state.bits.to_buildplate;
    if !to_bp {
        if dst.state.bits.to_buildplate != src.state.bits.to_buildplate {
            let rdst = dst.state.actual_radius(config);
            let rsrc = src.state.actual_radius(config);
            if dst.state.bits.to_buildplate && rsrc < rdst {
                increased_to_model_radius = src.state.increased_to_model_radius + rdst - rsrc;
            } else if !dst.state.bits.to_buildplate && rsrc > rdst {
                increased_to_model_radius = dst.state.increased_to_model_radius + rsrc - rdst;
            }
            if increased_to_model_radius > config.max_to_model_radius_increase {
                return Ok(false);
            }
        }
        if !dst.state.bits.supports_roof
            && !src.state.bits.supports_roof
            && (src.state.distance_to_top.max(dst.state.distance_to_top) as usize)
                < config.min_dtt_to_model
        {
            return Ok(false);
        }
    }
    if !big.state.bits.can_use_safe_radius && small.state.bits.can_use_safe_radius {
        return Ok(false);
    }
    let small_radius = small.state.collision_radius(config);
    let collision = volumes.collision(
        small_radius,
        layer - 1,
        big.state.bits.use_min_xy_dist && small.state.bits.use_min_xy_dist,
    )?;
    let intersection = |small: &[Polygon], big: &[Polygon]| {
        intersection_polygons_paths(
            &safe_offset_inc(
                small,
                collision,
                Expansion {
                    distance: radius_delta,
                    safe_step: 2 * (config.xy_distance + small_radius - 3),
                    unchecked_last: 0,
                    minimum_offsets: 0,
                },
                scale,
            )?,
            big,
        )
    };
    let overlap = intersection(
        if to_bp {
            &small.areas.to_bp_areas
        } else {
            &small.areas.to_model_areas
        },
        if to_bp {
            &big.areas.to_bp_areas
        } else {
            &big.areas.to_model_areas
        },
    )?;
    let tiny = scale.scaled_delta(0.001).powi(2);
    if area(&overlap) <= tiny
        || area(&offset_paths(
            &overlap,
            scale.scaled_delta(-0.025) as f32,
            JoinType::Miter,
            1.2,
        )?) <= tiny
    {
        return Ok(false);
    }
    let next = move_inside_if_outside(&overlap, dst.state.next_position);
    let mut merged = state::merge(config, &dst.state, &src.state, (next, layer - 1));
    merged.increased_to_model_radius = if increased_to_model_radius == 0 {
        dst.state
            .increased_to_model_radius
            .max(src.state.increased_to_model_radius)
    } else {
        increased_to_model_radius
    };
    let influence = safe_union(
        &intersection(&small.areas.influence_areas, &big.areas.influence_areas)?,
        &overlap,
        scale,
    )?;
    let model = if to_bp && config.support_rests_on_model {
        if merged.bits.to_model_gracious {
            safe_union(
                &intersection(&small.areas.to_model_areas, &big.areas.to_model_areas)?,
                &overlap,
                scale,
            )?
        } else {
            influence.clone()
        }
    } else {
        Vec::new()
    };
    dst.parents.extend_from_slice(&src.parents);
    dst.state = merged;
    dst.areas.influence_areas = influence;
    dst.areas.to_bp_areas.clear();
    dst.areas.to_model_areas.clear();
    if to_bp {
        dst.areas.to_bp_areas = overlap;
        if config.support_rests_on_model {
            dst.areas.to_model_areas = model;
        }
    } else {
        dst.areas.to_model_areas = overlap;
    }
    let mut bbox = BoundingBox::from_polygons(&dst.areas.influence_areas)
        .expect("successful native merge has a nonempty influence area");
    for area in [&dst.areas.to_bp_areas, &dst.areas.to_model_areas] {
        if let Some(other) = BoundingBox::from_polygons(area) {
            bbox.merge_point(other.min());
            bbox.merge_point(other.max());
        }
    }
    dst.bbox = Some(bbox.inflated(scale.scaled_delta(1e-4) as Coord));
    src.areas.influence_areas.clear();
    src.areas.to_bp_areas.clear();
    src.areas.to_model_areas.clear();
    src.parents.clear();
    Ok(true)
}

fn area(polygons: &[Polygon]) -> f64 {
    polygons.iter().map(Polygon::area).sum()
}
