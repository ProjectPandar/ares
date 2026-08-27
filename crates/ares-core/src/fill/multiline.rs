//! Straight scanline families shared by Grid, Triangles, Stars, and Cubic.
//! Rewrites `FillRectilinear::fill_surface_by_multilines`
//! (`FillRectilinear.cpp:2996-3046`).

use super::{
    checked_rotate::{rotate_points, rotate_points_with_trig},
    connect::{FillConnectionParams, connect_infill},
};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline,
    intersection_open_polylines, offset_expolygon,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MultilineFillParams {
    pub(crate) spacing: f64,
    pub(crate) angle: f32,
    pub(crate) density: f32,
    pub(crate) multiline: i32,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) dont_sort: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Sweep {
    pub(crate) angle: f32,
    pub(crate) shift: f32,
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: MultilineFillParams,
    sweeps: &[Sweep],
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    debug_assert!(!sweeps.is_empty());
    let contraction = (-0.5 * params.spacing / scale.factor()) as f32;
    let mut components = offset_expolygon(surface, contraction, JoinType::Miter, 3.0)?;
    if components.is_empty() {
        components.push(surface.clone());
    }
    let family_density = params.density / sweeps.len() as f32;
    let mut output = Vec::new();
    for component in components {
        let mut lines = Vec::new();
        for sweep in sweeps {
            lines.extend(generate_family(
                &component,
                params,
                family_density,
                *sweep,
                scale,
            )?);
        }
        if lines.is_empty() {
            continue;
        }
        output.extend(connect_infill(
            lines,
            &component,
            params.spacing,
            FillConnectionParams {
                anchor_length: params.anchor_length,
                anchor_length_max: params.anchor_length_max,
                multiline: params.multiline,
                dont_sort: params.dont_sort,
            },
            scale,
        )?);
    }
    Ok(output)
}

fn generate_family(
    component: &ExPolygon,
    params: MultilineFillParams,
    density: f32,
    sweep: Sweep,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let angle = params.angle + std::f32::consts::FRAC_PI_2 + sweep.angle;
    let rotated = rotate_expolygon(component, -f64::from(angle))?;
    let (minimum, maximum) = bounds(&rotated);
    let spacing = ((params.spacing / scale.factor()) * f64::from(params.multiline)
        / f64::from(density)) as i64;
    let shift = scale
        .checked_scale(f64::from(sweep.shift))
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let first_x = align_to_grid(minimum.x(), spacing, shift)?;
    let height_padding = spacing;
    let start_y = minimum
        .y()
        .checked_sub(height_padding)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let end_y = maximum
        .y()
        .checked_add(height_padding)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let count = usize::try_from(
        (i128::from(maximum.x()) - i128::from(first_x)).div_euclid(i128::from(spacing)) + 1,
    )
    .map_err(|_| ClipperError::CoordinateOutOfRange)?;
    let mut lines = (0..count)
        .map(|index| {
            let x = i64::try_from(index)
                .ok()
                .and_then(|index| index.checked_mul(spacing))
                .and_then(|delta| first_x.checked_add(delta))
                .ok_or(ClipperError::CoordinateOutOfRange)?;
            Ok(Polyline::new(vec![
                Point::new(x, start_y),
                Point::new(x, end_y),
            ]))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (contour, holes) = rotated.into_parts();
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour);
    clip.extend(holes);
    lines = intersection_open_polylines(&lines, &clip)?;
    rotate_polylines(&mut lines, f64::from(angle))?;
    Ok(lines)
}

fn bounds(expolygon: &ExPolygon) -> (Point, Point) {
    expolygon
        .contour()
        .points()
        .iter()
        .chain(expolygon.holes().iter().flat_map(|hole| hole.points()))
        .fold(
            (
                Point::new(i64::MAX, i64::MAX),
                Point::new(i64::MIN, i64::MIN),
            ),
            |(minimum, maximum), point| {
                (
                    Point::new(minimum.x().min(point.x()), minimum.y().min(point.y())),
                    Point::new(maximum.x().max(point.x()), maximum.y().max(point.y())),
                )
            },
        )
}

fn align_to_grid(coordinate: i64, spacing: i64, shift: i64) -> Result<i64, ClipperError> {
    i64::try_from(
        i128::from(shift)
            + (i128::from(coordinate) - i128::from(shift)).div_euclid(i128::from(spacing))
                * i128::from(spacing),
    )
    .map_err(|_| ClipperError::CoordinateOutOfRange)
}

fn rotate_expolygon(expolygon: &ExPolygon, angle: f64) -> Result<ExPolygon, ClipperError> {
    let rotate =
        |polygon: &Polygon| rotate_points(polygon.points().to_vec(), angle).map(Polygon::new);
    Ok(ExPolygon::new(
        rotate(expolygon.contour())?,
        expolygon
            .holes()
            .iter()
            .map(rotate)
            .collect::<Result<_, _>>()?,
    ))
}

fn rotate_polylines(polylines: &mut [Polyline], angle: f64) -> Result<(), ClipperError> {
    let (cosine, sine) = (angle.cos(), angle.sin());
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(rotate_points_with_trig(points, cosine, sine)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
