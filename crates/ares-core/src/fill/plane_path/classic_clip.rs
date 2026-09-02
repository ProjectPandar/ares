use std::cmp::Ordering;

use crate::geometry::{ClipperError, Point, Polygon, Polyline};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Ratio {
    numerator: i128,
    denominator: i128,
}

impl Ratio {
    const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };
    const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    fn new(mut numerator: i128, mut denominator: i128) -> Self {
        if denominator < 0 {
            numerator = -numerator;
            denominator = -denominator;
        }
        Self {
            numerator,
            denominator,
        }
    }

    fn compare(self, other: Self) -> Ordering {
        (self.numerator * other.denominator).cmp(&(other.numerator * self.denominator))
    }

    fn midpoint(self, other: Self) -> f64 {
        0.5 * (self.numerator as f64 / self.denominator as f64
            + other.numerator as f64 / other.denominator as f64)
    }
}

#[derive(Clone, Copy)]
struct Cut {
    ratio: Ratio,
    point: Point,
}

pub(super) fn intersect(
    subjects: &[Polyline],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    let mut output = Vec::new();
    for subject in subjects {
        intersect_subject(subject, clip, &mut output)?;
    }
    sort_scanbeam_outputs(&mut output);
    for polyline in &mut output {
        let first = polyline.front().unwrap();
        let last = polyline.back().unwrap();
        if (first.y() - last.y()).abs() <= 1 && first.x() < last.x() {
            polyline.reverse();
        }
    }
    Ok(output)
}

// Clipper 6 allocates open OutRecs while descending its scanbeam. Equal-height
// pairs are emitted from the active-edge side selected at that scanline.
fn sort_scanbeam_outputs(output: &mut [Polyline]) {
    output.sort_unstable_by_key(|polyline| {
        std::cmp::Reverse(
            polyline
                .points()
                .iter()
                .map(|point| point.y())
                .max()
                .unwrap(),
        )
    });
    let mut start = 0;
    while start < output.len() {
        let maximum = output[start]
            .points()
            .iter()
            .map(|point| point.y())
            .max()
            .unwrap();
        let mut end = start + 1;
        while end < output.len()
            && maximum
                - output[end]
                    .points()
                    .iter()
                    .map(|point| point.y())
                    .max()
                    .unwrap()
                <= 1
        {
            end += 1;
        }
        output[start..end].sort_unstable_by_key(|polyline| {
            let minimum_x = polyline
                .points()
                .iter()
                .map(|point| point.x())
                .min()
                .unwrap();
            if maximum < 0 { -minimum_x } else { minimum_x }
        });
        start = end;
    }
}

fn intersect_subject(
    subject: &Polyline,
    clip: &[Polygon],
    output: &mut Vec<Polyline>,
) -> Result<(), ClipperError> {
    for segment in subject.points().windows(2) {
        intersect_segment(segment[0], segment[1], clip, output)?;
    }
    Ok(())
}

fn intersect_segment(
    start: Point,
    end: Point,
    clip: &[Polygon],
    output: &mut Vec<Polyline>,
) -> Result<(), ClipperError> {
    let cuts = intersection_cuts(start, end, clip);
    for pair in cuts.windows(2) {
        let midpoint = pair[0].ratio.midpoint(pair[1].ratio);
        let sample = interpolate_f64(start, end, midpoint);
        if inside_nonzero(sample, clip) && pair[0].point != pair[1].point {
            append_segment(output, pair[0].point, pair[1].point);
        }
    }
    Ok(())
}

fn intersection_cuts(start: Point, end: Point, clip: &[Polygon]) -> Vec<Cut> {
    let mut cuts = vec![
        Cut {
            ratio: Ratio::ZERO,
            point: start,
        },
        Cut {
            ratio: Ratio::ONE,
            point: end,
        },
    ];
    for polygon in clip {
        append_polygon_intersections(start, end, polygon, &mut cuts);
    }
    cuts.sort_by(|left, right| left.ratio.compare(right.ratio));
    cuts.dedup_by(|left, right| left.ratio.compare(right.ratio) == Ordering::Equal);
    cuts
}

fn append_polygon_intersections(start: Point, end: Point, polygon: &Polygon, cuts: &mut Vec<Cut>) {
    let points = polygon.points();
    for (first, second) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        if let Some(ratio) = intersection_ratio(start, end, *first, *second) {
            cuts.push(Cut {
                ratio,
                point: clipper_intersection(start, end, *first, *second),
            });
        }
    }
}

fn append_segment(output: &mut Vec<Polyline>, first: Point, second: Point) {
    if let Some(last) = output.last_mut()
        && last.back() == Some(first)
    {
        let mut points = std::mem::replace(last, Polyline::new(Vec::new())).into_points();
        if points.last() != Some(&second) {
            points.push(second);
        }
        *last = Polyline::new(points);
    } else {
        output.push(Polyline::new(vec![first, second]));
    }
}

fn intersection_ratio(a: Point, b: Point, c: Point, d: Point) -> Option<Ratio> {
    let r = (i128::from(b.x() - a.x()), i128::from(b.y() - a.y()));
    let s = (i128::from(d.x() - c.x()), i128::from(d.y() - c.y()));
    let denominator = cross(r, s);
    if denominator == 0 {
        return None;
    }
    let delta = (i128::from(c.x() - a.x()), i128::from(c.y() - a.y()));
    let t = Ratio::new(cross(delta, s), denominator);
    let u = Ratio::new(cross(delta, r), denominator);
    (Ratio::ZERO.compare(t) != Ordering::Greater
        && t.compare(Ratio::ONE) != Ordering::Greater
        && Ratio::ZERO.compare(u) != Ordering::Greater
        && u.compare(Ratio::ONE) != Ordering::Greater)
        .then_some(t)
}

const fn cross(first: (i128, i128), second: (i128, i128)) -> i128 {
    first.0 * second.1 - first.1 * second.0
}

// Clipper 6 `IntersectPoint`: retain its double slope/intercept evaluation and
// half-away cast, including the one-unit results that exact rationals avoid.
fn clipper_intersection(
    first_start: Point,
    first_end: Point,
    second_start: Point,
    second_end: Point,
) -> Point {
    let first = Edge::new(first_start, first_end);
    let second = Edge::new(second_start, second_end);
    let (x, y) = if first.horizontal {
        (
            round_clipper(second.x_at(first_start.y() as f64)),
            first_start.y(),
        )
    } else if second.horizontal {
        (
            round_clipper(first.x_at(second_start.y() as f64)),
            second_start.y(),
        )
    } else if first.slope == 0.0 {
        let x = first.bottom.x();
        let y = round_clipper(
            x as f64 / second.slope + second.bottom.y() as f64
                - second.bottom.x() as f64 / second.slope,
        );
        (x, y)
    } else if second.slope == 0.0 {
        let x = second.bottom.x();
        let y = round_clipper(
            x as f64 / first.slope + first.bottom.y() as f64
                - first.bottom.x() as f64 / first.slope,
        );
        (x, y)
    } else {
        let first_intercept = first.intercept();
        let second_intercept = second.intercept();
        let y = (second_intercept - first_intercept) / (first.slope - second.slope);
        let x = if first.slope.abs() < second.slope.abs() {
            first.slope * y + first_intercept
        } else {
            second.slope * y + second_intercept
        };
        (round_clipper(x), round_clipper(y))
    };
    // Clipper 6 clamps intersections below both edge tops to the higher
    // top and recomputes x via `TopX` on the more vertical edge
    // (`clipper.cpp:404-409`); corner-adjacent cut points differ without it.
    let (mut x, mut y) = (x, y);
    let top_first = first.top.y();
    let top_second = second.top.y();
    if y < top_first || y < top_second {
        y = top_first.max(top_second);
        x = top_x(
            if more_vertical(&first, &second) {
                &first
            } else {
                &second
            },
            y,
        );
    }
    Point::new(x, y)
}

struct Edge {
    bottom: Point,
    top: Point,
    slope: f64,
    horizontal: bool,
}

impl Edge {
    fn new(first: Point, second: Point) -> Self {
        let (bottom, top) = if first.y() > second.y() {
            (first, second)
        } else {
            (second, first)
        };
        let horizontal = first.y() == second.y();
        let slope = if horizontal {
            0.0
        } else {
            (second.x() - first.x()) as f64 / (second.y() - first.y()) as f64
        };
        Self {
            bottom,
            top,
            slope,
            horizontal,
        }
    }

    fn intercept(&self) -> f64 {
        self.bottom.x() as f64 - self.bottom.y() as f64 * self.slope
    }

    fn x_at(&self, y: f64) -> f64 {
        // Clipper 6's `TopX` association — the exact top coordinate when
        // asked at the top, otherwise `Bot.x + Dx * (y - Bot.y)` — avoids
        // the fp re-association flip at half-unit boundaries.
        if y as i64 == self.top.y() {
            return self.top.x() as f64;
        }
        self.bottom.x() as f64 + self.slope * (y - self.bottom.y() as f64)
    }
}

// Clipper 6 `FRound` (`clipper.cpp:88-92`): floor-based half-up rounding
// with the `0.49999999999999994` guard — NOT half-away-from-zero
// (negative halves round toward +inf: -153013.5 -> -153013).
fn round_clipper(value: f64) -> i64 {
    if value == 0.499_999_999_999_999_94 {
        return 0;
    }
    (value + 0.5).floor() as i64
}

// Clipper 6 `TopX` (`clipper.cpp:350-356`).
fn top_x(edge: &Edge, y: i64) -> i64 {
    if y == edge.top.y() {
        edge.top.x()
    } else {
        edge.bottom.x() + round_clipper(edge.slope * (y - edge.bottom.y()) as f64)
    }
}

// Clipper 6 compares `|Dx|` to pick the more vertical edge for the clamped
// x; a horizontal edge carries the `HORIZONTAL` sentinel, so it never wins.
fn more_vertical(first: &Edge, second: &Edge) -> bool {
    if first.horizontal {
        false
    } else {
        second.horizontal || first.slope.abs() < second.slope.abs()
    }
}

fn interpolate_f64(start: Point, end: Point, ratio: f64) -> (f64, f64) {
    (
        start.x() as f64 + (end.x() - start.x()) as f64 * ratio,
        start.y() as f64 + (end.y() - start.y()) as f64 * ratio,
    )
}

fn inside_nonzero(point: (f64, f64), polygons: &[Polygon]) -> bool {
    polygons
        .iter()
        .map(|polygon| winding(point, polygon))
        .sum::<i32>()
        != 0
}

fn winding(point: (f64, f64), polygon: &Polygon) -> i32 {
    let mut winding = 0;
    let points = polygon.points();
    for (first, second) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        let first_y = first.y() as f64;
        let second_y = second.y() as f64;
        let side = (second.x() - first.x()) as f64 * (point.1 - first_y)
            - (second_y - first_y) * (point.0 - first.x() as f64);
        if first_y <= point.1 {
            if second_y > point.1 && side > 0.0 {
                winding += 1;
            }
        } else if second_y <= point.1 && side < 0.0 {
            winding -= 1;
        }
    }
    winding
}

#[cfg(test)]
mod tests;
