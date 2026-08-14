use crate::geometry::CoordinateScale;

use super::{
    base::{BeadingStrategy, BeadingStrategyConfig},
    distributed::DistributedBeadingStrategy,
    limited::LimitedBeadingStrategy,
    outer_inset::OuterWallInsetBeadingStrategy,
    redistribute::RedistributeBeadingStrategy,
    widening::WideningBeadingStrategy,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct BeadingStrategyFactoryConfig {
    pub(crate) preferred_bead_width_outer: i64,
    pub(crate) preferred_bead_width_inner: i64,
    pub(crate) preferred_transition_length: i64,
    pub(crate) transitioning_angle: f32,
    pub(crate) print_thin_walls: bool,
    pub(crate) min_bead_width: i64,
    pub(crate) min_feature_size: i64,
    pub(crate) wall_split_middle_threshold: f64,
    pub(crate) wall_add_middle_threshold: f64,
    pub(crate) max_bead_count: i64,
    pub(crate) outer_wall_offset: i64,
    pub(crate) inward_distributed_center_wall_count: i32,
    pub(crate) minimum_variable_line_ratio: f64,
    pub(crate) coordinate_scale: CoordinateScale,
}

pub(crate) fn make_strategy(config: BeadingStrategyFactoryConfig) -> Box<dyn BeadingStrategy> {
    let optimal_width = if config.max_bead_count <= 2 {
        config.preferred_bead_width_outer
    } else {
        config.preferred_bead_width_inner
    };
    let base = BeadingStrategyConfig {
        optimal_width,
        wall_split_middle_threshold: config.wall_split_middle_threshold,
        wall_add_middle_threshold: config.wall_add_middle_threshold,
        default_transition_length: config.preferred_transition_length,
        transitioning_angle: config.transitioning_angle as f64,
        coordinate_scale: config.coordinate_scale,
    };
    let mut strategy: Box<dyn BeadingStrategy> = Box::new(DistributedBeadingStrategy::new(
        base,
        config.inward_distributed_center_wall_count,
    ));
    strategy = Box::new(RedistributeBeadingStrategy::new(
        config.preferred_bead_width_outer,
        config.minimum_variable_line_ratio,
        strategy,
    ));
    if config.print_thin_walls {
        strategy = Box::new(WideningBeadingStrategy::new(
            strategy,
            config.min_feature_size,
            config.min_bead_width,
        ));
    }
    if config.outer_wall_offset != 0 {
        strategy = Box::new(OuterWallInsetBeadingStrategy::new(
            config.outer_wall_offset,
            strategy,
        ));
    }
    Box::new(LimitedBeadingStrategy::new(config.max_bead_count, strategy))
}
