use super::PerimeterPath;
use super::options::{PerimeterOptions, WallSequence};
use crate::SliceError;

pub(super) fn loop_shrink(loop_index: u32, options: PerimeterOptions) -> Result<f64, SliceError> {
    let first = first_internal_shrink(options)?;
    if loop_index <= 1 {
        return Ok(first);
    }
    Ok(first + f64::from(loop_index - 1) * internal_line_spacing(options)?)
}

fn first_internal_shrink(options: PerimeterOptions) -> Result<f64, SliceError> {
    if options.precise_outer_wall() && options.wall_sequence() == WallSequence::InnerOuter {
        Ok((options.external_line_width() + options.internal_line_width()) / 2.0)
    } else {
        Ok((external_line_spacing(options)? + internal_line_spacing(options)?) / 2.0)
    }
}

fn external_line_spacing(options: PerimeterOptions) -> Result<f64, SliceError> {
    rounded_rectangle_spacing(options.external_line_width(), options.layer_height_mm())
}

fn internal_line_spacing(options: PerimeterOptions) -> Result<f64, SliceError> {
    rounded_rectangle_spacing(options.internal_line_width(), options.layer_height_mm())
}

fn rounded_rectangle_spacing(width: f64, layer_height: f64) -> Result<f64, SliceError> {
    let spacing = width - layer_height * (1.0 - std::f64::consts::PI / 4.0);
    if spacing.is_finite() && spacing > 0.0 {
        Ok(spacing)
    } else {
        Err(SliceError::InvalidInput(
            "perimeter spacing must be positive".to_owned(),
        ))
    }
}

pub(super) fn resolve_wall_loops(
    options: PerimeterOptions,
    layer_id: usize,
    topmost_layer_id: Option<usize>,
) -> u32 {
    if options.wall_loops() == 0 {
        return 0;
    }
    if layer_id == 0 && options.only_one_wall_first_layer() {
        return 1;
    }
    let mut wall_loops = options.wall_loops();
    if options.alternate_extra_wall()
        && layer_id % 2 == 1
        && options.sparse_infill_density_percent() > 0.0
    {
        wall_loops += 1;
    }
    if topmost_layer_id == Some(layer_id) && options.only_one_wall_top() && wall_loops > 1 {
        return 1;
    }
    wall_loops
}

pub(super) fn order_wall_sequence(
    mut paths: Vec<PerimeterPath>,
    sequence: WallSequence,
    layer_id: usize,
) -> Vec<PerimeterPath> {
    if paths.len() <= 1 {
        return paths;
    }
    match sequence {
        WallSequence::OuterInner => paths,
        WallSequence::InnerOuter => {
            paths[1..].reverse();
            paths.rotate_left(1);
            paths
        }
        WallSequence::InnerOuterInner if layer_id == 0 => {
            paths[1..].reverse();
            paths.rotate_left(1);
            paths
        }
        WallSequence::InnerOuterInner if paths.len() >= 3 => {
            let external = paths.remove(0);
            let first_internal = paths.remove(0);
            paths.reverse();
            paths.push(external);
            paths.push(first_internal);
            paths
        }
        WallSequence::InnerOuterInner => paths,
    }
}
