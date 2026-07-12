use std::cmp::Ordering;

use crate::Point2;

pub(super) fn transform_contour(
    points: &[Point2],
    u: Vector2,
    v: Vector2,
) -> Vec<TransformedPoint> {
    points
        .iter()
        .map(|point| TransformedPoint {
            s: dot(*point, v),
            t: dot(*point, u),
        })
        .collect()
}

pub(super) fn clip_contours(
    contours: &[Vec<TransformedPoint>],
    basis: ScanlineBasis,
    spacing: f64,
    scanline_shift: f64,
    normalize_segments: bool,
) -> Vec<InfillCandidate> {
    let bounds = scanline_bounds(contours);

    let mut candidates = Vec::new();
    let shift = if scanline_shift == 0.0 {
        0.0
    } else {
        scanline_shift % spacing
    };
    let mut scanline = (bounds.min_s / spacing).floor() * spacing + spacing / 2.0 + shift;
    while scanline < bounds.min_s {
        scanline += spacing;
    }
    while scanline < bounds.max_s {
        let mut intersections = contours
            .iter()
            .flat_map(|contour| edge_intersections(contour, scanline))
            .collect::<Vec<_>>();
        intersections.sort_by(f64::total_cmp);
        for pair in intersections.chunks_exact(2) {
            let start = to_point(scanline, pair[0], basis);
            let end = to_point(scanline, pair[1], basis);
            let (start, end) = if normalize_segments {
                ordered_segment(start, end)
            } else {
                (start, end)
            };
            if start != end {
                candidates.push(InfillCandidate {
                    scanline,
                    start,
                    end,
                });
            }
        }
        scanline += spacing;
    }
    candidates
}

pub(super) fn scanline_bounds(contours: &[Vec<TransformedPoint>]) -> ScanlineBounds {
    let min_s = contours
        .iter()
        .flatten()
        .map(|point| point.s)
        .fold(f64::INFINITY, f64::min);
    let max_s = contours
        .iter()
        .flatten()
        .map(|point| point.s)
        .fold(f64::NEG_INFINITY, f64::max);
    ScanlineBounds { min_s, max_s }
}

fn edge_intersections(points: &[TransformedPoint], scanline: f64) -> Vec<f64> {
    let mut intersections = Vec::new();
    for (a, b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        if (a.s <= scanline && b.s > scanline) || (b.s <= scanline && a.s > scanline) {
            let ratio = (scanline - a.s) / (b.s - a.s);
            intersections.push(a.t + ratio * (b.t - a.t));
        }
    }
    intersections
}

fn dot(point: Point2, vector: Vector2) -> f64 {
    point.x() * vector.x + point.y() * vector.y
}

fn to_point(s: f64, t: f64, basis: ScanlineBasis) -> Point2 {
    Point2::new(s * basis.v.x + t * basis.u.x, s * basis.v.y + t * basis.u.y)
}

pub(super) fn compare_candidates(a: &InfillCandidate, b: &InfillCandidate) -> Ordering {
    a.scanline
        .total_cmp(&b.scanline)
        .then_with(|| compare_points(a.start, b.start))
        .then_with(|| compare_points(a.end, b.end))
}

pub(super) fn compare_points(a: Point2, b: Point2) -> Ordering {
    a.x()
        .total_cmp(&b.x())
        .then_with(|| a.y().total_cmp(&b.y()))
}

fn ordered_segment(a: Point2, b: Point2) -> (Point2, Point2) {
    if compare_points(a, b).is_gt() {
        (b, a)
    } else {
        (a, b)
    }
}

pub(super) fn anchored_segment(start: Point2, end: Point2, anchor_length: f64) -> (Point2, Point2) {
    if anchor_length == 0.0 {
        return (start, end);
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let length = (dx * dx + dy * dy).sqrt();
    if length == 0.0 {
        return (start, end);
    }
    let ux = dx / length;
    let uy = dy / length;
    (
        Point2::new(
            start.x() - ux * anchor_length,
            start.y() - uy * anchor_length,
        ),
        Point2::new(end.x() + ux * anchor_length, end.y() + uy * anchor_length),
    )
}

#[derive(Clone, Copy)]
pub(super) struct Vector2 {
    x: f64,
    y: f64,
}

impl Vector2 {
    pub(super) const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub(super) fn translate_point(self, point: Point2, distance: f64) -> Point2 {
        Point2::new(point.x() + self.x * distance, point.y() + self.y * distance)
    }
}

#[derive(Clone, Copy)]
pub(super) struct ScanlineBasis {
    u: Vector2,
    v: Vector2,
}

impl ScanlineBasis {
    pub(super) const fn new(u: Vector2, v: Vector2) -> Self {
        Self { u, v }
    }
}

#[derive(Clone, Copy)]
pub(super) struct TransformedPoint {
    s: f64,
    t: f64,
}

pub(super) struct InfillCandidate {
    pub(super) scanline: f64,
    pub(super) start: Point2,
    pub(super) end: Point2,
}

impl InfillCandidate {
    pub(super) fn translated(&self, normal: Vector2, distance: f64) -> Self {
        Self {
            scanline: self.scanline + distance,
            start: normal.translate_point(self.start, distance),
            end: normal.translate_point(self.end, distance),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ScanlineBounds {
    min_s: f64,
    max_s: f64,
}

impl ScanlineBounds {
    pub(super) fn contains(self, scanline: f64) -> bool {
        scanline > self.min_s && scanline < self.max_s
    }
}
