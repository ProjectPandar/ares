use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Line, Point, Polygon, Polyline};

const MIN_COORDINATE: f64 = i64::MIN as f64;
const MAX_COORDINATE_EXCLUSIVE: f64 = -MIN_COORDINATE;

pub(super) fn checked_point(x: f64, y: f64) -> Result<Point, ClipperError> {
    Ok(Point::new(checked_round(x)?, checked_round(y)?))
}

fn checked_round(value: f64) -> Result<i64, ClipperError> {
    let rounded = value.round();
    if rounded.is_finite() && (MIN_COORDINATE..MAX_COORDINATE_EXCLUSIVE).contains(&rounded) {
        Ok(rounded as i64)
    } else {
        Err(ClipperError::CoordinateOutOfRange)
    }
}

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

fn rotate_points(points: Vec<Point>, angle: f64) -> Result<Vec<Point>, ClipperError> {
    rotate_points_with_trig(points, angle.cos(), angle.sin())
}

fn rotate_points_with_trig(
    points: Vec<Point>,
    cosine: f64,
    sine: f64,
) -> Result<Vec<Point>, ClipperError> {
    points
        .into_iter()
        .map(|point| {
            let x = point.x() as f64;
            let y = point.y() as f64;
            checked_point(cosine * x - sine * y, cosine * y + sine * x)
        })
        .collect()
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
