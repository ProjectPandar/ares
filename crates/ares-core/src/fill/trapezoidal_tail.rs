//! `FillRectilinear.cpp:3429-3485`: shared trapezoidal fill post-row tail.
use super::{
    checked_rotate::rotate_points,
    connect::{FillBoundary, FillConnectionParams, connect_infill_polygons},
    multiline::MultilineFillParams,
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, Polyline,
    intersection_open_polylines, offset_expolygon,
};

pub(super) fn finish(
    rotated: ExPolygon,
    mut lines: Vec<Polyline>,
    params: MultilineFillParams,
    (angle, row_direction): (f64, [f64; 2]),
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // The scoped zero-factor smoothing path returns before touching points
    // (FillCornerSmoothing.cpp:296-302), before this centerline crop.
    let d1 = (params.spacing / scale.factor()) as i64 * i64::from(params.multiline);
    let expanded = offset_expolygon(&rotated, (d1 / 2) as f32, JoinType::Miter, 3.0)?;
    let mut centerline_bounds = Vec::new();
    for component in expanded {
        let (contour, holes) = component.into_parts();
        centerline_bounds.push(contour);
        centerline_bounds.extend(holes);
    }
    lines = intersection_open_polylines(&lines, &centerline_bounds)?;
    // multiline_fill's spacing parameter is float, unlike Fill::spacing.
    lines = super::multiline_offset::apply(
        lines,
        params.multiline,
        f64::from(params.spacing as f32),
        scale,
    )?;
    for line in &mut lines {
        let points = line.points_mut();
        if points.len() <= 3 || points.first() != points.last() {
            continue;
        }
        points.pop();
        let dot = |index: usize| {
            row_direction[0] * points[index].x() as f64
                + row_direction[1] * points[index].y() as f64
        };
        let mut first = 0;
        for index in 1..points.len() {
            if dot(index) < dot(first) {
                first = index;
            }
        }
        points.rotate_left(first);
        points.push(points[0]);
    }
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

pub(super) fn rotate_surface(surface: &ExPolygon, angle: f64) -> Result<ExPolygon, ClipperError> {
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
