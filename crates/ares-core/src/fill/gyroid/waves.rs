//! Gyroid wave period construction (`FillGyroid.cpp` make_one_period /
//! make_wave family) and the point/contour helpers.

use super::super::checked_rotate::{rotate_points, rotate_points_with_trig};
use super::{DENSITY_ADJUST, EPSILON, PATTERN_TOLERANCE, WaveContext, WavePlacement};
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Line, Point, Polygon, Polyline};

pub(super) fn make_one_period(width: f64, context: WaveContext, flip: bool) -> Vec<(f64, f64)> {
    let points = make_one_period_inner(width, context, flip);
    if let Ok(path) = std::env::var("ARES_DUMP_GPTS") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            for (x, y) in &points {
                let _ = writeln!(file, "GP {x:.17e} {y:.17e}");
            }
        }
    }
    points
}

pub(super) fn make_one_period_inner(
    width: f64,
    context: WaveContext,
    flip: bool,
) -> Vec<(f64, f64)> {
    let limit = (2.0 * std::f64::consts::PI).min(width);
    let mut points = Vec::new();
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
    context: WaveContext,
    flip: bool,
) -> Result<Polyline, ClipperError> {
    let mut points = period_points.to_vec();
    let period = points.last().unwrap().0;
    if placement.width != period {
        points.pop();
        let count = points.len();
        loop {
            let source = points[points.len() - count];
            points.push((source.0 + period, source.1));
            if points.last().unwrap().0 >= placement.width - EPSILON {
                break;
            }
        }
        points.push((placement.width, context.value(placement.width, flip)));
    }
    points
        .into_iter()
        .map(|(mut x, mut y)| {
            y = (y + placement.offset).clamp(0.0, placement.height);
            if context.vertical {
                std::mem::swap(&mut x, &mut y);
            }
            checked_point(x * placement.scale_factor, y * placement.scale_factor)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Polyline::new)
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
