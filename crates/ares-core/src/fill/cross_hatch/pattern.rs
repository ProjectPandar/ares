use super::transform::{checked_point, translate_polylines};
use crate::geometry::{ClipperError, Point, Polyline};

pub(super) fn repeat_ratio(density: f32) -> f64 {
    if f64::from(density) < 0.3 {
        (1.0 - f64::from((-5.0_f32 * density).exp())).clamp(0.2, 1.0)
    } else {
        1.0
    }
}

pub(super) fn generate_infill_layers(
    mut z_height: f64,
    repeat_ratio: f64,
    grid_size: i64,
    width: f64,
    height: f64,
) -> Result<Vec<Polyline>, ClipperError> {
    let grid_size = grid_size as f64;
    let trans_layer_size = grid_size * 0.4;
    let repeat_layer_size = grid_size * repeat_ratio;
    z_height += repeat_layer_size / 2.0 + trans_layer_size;
    let period = trans_layer_size + repeat_layer_size;
    let remains = z_height - (z_height / period).floor() * period;
    let trans_z = remains - repeat_layer_size;
    let phase = (z_height % (period * 2.0) - (period - 1.0)) as i32;
    let direction = if phase <= 0 { -1 } else { 1 };

    if trans_z < 0.0 {
        generate_repeat_pattern(direction, grid_size, width, height)
    } else {
        let progress = (trans_z % trans_layer_size) / trans_layer_size;
        if progress < 0.5 {
            generate_transform_pattern((progress + 0.1) * 2.0, direction, grid_size, width, height)
        } else {
            generate_transform_pattern((1.1 - progress) * 2.0, -direction, grid_size, width, height)
        }
    }
}

fn generate_transform_pattern(
    progress: f64,
    direction: i32,
    grid_size: f64,
    inwidth: f64,
    inheight: f64,
) -> Result<Vec<Polyline>, ClipperError> {
    let transform_grid = grid_size * 2.0;
    let offset = progress * (1.0 / 8.0) * transform_grid;
    let one_cycle = Polyline::new(vec![
        checked_point(0.25 * transform_grid - offset, offset)?,
        checked_point(0.25 * transform_grid + offset, offset)?,
        checked_point(0.75 * transform_grid - offset, -offset)?,
        checked_point(0.75 * transform_grid + offset, -offset)?,
    ]);
    let (width, height) = if direction < 0 {
        (inheight, inwidth)
    } else {
        (inwidth, inheight)
    };

    let number_of_cycles = (width / transform_grid + 2.0) as i32;
    let mut base_points = Vec::with_capacity(number_of_cycles as usize * one_cycle.points().len());
    for index in 0..number_of_cycles {
        let mut cycle = one_cycle.clone();
        let delta = checked_point(f64::from(index) * transform_grid, 0.0)?;
        translate_polylines(std::slice::from_mut(&mut cycle), delta)?;
        base_points.extend(cycle.into_points());
    }
    let base_row = Polyline::new(base_points);

    let number_of_rows = (height / transform_grid + 2.0) as i32;
    let mut polylines = Vec::with_capacity(number_of_rows as usize * 2);
    for index in 0..number_of_rows {
        let mut row = base_row.clone();
        let delta = checked_point(0.0, f64::from(index) * transform_grid)?;
        translate_polylines(std::slice::from_mut(&mut row), delta)?;
        polylines.push(row);
    }
    for index in 0..number_of_rows {
        let mut row = base_row.clone();
        let index = f64::from(index);
        let delta = checked_point(-0.5 * transform_grid, (index + 0.5) * transform_grid)?;
        translate_polylines(std::slice::from_mut(&mut row), delta)?;
        polylines.push(row);
    }

    if direction < 0 {
        for polyline in &mut polylines {
            let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
            *polyline = Polyline::new(
                points
                    .into_iter()
                    .map(|point| Point::new(point.y(), point.x()))
                    .collect(),
            );
        }
    }

    Ok(polylines)
}

fn generate_repeat_pattern(
    direction: i32,
    grid_size: f64,
    inwidth: f64,
    inheight: f64,
) -> Result<Vec<Polyline>, ClipperError> {
    let (width, height) = if direction < 0 {
        (inheight, inwidth)
    } else {
        (inwidth, inheight)
    };
    let number_of_lines = (height / grid_size + 1.0) as i32;
    let mut polylines = Vec::with_capacity(number_of_lines as usize);

    for index in 0..number_of_lines {
        let y = grid_size * f64::from(index);
        let mut start = checked_point(0.0, y)?;
        let mut end = checked_point(width, y)?;
        if direction < 0 {
            start = Point::new(start.y(), start.x());
            end = Point::new(end.y(), end.x());
        }
        polylines.push(Polyline::new(vec![start, end]));
    }

    Ok(polylines)
}
