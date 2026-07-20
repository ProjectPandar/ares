use std::ops::{Add, Mul, Sub};

use crate::geometry::{ExPolygon, Point, Polygon};

use super::bounds::negative_outer;
use super::predicates::HI_RANGE;
use super::{ClipOperation, ClipperError, ClipperOptions, ClosedClipper, FillRule, PathRole};

const SHORTEST_EDGE_FACTOR: f64 = 0.005;

pub(crate) fn variable_offset_inner_ex(
    input: &ExPolygon,
    deltas: &[Vec<f32>],
    miter_limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let coordinate_out_of_range = std::iter::once(input.contour())
        .chain(input.holes())
        .flat_map(|polygon| polygon.points())
        .any(|point| {
            !(-HI_RANGE..=HI_RANGE).contains(&point.x())
                || !(-HI_RANGE..=HI_RANGE).contains(&point.y())
        });
    if coordinate_out_of_range
        || !miter_limit.is_finite()
        || deltas.iter().flatten().any(|delta| !delta.is_finite())
    {
        return Err(ClipperError::CoordinateOutOfRange);
    }

    debug_assert_eq!(input.holes().len() + 1, deltas.len());
    debug_assert_eq!(input.contour().points().len(), deltas[0].len());
    debug_assert!(
        input
            .holes()
            .iter()
            .zip(&deltas[1..])
            .all(|(hole, values)| hole.points().len() == values.len())
    );
    debug_assert!(deltas.iter().flatten().all(|&delta| delta <= 0.0));

    let contour_path = mitered_offset_path(input.contour(), &deltas[0], miter_limit)?;
    let contours = repair_inner(contour_path)?;
    let mut holes = Vec::with_capacity(input.holes().len());
    for (hole, hole_deltas) in input.holes().iter().zip(&deltas[1..]) {
        let path = mitered_offset_path(hole, hole_deltas, miter_limit)?;
        holes.append(&mut repair_outer(path)?);
    }

    if holes.is_empty() {
        return Ok(contours
            .into_iter()
            .map(|contour| ExPolygon::new(contour, Vec::new()))
            .collect());
    }

    let mut clipper = ClosedClipper::new(ClipperOptions::default());
    clipper.add_closed_paths(&contours, PathRole::Subject)?;
    clipper.add_closed_paths(&holes, PathRole::Clip)?;
    Ok(clipper
        .execute_polytree(
            ClipOperation::Difference,
            FillRule::NonZero,
            FillRule::NonZero,
        )
        .into_expolygons())
}

type VariableOffsetFn = fn(&ExPolygon, &[Vec<f32>], f64) -> Result<Vec<ExPolygon>, ClipperError>;
const _: VariableOffsetFn = variable_offset_inner_ex;

fn mitered_offset_path(
    contour: &Polygon,
    deltas: &[f32],
    miter_limit: f64,
) -> Result<Polygon, ClipperError> {
    let points = contour.points();
    debug_assert_eq!(points.len(), deltas.len());
    if points.len() <= 2 {
        return Ok(Polygon::new(Vec::new()));
    }

    let threshold = if miter_limit > 2.0 {
        2.0 / (miter_limit * miter_limit)
    } else {
        0.5
    };
    let mut max_delta = deltas[0];
    for &delta in &deltas[1..] {
        if max_delta < delta {
            max_delta = delta;
        }
    }
    let minimum_length = f64::from(max_delta) * SHORTEST_EDGE_FACTOR;
    let minimum_length_squared = minimum_length * minimum_length;
    let first = Vector::from(points[0]);

    let mut previous_index = points.len() - 1;
    while previous_index > 0
        && (Vector::from(points[previous_index]) - first).squared_norm() <= minimum_length_squared
    {
        previous_index -= 1;
    }
    if previous_index == 0 {
        return Ok(Polygon::new(Vec::new()));
    }

    let last = previous_index;
    let mut previous_normal = (first - Vector::from(points[previous_index]))
        .perpendicular()
        .normalized();
    let mut point = first;
    let mut index = 0;
    let mut output = Vec::with_capacity(points.len() * 2);

    loop {
        let mut next_index = index + 1;
        while next_index <= last
            && (Vector::from(points[next_index]) - point).squared_norm() <= minimum_length_squared
        {
            next_index += 1;
        }
        let next = if next_index > last {
            index = last;
            first
        } else {
            Vector::from(points[next_index])
        };

        let next_normal = (next - point).perpendicular().normalized();
        let delta = f64::from(deltas[index]);
        let sine = previous_normal.cross(next_normal).clamp(-1.0, 1.0);
        let convexity = sine * delta;
        let dot = previous_normal.dot(next_normal);
        if convexity <= -1.0 {
            append_rounded(&mut output, point + previous_normal * delta)?;
            append_rounded(&mut output, point)?;
            append_rounded(&mut output, point + next_normal * delta)?;
        } else if convexity < 1.0 && dot > 0.0 {
            append_rounded(&mut output, point + previous_normal * delta)?;
        } else {
            let ratio = 1.0 + dot;
            if ratio >= threshold {
                append_rounded(
                    &mut output,
                    point + (previous_normal + next_normal) * (delta / ratio),
                )?;
            } else {
                let tangent = (sine.atan2(dot) / 4.0).tan();
                append_rounded(
                    &mut output,
                    point + (previous_normal - previous_normal.perpendicular() * tangent) * delta,
                )?;
                append_rounded(
                    &mut output,
                    point + (next_normal + next_normal.perpendicular() * tangent) * delta,
                )?;
            }
        }

        if index == last {
            break;
        }
        previous_normal = next_normal;
        point = next;
        index = next_index;
    }

    Ok(Polygon::new(output))
}

fn repair_inner(path: Polygon) -> Result<Vec<Polygon>, ClipperError> {
    if path.points().is_empty() {
        return Ok(Vec::new());
    }

    let mut clipper = ClosedClipper::new(ClipperOptions {
        reverse_solution: true,
        preserve_collinear: false,
        strictly_simple: false,
    });
    clipper.add_closed_path(&path, PathRole::Subject)?;
    let outer = negative_outer(clipper.bounds());
    clipper.add_closed_path(&outer, PathRole::Subject)?;
    let mut output =
        clipper.execute_paths(ClipOperation::Union, FillRule::Negative, FillRule::Negative);
    if !output.is_empty() {
        output.remove(0);
    }
    Ok(output)
}

fn repair_outer(path: Polygon) -> Result<Vec<Polygon>, ClipperError> {
    if path.points().is_empty() {
        return Ok(Vec::new());
    }

    let mut clipper = ClosedClipper::new(ClipperOptions::default());
    clipper.add_closed_path(&path, PathRole::Subject)?;
    Ok(clipper.execute_paths(ClipOperation::Union, FillRule::Negative, FillRule::Negative))
}

fn append_rounded(output: &mut Vec<Point>, point: Vector) -> Result<(), ClipperError> {
    output.push(Point::new(
        round_half_away(point.0)?,
        round_half_away(point.1)?,
    ));
    Ok(())
}

fn round_half_away(value: f64) -> Result<i64, ClipperError> {
    let adjusted = value + if value < 0.0 { -0.5 } else { 0.5 };
    if !adjusted.is_finite() {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    let rounded = adjusted.trunc() as i64;
    if !(-HI_RANGE..=HI_RANGE).contains(&rounded) {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    Ok(rounded)
}

#[derive(Clone, Copy)]
struct Vector(f64, f64);

impl Vector {
    fn perpendicular(self) -> Self {
        Self(self.1, -self.0)
    }

    fn squared_norm(self) -> f64 {
        self.0 * self.0 + self.1 * self.1
    }

    fn normalized(self) -> Self {
        let norm = self.squared_norm().sqrt();
        Self(self.0 / norm, self.1 / norm)
    }

    fn dot(self, other: Self) -> f64 {
        self.0 * other.0 + self.1 * other.1
    }

    fn cross(self, other: Self) -> f64 {
        self.0 * other.1 - self.1 * other.0
    }
}

impl From<Point> for Vector {
    fn from(point: Point) -> Self {
        Self(point.x() as f64, point.y() as f64)
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl Mul<f64> for Vector {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self(self.0 * scalar, self.1 * scalar)
    }
}
