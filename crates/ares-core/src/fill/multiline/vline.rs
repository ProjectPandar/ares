//! Port of `slice_region_by_vertical_lines` (FillRectilinear.cpp:759-905)
//! for the multiline fill generator. Computes vertical-line/contour
//! intersections with the upstream exact integer rational arithmetic so the
//! emitted line endpoints match the reference slicer to the unit, replacing
//! the general Clipper open-path intersection that rounded rotated miter
//! corners differently.

use crate::geometry::{Coord, Point, Polygon, Polyline};

#[derive(Clone, Copy)]
struct Intersection {
    /// Rational y position numerator (denominator `pos_q`).
    pos_p: i64,
    /// Rational denominator, always positive.
    pos_q: u32,
    /// True where the crossed segment runs toward +x (`OUTER_LOW`).
    low: bool,
    contour: usize,
    segment: usize,
}

impl Intersection {
    /// `SegmentIntersection::pos` (FillRectilinear.cpp:129-139): arithmetic
    /// rounding of the rational towards the nearest integer.
    fn pos(&self) -> Coord {
        let mut p = self.pos_p;
        if p < 0 {
            p -= i64::from(self.pos_q >> 1);
        } else {
            p += i64::from(self.pos_q >> 1);
        }
        (p / i64::from(self.pos_q)) as Coord
    }

    /// `SegmentIntersection::operator<` (FillRectilinear.cpp:275-318): exact
    /// cross-multiplied rational comparison with sign-normalized nominators.
    fn below(&self, other: &Self) -> bool {
        if self.pos_p == 0 || other.pos_p == 0 {
            return self.pos_p < other.pos_p;
        }
        let sign1 = if self.pos_p > 0 { 1i32 } else { -1 };
        let sign2 = if other.pos_p > 0 { 1i32 } else { -1 };
        if sign1 * sign2 < 0 {
            // Mixed signs: the negative nominator is the smaller position.
            return sign1 < 0;
        }
        let left = i128::from(self.pos_p.unsigned_abs()) * i128::from(other.pos_q);
        let right = i128::from(other.pos_p.unsigned_abs()) * i128::from(self.pos_q);
        if sign1 < 0 {
            left > right
        } else {
            left < right
        }
    }
}

/// Slice `polygons` with vertical lines at `x0 + i * spacing` and return, per
/// line, the paired `(x, low) -> (x, high)` spans (upstream
/// `make_fill_lines`, FillRectilinear.cpp:2936-2955).
pub(super) fn vertical_spans(
    polygons: &[Polygon],
    x0: Coord,
    spacing: Coord,
    count: usize,
) -> Vec<Polyline> {
    let mut spans = Vec::new();
    for index in 0..count {
        let x = x0 + (index as Coord).saturating_mul(spacing);
        let mut intersections = line_intersections(polygons, x);
        // Sort with the exact rational comparator; the sort must be stable to
        // break ties like the reference `std::sort` on equal positions.
        intersections.sort_by(|left, right| {
            if left.below(right) {
                std::cmp::Ordering::Less
            } else if right.below(left) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        remove_tangential(&mut intersections, polygons);
        let mut position = 0;
        while position < intersections.len() {
            let low = &intersections[position];
            if !low.low {
                position += 1;
                continue;
            }
            let Some(high) = intersections.get(position + 1) else {
                break;
            };
            if high.low {
                position += 1;
                continue;
            }
            spans.push(Polyline::new(vec![
                Point::new(x, low.pos()),
                Point::new(x, high.pos()),
            ]));
            position += 2;
        }
    }
    spans
}

fn line_intersections(polygons: &[Polygon], x: Coord) -> Vec<Intersection> {
    let mut intersections = Vec::new();
    for (contour_index, polygon) in polygons.iter().enumerate() {
        let points = polygon.points();
        if points.len() < 2 {
            continue;
        }
        for segment in 0..points.len() {
            if let Some(intersection) = segment_intersection(points, segment, contour_index, x) {
                intersections.push(intersection);
            }
        }
    }
    intersections
}

/// `slice_region_by_vertical_lines` duplicate/tangency removal
/// (FillRectilinear.cpp:851-905) for the single-offset (all outer) case.
fn remove_tangential(intersections: &mut Vec<Intersection>, polygons: &[Polygon]) {
    let mut kept = 0;
    for index in 0..intersections.len() {
        let candidate = intersections[index];
        let take =
            kept == 0 || keep_intersection(&mut intersections[kept - 1], candidate, polygons);
        if take {
            intersections[kept] = candidate;
            kept += 1;
        }
    }
    intersections.truncate(kept);
}

/// `sample_grid_pattern` (`FillRectilinear.cpp:3639`): grid-sample an
/// expolygon's interior at `spacing` in both axes, anchored on the *global*
/// bounding box so two disjoint expolygons can't produce points closer than
/// `spacing`. Emits a point per (vertical-line, in-span grid y).
pub(crate) fn sample_grid_pattern(
    polygons: &[Polygon],
    spacing: Coord,
    global_bounding_box: (Point, Point),
) -> Vec<Point> {
    let (min, max) = global_bounding_box;
    let count = ((max.x() - min.x() + spacing - 1) / spacing).max(0) as usize;
    let mut out = Vec::new();
    for index in 0..count {
        let x = min.x() + (index as Coord).saturating_mul(spacing);
        let mut intersections = line_intersections(polygons, x);
        intersections.sort_by(|left, right| {
            if left.below(right) {
                std::cmp::Ordering::Less
            } else if right.below(left) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        remove_tangential(&mut intersections, polygons);
        let mut i = 0;
        while i + 1 < intersections.len() {
            let a = intersections[i].pos();
            let b = intersections[i + 1].pos();
            append_span_samples(&mut out, x, a, b, spacing);
            i += 2;
        }
    }
    out
}

fn segment_intersection(
    points: &[Point],
    segment: usize,
    contour_index: usize,
    x: Coord,
) -> Option<Intersection> {
    let previous = if segment == 0 {
        points.len() - 1
    } else {
        segment - 1
    };
    let p1 = points[previous];
    let p2 = points[segment];
    let (left, right) = if p1.x() <= p2.x() {
        (p1.x(), p2.x())
    } else {
        (p2.x(), p1.x())
    };
    if !(left <= x && x <= right) {
        return None;
    }
    let (pos_p, mut pos_q) = if p1.x() == x {
        if p2.x() == x {
            // Strictly vertical segments are ignored.
            return None;
        }
        let p0 = points[if previous == 0 {
            points.len() - 1
        } else {
            previous - 1
        }];
        if (p0.x() - p1.x()) * (p2.x() - p1.x()) > 0 {
            // Contour touches the line from one side.
            return None;
        }
        (p1.y(), 1u64)
    } else if p2.x() == x {
        let p3 = points[(segment + 1) % points.len()];
        if (p3.x() - p2.x()) * (p1.x() - p2.x()) > 0 {
            return None;
        }
        (p2.y(), 1u64)
    } else {
        // General position: the rational parameter t in (0, 1) with a
        // positive denominator, then pos = t * (p2.y - p1.y) + p1.y.
        let (numerator, denominator) = if p2.x() > p1.x() {
            (x - p1.x(), p2.x() - p1.x())
        } else {
            (p1.x() - x, p1.x() - p2.x())
        };
        (
            numerator
                .saturating_mul(p2.y() - p1.y())
                .saturating_add(p1.y() * denominator),
            denominator as u64,
        )
    };
    if pos_q == 0 {
        pos_q = 1;
    }
    Some(Intersection {
        pos_p,
        pos_q: pos_q as u32,
        low: p2.x() > p1.x(),
        contour: contour_index,
        segment,
    })
}

fn keep_intersection(
    last: &mut Intersection,
    candidate: Intersection,
    polygons: &[Polygon],
) -> bool {
    let mut take = true;
    if candidate.contour == last.contour && candidate.pos_q == 1 && last.pos_q == 1 {
        let points = polygons[candidate.contour].points();
        let previous = if candidate.segment == 0 {
            points.len() - 1
        } else {
            candidate.segment - 1
        };
        if last.pos_p == candidate.pos_p {
            // Successive same-direction segments meeting on the line.
            let _ = previous;
            take = false;
        } else if candidate.low == last.low {
            // Z shaped path with the center segment on the line.
            if !candidate.low {
                *last = candidate;
            }
            take = false;
        }
    }
    take
}

fn append_span_samples(out: &mut Vec<Point>, x: Coord, a: Coord, b: Coord, spacing: Coord) {
    let mut y = a - (a % spacing) - spacing;
    while y < b {
        if y > a {
            out.push(Point::new(x, y));
        }
        y += spacing;
    }
}
