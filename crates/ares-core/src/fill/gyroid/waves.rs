//! Gyroid wave period construction (`FillGyroid.cpp` make_one_period /
//! make_wave family) and the point/contour helpers.

use super::super::checked_rotate::{rotate_points, rotate_points_with_trig};
use super::{EPSILON, WaveContext, WavePlacement};
use crate::geometry::{ClipperError, ExPolygon, Line, Point, Polygon, Polyline};

pub(super) fn make_one_period(context: WaveContext, flip: bool) -> Vec<(f64, f64)> {
    let limit = 2.0 * std::f64::consts::PI;
    let mut points = Vec::with_capacity((limit / context.tolerance / 3.0).ceil() as usize);
    let mut x = 0.0;
    while x < limit - EPSILON {
        points.push((x, context.value(x, flip)));
        x += std::f64::consts::FRAC_PI_2;
    }
    points.push((limit, context.value(limit, flip)));
    loop {
        let size = points.len();
        for index in 1..size {
            let left = points[index - 1];
            let right = points[index];
            let x = left.0 + (right.0 - left.0) / 2.0;
            let point = (x, context.value(x, flip));
            let cross =
                (point.0 - left.0) * (point.1 - right.1) - (point.1 - left.1) * (point.0 - right.0);
            if cross.abs() > context.tolerance * context.tolerance {
                points.push(point);
            }
        }
        if points.len() == size {
            break;
        }
        points.sort_by(|left, right| left.0.total_cmp(&right.0));
    }
    points
}

pub(super) fn make_wave(
    period_points: &[(f64, f64)],
    placement: WavePlacement,
    vertical: bool,
) -> Result<Polyline, ClipperError> {
    let period = period_points.last().unwrap().0;
    let count = period_points.len() - 1;
    let mut x0 = (placement.minimum / period).floor() * period;
    let mut index = 0;
    while index + 1 < count && x0 + period_points[index + 1].0 <= placement.minimum {
        index += 1;
    }
    let mut points =
        Vec::with_capacity(((placement.maximum - x0) / period + 1.0) as usize * count + 1);
    loop {
        let (mut x, mut y) = (
            x0 + period_points[index].0,
            period_points[index].1 + placement.offset,
        );
        let last = x >= placement.maximum;
        if vertical {
            std::mem::swap(&mut x, &mut y);
        }
        points.push(checked_point(
            x * placement.scale_factor,
            y * placement.scale_factor,
        )?);
        if last {
            break;
        }
        index += 1;
        if index == count {
            index = 0;
            x0 += period;
        }
    }
    Ok(Polyline::new(points))
}

pub(super) fn checked_point(x: f64, y: f64) -> Result<Point, ClipperError> {
    if !x.is_finite()
        || !y.is_finite()
        || x < i64::MIN as f64
        || x >= -(i64::MIN as f64)
        || y < i64::MIN as f64
        || y >= -(i64::MIN as f64)
    {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    Ok(Point::new(x as i64, y as i64))
}

pub(super) fn contour_bounds(contour: &Polygon) -> (Point, Point) {
    contour.points().iter().fold(
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

pub(super) fn translate(polylines: &mut [Polyline], offset: Point) -> Result<(), ClipperError> {
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(
            points
                .into_iter()
                .map(|point| {
                    Ok(Point::new(
                        point
                            .x()
                            .checked_add(offset.x())
                            .ok_or(ClipperError::CoordinateOutOfRange)?,
                        point
                            .y()
                            .checked_add(offset.y())
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

pub(super) fn rotate_expolygon(
    expolygon: &ExPolygon,
    angle: f64,
) -> Result<ExPolygon, ClipperError> {
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

pub(super) fn rotate_polylines(polylines: &mut [Polyline], angle: f64) -> Result<(), ClipperError> {
    let (cosine, sine) = (angle.cos(), angle.sin());
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(rotate_points_with_trig(points, cosine, sine)?);
    }
    Ok(())
}
