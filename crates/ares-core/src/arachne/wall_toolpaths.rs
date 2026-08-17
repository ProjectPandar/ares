mod stitch;

use crate::geometry::{CoordinateScale, Polygon};

use super::{
    beading::factory::{BeadingStrategyFactoryConfig, make_strategy},
    extrusion_line::ExtrusionLine,
    trapezoidation::{SkeletalTrapezoidation, TrapezoidationConfig, TrapezoidationError},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawWallToolPathConfig {
    pub(crate) outer_spacing: i64,
    pub(crate) inner_spacing: i64,
    pub(crate) inset_count: usize,
    pub(crate) outer_wall_inset: i64,
    pub(crate) layer_height: i64,
    pub(crate) min_bead_width: i64,
    pub(crate) min_feature_size: i64,
    pub(crate) transition_length: i64,
    pub(crate) transitioning_angle: f64,
    pub(crate) transition_filter_deviation: i64,
    pub(crate) wall_distribution_count: i32,
    pub(crate) coordinate_scale: CoordinateScale,
}

pub(crate) fn generate_raw(
    prepared_outline: &[Polygon],
    config: RawWallToolPathConfig,
) -> Result<Vec<Vec<ExtrusionLine>>, TrapezoidationError> {
    if config.inset_count == 0 {
        return Ok(Vec::new());
    }
    let rounded_rectangle_factor = 1.0 - 0.25 * std::f64::consts::PI;
    let outer_width =
        config.outer_spacing as f64 + config.layer_height as f64 * rounded_rectangle_factor;
    let inner_width =
        config.inner_spacing as f64 + config.layer_height as f64 * rounded_rectangle_factor;
    let strategy = make_strategy(BeadingStrategyFactoryConfig {
        preferred_bead_width_outer: config.outer_spacing,
        preferred_bead_width_inner: config.inner_spacing,
        preferred_transition_length: config.transition_length,
        transitioning_angle: config.transitioning_angle as f32,
        print_thin_walls: true,
        min_bead_width: config.min_bead_width,
        min_feature_size: config.min_feature_size,
        wall_split_middle_threshold: (2.0 * config.min_bead_width as f64 / outer_width - 1.0)
            .clamp(0.01, 0.99),
        wall_add_middle_threshold: (config.min_bead_width as f64 / inner_width).clamp(0.01, 0.99),
        max_bead_count: (config.inset_count * 2) as i64,
        outer_wall_offset: config.outer_wall_inset,
        inward_distributed_center_wall_count: config.wall_distribution_count,
        minimum_variable_line_ratio: 0.5,
        coordinate_scale: config.coordinate_scale,
    });
    let trapezoidation = SkeletalTrapezoidation::new(
        prepared_outline,
        strategy.as_ref(),
        TrapezoidationConfig {
            transitioning_angle: config.transitioning_angle,
            discretization_step_size: config.coordinate_scale.checked_scale(0.8).unwrap(),
            transition_filter_dist: config.coordinate_scale.checked_scale(100.0).unwrap(),
            allowed_filter_deviation: config.transition_filter_deviation,
            beading_propagation_transition_dist: config.transition_length,
            coordinate_scale: config.coordinate_scale,
        },
    )?;
    Ok(trapezoidation.generate_toolpaths(false))
}

pub(crate) fn generate(
    prepared_outline: &[Polygon],
    config: RawWallToolPathConfig,
) -> Result<Vec<Vec<ExtrusionLine>>, TrapezoidationError> {
    let mut toolpaths = generate_raw(prepared_outline, config)?;
    stitch::stitch_toolpaths(
        &mut toolpaths,
        config.inner_spacing - 1,
        config.coordinate_scale.checked_scale(0.01).unwrap(),
    );
    Ok(toolpaths)
}

#[cfg(test)]
mod tests {
    use crate::geometry::{CoordinateScale, Point, Polygon};

    use super::{RawWallToolPathConfig, generate, generate_raw};

    fn fixture() -> (Polygon, RawWallToolPathConfig) {
        let scale = CoordinateScale::Normal;
        let scaled = |value| scale.checked_scale(value).unwrap();
        (
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(scaled(10.0), 0),
                Point::new(scaled(10.0), scaled(10.0)),
                Point::new(0, scaled(10.0)),
            ]),
            RawWallToolPathConfig {
                outer_spacing: scaled(0.4),
                inner_spacing: scaled(0.4),
                inset_count: 26,
                outer_wall_inset: 0,
                layer_height: scaled(0.2),
                min_bead_width: scaled(0.34),
                min_feature_size: scaled(0.1),
                transition_length: scaled(0.4),
                transitioning_angle: 10.0_f64.to_radians(),
                transition_filter_deviation: scaled(0.1),
                wall_distribution_count: 1,
                coordinate_scale: scale,
            },
        )
    }

    #[test]
    fn task22o195_rectangle_generates_positive_width_raw_wall_toolpaths() {
        let (outline, config) = fixture();

        let toolpaths = generate_raw(&[outline], config).unwrap();

        assert!(toolpaths.iter().flatten().any(|line| {
            line.inset_index < config.inset_count
                && line.junctions.len() >= 2
                && line.junctions.iter().all(|junction| junction.width > 0)
        }));
    }

    #[test]
    fn task22o196_wall_toolpath_pipeline_stitches_closed_rectangle_lines() {
        let (outline, config) = fixture();

        let toolpaths = generate(&[outline], config).unwrap();

        assert!(toolpaths.iter().flatten().any(|line| line.is_closed));
    }
}
