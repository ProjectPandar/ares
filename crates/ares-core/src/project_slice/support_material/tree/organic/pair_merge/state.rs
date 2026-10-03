//! `TreeSupport3D.cpp::merge_support_element_states`.

use super::super::{
    TreeSettings,
    support_element::{AreaIncreaseSettings, StateBits, SupportElementState},
};
use crate::geometry::Point;

pub(super) fn merge(
    config: &TreeSettings,
    first: &SupportElementState,
    second: &SupportElementState,
    next: (Point, usize),
) -> SupportElementState {
    let target = if first.target_height > second.target_height {
        first
    } else {
        second
    };
    let missing_roof_layers = first.missing_roof_layers.max(second.missing_roof_layers);
    let mut state = SupportElementState {
        bits: StateBits {
            use_min_xy_dist: first.bits.use_min_xy_dist || second.bits.use_min_xy_dist,
            supports_roof: first.bits.supports_roof || second.bits.supports_roof,
            can_use_safe_radius: first.bits.can_use_safe_radius || second.bits.can_use_safe_radius,
            to_buildplate: first.bits.to_buildplate && second.bits.to_buildplate,
            to_model_gracious: first.bits.to_model_gracious && second.bits.to_model_gracious,
            ..StateBits::default()
        },
        target_height: target.target_height,
        target_position: target.target_position,
        next_position: next.0,
        layer_index: next.1,
        effective_radius_height: first
            .effective_radius_height
            .max(second.effective_radius_height),
        distance_to_top: first.distance_to_top.max(second.distance_to_top),
        result_on_layer: None,
        // The pair consumer sets this before the merged node becomes observable.
        increased_to_model_radius: 0,
        elephant_foot_increases: 0.0,
        dont_move_until: first.dont_move_until.max(second.dont_move_until),
        last_area_increase: AreaIncreaseSettings {
            kind: first
                .last_area_increase
                .kind
                .min(second.last_area_increase.kind),
            increase_speed: first
                .last_area_increase
                .increase_speed
                .min(second.last_area_increase.increase_speed),
            increase_radius: first.last_area_increase.increase_radius
                || second.last_area_increase.increase_radius,
            no_error: first.last_area_increase.no_error || second.last_area_increase.no_error,
            use_min_distance: first.last_area_increase.use_min_distance
                && second.last_area_increase.use_min_distance,
            move_area: first.last_area_increase.move_area || second.last_area_increase.move_area,
        },
        missing_roof_layers,
        roof_recovery_depth: if missing_roof_layers > 0 {
            first.roof_recovery_depth.max(second.roof_recovery_depth)
        } else {
            0
        },
    };
    if config.buildplate_radius_increase_per_layer > 0.0 {
        let foot_radius = (second
            .collision_radius(config)
            .max(first.collision_radius(config))
            - state.collision_radius(config))
        .abs();
        state.elephant_foot_increases = foot_radius as f64
            / (config.buildplate_radius_increase_per_layer
                - config.branch_radius_increase_per_layer);
    }
    state
}
