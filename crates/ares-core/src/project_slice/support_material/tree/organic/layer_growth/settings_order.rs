//! First-valid candidate priority from `increase_areas_one_layer`.

use super::super::support_element::AreaIncreaseSettings;
use super::{
    AvoidanceType, Movement, SupportElement, SupportElementState, TreeModelVolumes, TreeSettings,
};
use crate::geometry::{ClipperError, intersection_polygons_paths};

pub(super) fn ordered(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    elements: (&SupportElementState, &SupportElement),
    movement: (&Movement, bool),
    layer: usize,
) -> Result<Vec<AreaIncreaseSettings>, ClipperError> {
    let (state, parent) = elements;
    let (movement, merge_layer) = movement;
    let slow = movement.slow_speed;
    let fast = movement.fast_speed;
    let mut order = Vec::new();
    let mut insert = |kind, speed, increase_radius, move_area| {
        let settings = AreaIncreaseSettings {
            kind,
            increase_speed: speed,
            increase_radius,
            no_error: true,
            use_min_distance: false,
            move_area,
        };
        if !order.contains(&settings) {
            order.push(settings);
        }
    };
    let last = state.last_area_increase;
    let moved_slow = last.increase_speed < config.maximum_move_distance;
    let mismatch = moved_slow && last.kind != AvoidanceType::Slow;
    if last.move_area
        && last.no_error
        && state.bits.can_use_safe_radius
        && !merge_layer
        && !mismatch
        && (state.distance_to_top as usize >= config.tip_layers || moved_slow)
    {
        let speed = if moved_slow { slow } else { fast };
        insert(last.kind, speed, true, last.move_area);
        insert(last.kind, speed, false, last.move_area);
    }
    if !state.bits.can_use_safe_radius {
        insert(AvoidanceType::Slow, slow, true, false);
        if (state.distance_to_top as usize) < config.tip_layers.div_ceil(2) {
            insert(AvoidanceType::Fast, slow, true, false);
        }
        insert(AvoidanceType::FastSafe, fast, true, false);
        insert(AvoidanceType::FastSafe, fast, false, true);
        insert(AvoidanceType::Fast, fast, false, true);
    } else {
        insert(AvoidanceType::Slow, slow, true, true);
        insert(AvoidanceType::Slow, slow, false, true);
        if (state.distance_to_top as usize) < config.tip_layers {
            insert(AvoidanceType::FastSafe, slow, true, true);
        }
        insert(AvoidanceType::FastSafe, fast, true, true);
        insert(AvoidanceType::FastSafe, fast, false, true);
    }
    if state.bits.use_min_xy_dist {
        order = order
            .into_iter()
            .flat_map(|settings| {
                [
                    settings,
                    AreaIncreaseSettings {
                        use_min_distance: true,
                        ..settings
                    },
                ]
            })
            .collect();
    }
    if state.bits.to_buildplate
        || (state.bits.to_model_gracious
            && intersection_polygons_paths(
                &parent.influence_area,
                volumes.placeable_areas(state.collision_radius(config), layer)?,
            )?
            .is_empty())
    {
        let settings = AreaIncreaseSettings {
            kind: AvoidanceType::Fast,
            increase_speed: fast,
            increase_radius: false,
            no_error: false,
            use_min_distance: state.bits.use_min_xy_dist,
            move_area: true,
        };
        if !order.contains(&settings) {
            order.push(settings);
        }
    }
    if state.distance_to_top < state.dont_move_until && state.bits.can_use_safe_radius {
        let settings = AreaIncreaseSettings {
            kind: AvoidanceType::Slow,
            increase_speed: 0,
            increase_radius: true,
            no_error: true,
            use_min_distance: false,
            move_area: false,
        };
        if !order.contains(&settings) {
            order.insert(0, settings);
        }
    }
    Ok(order)
}
