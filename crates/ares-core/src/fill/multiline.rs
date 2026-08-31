//! Straight scanline families shared by Grid, Triangles, Stars, and Cubic.
//! Rewrites `FillRectilinear::fill_surface_by_multilines`
//! (`FillRectilinear.cpp:2996-3046`).

use super::{
    checked_rotate::{rotate_points, rotate_points_with_trig},
    connect::{FillBoundary, FillConnectionParams, connect_infill_polygons},
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline,
    intersection_open_polylines, offset_expolygon,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MultilineFillParams {
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
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

pub(crate) fn source_base_angle(angle: f32) -> f32 {
    angle + std::f32::consts::FRAC_PI_2
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: MultilineFillParams,
    sweeps: &[Sweep],
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    debug_assert!(!sweeps.is_empty());
    let reference = center(surface);
    let expansion =
        (params.overlap + 0.5 * f64::from(params.multiline) * params.spacing) / scale.factor();
    let expanded = offset_expolygon(surface, expansion as f32, JoinType::Miter, 3.0)?;
    if expanded.is_empty() {
        return Ok(Vec::new());
    }
    let contraction = (-0.5 * params.spacing / scale.factor()) as f32;
    let mut contracted = offset_expolygon(surface, contraction, JoinType::Miter, 3.0)?;
    if contracted.is_empty() {
        contracted.push(surface.clone());
    }
    let mut boundaries = Vec::new();
    for component in contracted {
        let (contour, holes) = component.into_parts();
        boundaries.push(contour);
        boundaries.extend(holes);
    }

    let family_density = params.density / sweeps.len() as f32;
    let line_width = scale
        .checked_scale(params.spacing)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let epsilon = scale
        .checked_scale(1.0e-4)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let x_margin = line_width
        .checked_add(epsilon)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let mut lines = Vec::new();
    for sweep in sweeps {
        let mut family = Vec::new();
        for component in &expanded {
            family.extend(generate_family(FamilyRequest {
                component,
                source: surface,
                reference,
                params,
                density: family_density,
                sweep: *sweep,
                x_margin,
                scale,
            })?);
        }
        lines.extend(intersection_open_polylines(&family, &boundaries)?);
    }
    if lines.is_empty() {
        return Ok(Vec::new());
    }
    let bbox = BoundingBox::from_expolygon(surface)
        .expect("the multiline fill surface contour must be nonempty");
    connect_infill_polygons(
        lines,
        FillBoundary {
            polygons: &boundaries,
            bbox,
        },
        params.spacing,
        FillConnectionParams {
            anchor_length: params.anchor_length,
            anchor_length_max: params.anchor_length_max,
            multiline: params.multiline,
            dont_sort: params.dont_sort,
        },
        scale,
    )
}

struct FamilyRequest<'a> {
    component: &'a ExPolygon,
    source: &'a ExPolygon,
    reference: Point,
    params: MultilineFillParams,
    density: f32,
    sweep: Sweep,
    x_margin: i64,
    scale: CoordinateScale,
}

fn generate_family(request: FamilyRequest<'_>) -> Result<Vec<Polyline>, ClipperError> {
    let FamilyRequest {
        component,
        source,
        reference,
        params,
        density,
        sweep,
        x_margin,
        scale,
    } = request;
    let angle = -(params.angle + sweep.angle);
    let rotated_reference = rotate_points(vec![reference], -f64::from(angle))?[0];
    let rotated_source = rotate_expolygon(source, -f64::from(angle))?;
    let (minimum, maximum) = bounds(&rotated_source);
    let rotated = rotate_expolygon(component, -f64::from(angle))?;
    let spacing = ((params.spacing / scale.factor()) * f64::from(params.multiline)
        / f64::from(density)) as i64;
    let shift = scale
        .checked_scale(-f64::from(sweep.shift))
        .ok_or(ClipperError::CoordinateOutOfRange)?
        % spacing;
    let reference_x = rotated_reference
        .x()
        .checked_sub(if shift >= 0 { shift } else { spacing + shift })
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let first_x = align_to_grid(minimum.x(), spacing, reference_x)?;
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
    let x_min = minimum
        .x()
        .checked_add(x_margin)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let x_max = maximum
        .x()
        .checked_sub(x_margin)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let mut lines = (0..count)
        .map(|index| {
            let x = i64::try_from(index)
                .ok()
                .and_then(|index| index.checked_mul(spacing))
                .and_then(|delta| first_x.checked_add(delta))
                .ok_or(ClipperError::CoordinateOutOfRange)?;
            Ok((x >= x_min && x <= x_max)
                .then(|| Polyline::new(vec![Point::new(x, start_y), Point::new(x, end_y)])))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    lines = super::multiline_offset::apply(lines, params.multiline, params.spacing, scale)?;
    let (contour, holes) = rotated.into_parts();
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour);
    clip.extend(holes);
    lines = intersection_open_polylines(&lines, &clip)?;
    for line in &mut lines {
        if line.points().first().unwrap().y() > line.points().last().unwrap().y() {
            line.reverse();
        }
    }
    rotate_polylines(&mut lines, f64::from(angle))?;
    Ok(lines)
}

fn center(expolygon: &ExPolygon) -> Point {
    let (minimum, maximum) = bounds(expolygon);
    Point::new(
        minimum.x() + (maximum.x() - minimum.x()) / 2,
        minimum.y() + (maximum.y() - minimum.y()) / 2,
    )
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
