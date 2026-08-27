use super::super::checked_rotate::{rotate_points, rotate_points_with_trig};
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Line, Point, Polygon, Polyline};

pub(super) fn rotate_expolygon(
    expolygon: ExPolygon,
    angle: f64,
) -> Result<ExPolygon, ClipperError> {
    let (contour, holes) = expolygon.into_parts();
    Ok(ExPolygon::new(
        rotate_polygon(contour, angle)?,
        holes
            .into_iter()
            .map(|hole| rotate_polygon(hole, angle))
            .collect::<Result<_, _>>()?,
    ))
}

fn rotate_polygon(polygon: Polygon, angle: f64) -> Result<Polygon, ClipperError> {
    Ok(Polygon::new(rotate_points(polygon.into_points(), angle)?))
}

pub(super) fn rotate_polylines(polylines: &mut [Polyline], angle: f64) -> Result<(), ClipperError> {
    let cosine = angle.cos();
    let sine = angle.sin();
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(rotate_points_with_trig(points, cosine, sine)?);
    }
    Ok(())
}

pub(crate) fn line_spacing(
    spacing: f64,
    density: f32,
    multiline: i32,
    scale: CoordinateScale,
) -> i64 {
    let density_adjusted = f64::from(density / multiline as f32);
    let mut line_spacing = ((spacing / scale.factor()) / density_adjusted) as i64;
    if f64::from(density) < 0.999 {
        line_spacing = (line_spacing as f64 * 1.08) as i64;
    }
    line_spacing
}

pub(super) fn aligned_contour_bounds(
    contour: &Polygon,
    spacing: f64,
    density: f32,
    multiline: i32,
    scale: CoordinateScale,
) -> Result<(Point, f64, f64), ClipperError> {
    let mut points = contour.points().iter().copied();
    let first = points
        .next()
        .expect("trusted CrossHatch contour is nonempty");
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x(), first.y(), first.x(), first.y());
    for point in points {
        min_x = min_x.min(point.x());
        min_y = min_y.min(point.y());
        max_x = max_x.max(point.x());
        max_y = max_y.max(point.y());
    }

    let cell = line_spacing(spacing, density, multiline, scale)
        .checked_mul(4)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    min_x = align_to_grid(min_x, cell)?;
    min_y = align_to_grid(min_y, cell)?;
    let width = max_x
        .checked_sub(min_x)
        .ok_or(ClipperError::CoordinateOutOfRange)? as f64;
    let height = max_y
        .checked_sub(min_y)
        .ok_or(ClipperError::CoordinateOutOfRange)? as f64;
    Ok((Point::new(min_x, min_y), width, height))
}

fn align_to_grid(coordinate: i64, spacing: i64) -> Result<i64, ClipperError> {
    let quotient = if coordinate < 0 {
        coordinate
            .checked_sub(spacing)
            .and_then(|value| value.checked_add(1))
            .ok_or(ClipperError::CoordinateOutOfRange)?
            / spacing
    } else {
        coordinate / spacing
    };
    quotient
        .checked_mul(spacing)
        .ok_or(ClipperError::CoordinateOutOfRange)
}

pub(super) fn translate_polylines(
    polylines: &mut [Polyline],
    delta: Point,
) -> Result<(), ClipperError> {
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(
            points
                .into_iter()
                .map(|point| {
                    Ok(Point::new(
                        point
                            .x()
                            .checked_add(delta.x())
                            .ok_or(ClipperError::CoordinateOutOfRange)?,
                        point
                            .y()
                            .checked_add(delta.y())
                            .ok_or(ClipperError::CoordinateOutOfRange)?,
                    ))
                })
                .collect::<Result<_, _>>()?,
        );
    }
    Ok(())
}

pub(super) fn polyline_length(polyline: &Polyline) -> f64 {
    polyline
        .points()
        .windows(2)
        .map(|points| Line::new(points[0], points[1]).length())
        .sum()
}
