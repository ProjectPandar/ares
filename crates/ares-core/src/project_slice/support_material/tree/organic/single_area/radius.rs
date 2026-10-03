//! Radius catch-up and buildplate foot growth in `increase_single_area`.

use super::{
    IncreaseRequest, SupportElementInfluenceAreas, SupportElementState, TreeModelVolumes,
    TreeSettings, area, diff_clipped, safe_union,
};
use crate::geometry::{ClipperError, Coord, CoordinateScale, Polygon};

pub(super) fn grow(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    request: &IncreaseRequest<'_>,
    output: (&mut SupportElementState, &mut SupportElementInfluenceAreas),
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let (current, areas) = output;
    let minimum_xy = request.settings.use_min_distance;
    let mut radius = current.collision_radius(config);
    let ceil_before = volumes.ceil_radius(radius, minimum_xy);
    if radius < config.increase_radius_until_radius && radius < current.actual_radius(config) {
        let target = current
            .actual_radius(config)
            .min(config.increase_radius_until_radius);
        let mut ceil = volumes.next_ceil_radius(radius, minimum_xy);
        while ceil < target {
            let next = volumes.next_ceil_radius(ceil + 1, minimum_xy);
            if !valid(
                volumes,
                request,
                (config, current, radius),
                (next, &areas.influence_areas),
                scale,
            )? {
                break;
            }
            ceil = next;
        }
        let mut height = current.effective_radius_height as usize;
        while height + 1 < current.distance_to_top as usize
            && config.radius(height + 1, current.elephant_foot_increases) <= ceil
            && config.radius(height + 1, current.elephant_foot_increases)
                <= current.actual_radius(config)
        {
            height += 1;
        }
        current.effective_radius_height = height as u32;
    }
    radius = current.collision_radius(config);
    let foot_radius_increase = (config.buildplate_radius_increase_per_layer
        - config.branch_radius_increase_per_layer)
        .max(0.0) as Coord;
    let planned = 1.0_f64.min(
        (config.recommended_min_radius((request.layer - 1) as i32) - current.actual_radius(config))
            as f64
            / foot_radius_increase as f64,
    );
    if planned > 0.0
        && current.bits.to_buildplate
        && current.actual_radius(config) >= config.branch_radius
        && current.actual_radius(config) >= config.increase_radius_until_radius
        && valid(
            volumes,
            request,
            (config, current, radius),
            (
                config.radius(
                    current.effective_radius_height as usize,
                    current.elephant_foot_increases + planned,
                ),
                &areas.influence_areas,
            ),
            scale,
        )?
    {
        current.elephant_foot_increases += planned;
        radius = current.collision_radius(config);
    }
    if ceil_before != volumes.ceil_radius(radius, minimum_xy) {
        if current.bits.to_buildplate {
            let difference = diff_clipped(
                &areas.influence_areas,
                volumes.avoidance(
                    (radius, request.layer - 1),
                    request.settings.kind,
                    false,
                    minimum_xy,
                )?,
                scale,
            )?;
            areas.to_bp_areas = safe_union(&difference, &[], scale)?;
        }
        if config.support_rests_on_model && (!current.bits.to_buildplate || request.merge_layer) {
            let collision = if current.bits.to_model_gracious {
                volumes.avoidance(
                    (radius, request.layer - 1),
                    request.settings.kind,
                    true,
                    minimum_xy,
                )?
            } else {
                volumes.collision(radius, request.layer - 1, minimum_xy)?
            };
            areas.to_model_areas = safe_union(
                &diff_clipped(&areas.influence_areas, collision, scale)?,
                &[],
                scale,
            )?;
        }
    }
    Ok(())
}

fn valid(
    volumes: &mut TreeModelVolumes<'_>,
    request: &IncreaseRequest<'_>,
    current: (&TreeSettings, &SupportElementState, Coord),
    next: (Coord, &[Polygon]),
    scale: CoordinateScale,
) -> Result<bool, ClipperError> {
    let (config, state, radius) = current;
    let (next_radius, increased) = next;
    let settings = request.settings;
    if volumes.ceil_radius(next_radius, settings.use_min_distance)
        <= volumes.ceil_radius(radius, settings.use_min_distance)
    {
        return Ok(true);
    }
    if !state.bits.to_buildplate && !config.support_rests_on_model {
        return Ok(false);
    }
    let collision = if state.bits.to_buildplate || state.bits.to_model_gracious {
        volumes.avoidance(
            (next_radius, request.layer - 1),
            settings.kind,
            !state.bits.to_buildplate,
            settings.use_min_distance,
        )?
    } else {
        volumes.collision(next_radius, request.layer - 1, settings.use_min_distance)?
    };
    Ok(area(&diff_clipped(increased, collision, scale)?) > scale.scaled_delta(0.001).powi(2))
}
