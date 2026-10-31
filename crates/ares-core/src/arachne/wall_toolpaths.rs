mod outline;
mod postprocess;
pub(crate) mod region_order;
mod stitch;

use crate::geometry::{ClipperError, CoordinateScale, Polygon};

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
    pub(crate) layer_height: f64,
    pub(crate) min_bead_width: i64,
    pub(crate) min_feature_size: i64,
    pub(crate) transition_length: i64,
    /// Radians, as consumed by the beading and trapezoidation constructors.
    pub(crate) transitioning_angle: f64,
    pub(crate) transition_filter_deviation: i64,
    pub(crate) wall_distribution_count: i32,
    pub(crate) min_length_factor: f64,
    pub(crate) wall_maximum_resolution: i64,
    pub(crate) wall_maximum_deviation: i64,
    pub(crate) is_top_or_bottom_layer: bool,
    pub(crate) coordinate_scale: CoordinateScale,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GeneratedWallToolPaths {
    pub(crate) toolpaths: Vec<Vec<ExtrusionLine>>,
    pub(crate) inner_contour: Vec<Polygon>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WallToolPathError {
    Geometry(ClipperError),
    Trapezoidation(TrapezoidationError),
}

impl From<ClipperError> for WallToolPathError {
    fn from(error: ClipperError) -> Self {
        Self::Geometry(error)
    }
}

impl From<TrapezoidationError> for WallToolPathError {
    fn from(error: TrapezoidationError) -> Self {
        Self::Trapezoidation(error)
    }
}

pub(crate) fn generate_raw(
    prepared_outline: &[Polygon],
    config: RawWallToolPathConfig,
) -> Result<Vec<Vec<ExtrusionLine>>, TrapezoidationError> {
    if config.inset_count == 0 {
        return Ok(Vec::new());
    }
    let rounded_rectangle_factor = 1.0 - 0.25 * std::f64::consts::PI;
    let scale = config.coordinate_scale.factor();
    let layer_height = config.layer_height as f32;
    let extrusion_width = |spacing| {
        let spacing = spacing as f32 * scale as f32;
        (f64::from(spacing) + f64::from(layer_height) * rounded_rectangle_factor) as f32
    };
    let outer_width = f64::from(extrusion_width(config.outer_spacing));
    let inner_width = f64::from(extrusion_width(config.inner_spacing));
    let min_bead_width = config.min_bead_width as f64 * scale;
    let strategy = make_strategy(BeadingStrategyFactoryConfig {
        preferred_bead_width_outer: config.outer_spacing,
        preferred_bead_width_inner: config.inner_spacing,
        preferred_transition_length: config.transition_length,
        transitioning_angle: config.transitioning_angle as f32,
        print_thin_walls: true,
        min_bead_width: config.min_bead_width,
        min_feature_size: config.min_feature_size,
        wall_split_middle_threshold: (2.0 * min_bead_width / outer_width - 1.0).clamp(0.01, 0.99),
        wall_add_middle_threshold: (min_bead_width / inner_width).clamp(0.01, 0.99),
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
            transitioning_angle: strategy.transitioning_angle(),
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
    outline: &[Polygon],
    config: RawWallToolPathConfig,
) -> Result<GeneratedWallToolPaths, WallToolPathError> {
    // `WallToolPaths.cpp:778-789::getInnerContour`: zero walls retain the outline.
    if config.inset_count == 0 {
        return Ok(GeneratedWallToolPaths {
            toolpaths: Vec::new(),
            inner_contour: outline.to_vec(),
        });
    }
    let prepared_outline = outline::prepare(outline, config)?;
    let mut toolpaths = generate_raw(&prepared_outline, config)?;
    stitch::stitch_toolpaths(
        &mut toolpaths,
        config.inner_spacing - 1,
        config.coordinate_scale.checked_scale(0.01).unwrap(),
    );
    postprocess::remove_small_lines(
        &mut toolpaths,
        config.min_length_factor,
        config.is_top_or_bottom_layer,
    );
    let inner_contour = postprocess::separate_inner_contour(&mut toolpaths);
    postprocess::simplify_toolpaths(&mut toolpaths, config);
    Ok(GeneratedWallToolPaths {
        toolpaths,
        inner_contour,
    })
}
