use crate::{PerimeterOptions, PerimeterPath, Point2, SliceError, WallGenerator};

use super::rectangles;

pub(super) fn append_rectangular_thin_wall(
    paths: &mut Vec<PerimeterPath>,
    points: &[Point2],
    options: PerimeterOptions,
    config: RectangularThinWallConfig,
) -> Result<(), SliceError> {
    if !options.detect_thin_wall() || config.effective_wall_loops == 0 {
        return Ok(());
    }
    let Some(thin_wall) = rectangular_centerline(points, options, config.effective_wall_loops)
    else {
        return Ok(());
    };
    if shorter_than(
        &thin_wall.centerline,
        min_open_wall_length(options, config.uses_surface_length_threshold),
    ) {
        return Ok(());
    }
    let min_bead_width_mm = options.min_bead_width_mm_for_layer(config.layer_id);
    if options.wall_generator() == WallGenerator::Arachne
        && (thin_wall.thickness_mm < options.min_feature_size_mm()
            || exceeds_transition_filter(thin_wall.thickness_mm, min_bead_width_mm, options))
    {
        return Ok(());
    }
    let effective_line_width_mm = (options.wall_generator() == WallGenerator::Arachne)
        .then(|| thin_wall.thickness_mm.max(min_bead_width_mm));
    paths.push(
        PerimeterPath::open_external_thin_wall(
            options.wall_direction().orient_points(thin_wall.centerline),
        )?
        .with_effective_line_width_mm(effective_line_width_mm),
    );
    Ok(())
}

pub(super) struct RectangularThinWallConfig {
    pub(super) effective_wall_loops: u32,
    pub(super) layer_id: usize,
    pub(super) uses_surface_length_threshold: bool,
}

fn min_open_wall_length(options: PerimeterOptions, uses_surface_length_threshold: bool) -> f64 {
    if uses_surface_length_threshold {
        options.external_line_width() / 2.0
    } else {
        options.external_line_width() * options.min_length_factor()
    }
}

fn exceeds_transition_filter(
    thickness_mm: f64,
    min_bead_width_mm: f64,
    options: PerimeterOptions,
) -> bool {
    let line_width_deviation_mm = (thickness_mm - 2.0 * min_bead_width_mm).max(0.0);
    let distributed_deviation_mm =
        line_width_deviation_mm / f64::from(options.wall_distribution_count());
    distributed_deviation_mm > options.wall_transition_filter_deviation_mm()
}

fn shorter_than(points: &[Point2], check_length: f64) -> bool {
    let mut length = 0.0;
    for pair in points.windows(2) {
        length += distance(pair[0], pair[1]);
        if length >= check_length {
            return false;
        }
    }
    true
}

fn distance(start: Point2, end: Point2) -> f64 {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    (dx * dx + dy * dy).sqrt()
}

fn rectangular_centerline(
    points: &[Point2],
    options: PerimeterOptions,
    effective_wall_loops: u32,
) -> Option<RectangularThinWall> {
    let (min_x, min_y, max_x, max_y) = rectangles::bounds(points)?;
    let first_internal_shrink =
        (options.external_line_width() + options.internal_line_width()) / 2.0;
    let mut last_generated_offset = 0.0;
    for loop_index in 1..effective_wall_loops {
        let shrink =
            first_internal_shrink + f64::from(loop_index - 1) * options.internal_line_width();
        if min_x + shrink < max_x - shrink && min_y + shrink < max_y - shrink {
            last_generated_offset = shrink;
        }
    }

    let next_loop_offset = if last_generated_offset == 0.0 {
        first_internal_shrink
    } else {
        last_generated_offset + options.internal_line_width()
    };
    let next_width = max_x - min_x - 2.0 * next_loop_offset;
    let next_height = max_y - min_y - 2.0 * next_loop_offset;

    match (next_width > 0.0, next_height > 0.0) {
        (true, false) => {
            let center_y = (min_y + max_y) / 2.0;
            Some(RectangularThinWall {
                centerline: vec![
                    Point2::new(min_x + next_loop_offset, center_y),
                    Point2::new(max_x - next_loop_offset, center_y),
                ],
                thickness_mm: max_y - min_y,
            })
        }
        (false, true) => {
            let center_x = (min_x + max_x) / 2.0;
            Some(RectangularThinWall {
                centerline: vec![
                    Point2::new(center_x, min_y + next_loop_offset),
                    Point2::new(center_x, max_y - next_loop_offset),
                ],
                thickness_mm: max_x - min_x,
            })
        }
        (true, true) | (false, false) => None,
    }
}

struct RectangularThinWall {
    centerline: Vec<Point2>,
    thickness_mm: f64,
}
