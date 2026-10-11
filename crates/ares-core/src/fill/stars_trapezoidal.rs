//! `FillRectilinear.cpp:3048-3085/3260-3384`: Stars multiline mode2.
mod rows;

use super::{
    checked_rotate::rotate_points,
    connect::{FillBoundary, FillConnectionParams, connect_infill_polygons},
    multiline::MultilineFillParams,
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, Polyline,
    intersection_open_polylines, offset_expolygon,
};

pub(crate) struct SurfaceGeometry {
    pub(crate) layer_id: usize,
    pub(crate) thickness_layers: u16,
    pub(crate) bounds: BoundingBox,
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: MultilineFillParams,
    geometry: SurfaceGeometry,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // _infill_direction returns float; the caller adds M_PI_2 in double.
    let angle = f64::from(params.angle) + std::f64::consts::FRAC_PI_2;
    let rotated = rotate_surface(surface, -angle)?;
    let mut lines = rows::generate(params, geometry, scale)?;
    // multiline_fill's spacing parameter is float, unlike Fill::spacing.
    lines = super::multiline_offset::apply(
        lines,
        params.multiline,
        f64::from(params.spacing as f32),
        scale,
    )?;
    let contraction = (-0.5 * params.spacing / scale.factor()) as f32;
    let mut contracted = offset_expolygon(&rotated, contraction, JoinType::Miter, 3.0)?;
    // Source :3350-3351 retains the original surface if contraction is empty.
    if contracted.is_empty() {
        contracted.push(rotated);
    }
    let mut boundaries = Vec::new();
    for component in contracted {
        let (mut contour, holes) = component.into_parts();
        if contour.area() < 0.0 {
            contour.reverse();
        }
        boundaries.push(contour);
        for mut hole in holes {
            if hole.area() >= 0.0 {
                hole.reverse();
            }
            boundaries.push(hole);
        }
    }
    let mut lines = intersection_open_polylines(&lines, &boundaries)?;
    let minimum_length = 0.8 * params.spacing / scale.factor();
    lines.retain(|line| {
        line.points()
            .windows(2)
            .map(|points| crate::geometry::Line::new(points[0], points[1]).length())
            .sum::<f64>()
            >= minimum_length
    });
    if lines.is_empty() {
        return Ok(lines);
    }
    let bbox = BoundingBox::from_polygons(&boundaries).ok_or(ClipperError::CoordinateOutOfRange)?;
    let mut lines = connect_infill_polygons(
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
            dont_sort: false,
        },
        scale,
    )?;
    if angle.abs() >= 1.0e-4 {
        for line in &mut lines {
            let points = std::mem::replace(line, Polyline::new(Vec::new())).into_points();
            *line = Polyline::new(rotate_points(points, angle)?);
        }
    }
    Ok(lines)
}

fn rotate_surface(surface: &ExPolygon, angle: f64) -> Result<ExPolygon, ClipperError> {
    if angle.abs() < 1.0e-4 {
        return Ok(surface.clone());
    }
    let rotate =
        |polygon: &Polygon| rotate_points(polygon.points().to_vec(), angle).map(Polygon::new);
    Ok(ExPolygon::new(
        rotate(surface.contour())?,
        surface
            .holes()
            .iter()
            .map(rotate)
            .collect::<Result<_, _>>()?,
    ))
}
