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
                continue;
            }
            let (mut pos_p, mut pos_q) = if p1.x() == x {
                if p2.x() == x {
                    // Strictly vertical segments are ignored.
                    continue;
                }
                let p0 = points[if previous == 0 {
                    points.len() - 1
                } else {
                    previous - 1
                }];
                if (p0.x() as i64 - p1.x() as i64) * (p2.x() as i64 - p1.x() as i64) > 0 {
                    // Contour touches the line from one side.
                    continue;
                }
                (p1.y() as i64, 1u64)
            } else if p2.x() == x {
                let p3 = points[(segment + 1) % points.len()];
                if (p3.x() as i64 - p2.x() as i64) * (p1.x() as i64 - p2.x() as i64) > 0 {
                    continue;
                }
                (p2.y() as i64, 1u64)
            } else {
                // General position: the rational parameter t in (0, 1) with a
                // positive denominator, then pos = t * (p2.y - p1.y) + p1.y.
                let (numerator, denominator) = if p2.x() > p1.x() {
                    (x as i64 - p1.x() as i64, p2.x() as i64 - p1.x() as i64)
                } else {
                    (p1.x() as i64 - x as i64, p1.x() as i64 - p2.x() as i64)
                };
                (
                    numerator
                        .saturating_mul(p2.y() as i64 - p1.y() as i64)
                        .saturating_add(p1.y() as i64 * denominator),
                    denominator as u64,
                )
            };
            if pos_q == 0 {
                pos_q = 1;
            }
            intersections.push(Intersection {
                pos_p,
                pos_q: pos_q as u32,
                low: p2.x() > p1.x(),
                contour: contour_index,
                segment,
            });
        }
    }
    intersections
}

/// `slice_region_by_vertical_lines` duplicate/tangency removal
/// (FillRectilinear.cpp:851-905) for the single-offset (all outer) case.
fn remove_tangential(intersections: &mut Vec<Intersection>, polygons: &[Polygon]) {
    let mut kept = 0;
    for index in 0..intersections.len() {
        let mut take = true;
        if kept > 0 {
            let candidate = intersections[index];
            let last = intersections[kept - 1];
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
                        intersections[kept - 1] = candidate;
                    }
                    take = false;
                }
            }
        }
        if take {
            intersections[kept] = intersections[index];
            kept += 1;
        }
    }
    intersections.truncate(kept);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diamond(radius: Coord) -> Polygon {
        Polygon::new(vec![
            Point::new(0, radius),
            Point::new(radius, 0),
            Point::new(0, -radius),
            Point::new(-radius, 0),
        ])
    }

    #[test]
    fn center_line_spans_the_diamond() {
        // Rounded diamond (as the rotated square produces after integer
        // rotation) so no vertex lies exactly on the sliced line. CCW like
        // the upstream polygon orientation convention.
        let radius = 5_112_000;
        let diamond = Polygon::new(vec![
            Point::new(3_797, radius),
            Point::new(-radius, 3_797),
            Point::new(-3_797, -radius),
            Point::new(radius, -3_797),
        ]);
        let spans = vertical_spans(&[diamond], -6_000_000, 3_000_000, 5);
        let center = spans
            .iter()
            .find(|span| span.points()[0].x() == 0)
            .expect("center span exists");
        let mut ys = [center.points()[0].y(), center.points()[1].y()];
        ys.sort();
        assert_eq!(ys, [-5_108_209, 5_108_209]);
        assert_eq!(
            spans
                .iter()
                .filter(|span| span.points()[0].x() == 0)
                .count(),
            1
        );
    }
}

#[cfg(test)]
mod mixed_sign_rational_ordering {
    use super::*;

    #[test]
    fn negative_position_sorts_below_positive() {
        // The rounded-diamond center vline: top crossing positive, bottom
        // crossing negative — the mixed-sign case of `operator<`.
        let radius = 5_112_000;
        let diamond = Polygon::new(vec![
            Point::new(3_797, radius),
            Point::new(-radius, 3_797),
            Point::new(-3_797, -radius),
            Point::new(radius, -3_797),
        ]);
        let raw = line_intersections(&[diamond], 0);
        assert_eq!(raw.len(), 2);
        let mut sorted = raw.clone();
        sorted.sort_by(|left, right| {
            if left.below(right) {
                std::cmp::Ordering::Less
            } else if right.below(left) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        // The OUTER_LOW (entering, `low`) crossing sorts first at the
        // geometrically lower position.
        assert!(sorted[0].low);
        assert!(!sorted[1].low);
    }
}
