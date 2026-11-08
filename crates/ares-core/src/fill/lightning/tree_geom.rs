//! `FillLightning::Node` convertToPolylines + geometry helpers
//! (`TreeNode.cpp`), split from `tree_node.rs` for the <400 LOC rule.

use crate::geometry::point_in_polygon;
use crate::geometry::{Coord, EdgeGrid, Point, Polygon, Polyline};

use super::distance_field::point_hash;
use super::tree_node::{Node, NodeSPtr};
use crate::fill::rectilinear::rng::Mt19937_64;

impl Node {
    /// `convertToPolylines` — depth-first branch order, the first child picked by
    /// a generator seeded from this tree's root, other children get new polylines
    /// (TreeNode.cpp:358-369,372-388).
    pub(super) fn convert_to_polylines(this: &NodeSPtr, line_overlap: Coord) -> Vec<Polyline> {
        let mut result: Vec<Polyline> = vec![Polyline::new(Vec::new())];
        let mut rng = Mt19937_64::with_seed(point_hash(this.borrow().p));
        Self::convert_rec(this, 0, &mut result, &mut rng);
        Self::remove_junction_overlap(&mut result, line_overlap);
        result
    }

    fn convert_rec(
        this: &NodeSPtr,
        long_line_idx: usize,
        output: &mut Vec<Polyline>,
        rng: &mut Mt19937_64,
    ) {
        let (children, p) = {
            let n = this.borrow();
            (n.children.clone(), n.p)
        };
        if children.is_empty() {
            output[long_line_idx].points_mut().push(p);
            return;
        }
        let first_child_idx = (rng.next() as usize) % children.len();
        Self::convert_rec(&children[first_child_idx], long_line_idx, output, rng);
        output[long_line_idx].points_mut().push(p);
        for idx_offset in 1..children.len() {
            let child_idx = (first_child_idx + idx_offset) % children.len();
            let child = children[child_idx].clone();
            output.push(Polyline::new(Vec::new()));
            let child_line_idx = output.len() - 1;
            Self::convert_rec(&child, child_line_idx, output, rng);
            output[child_line_idx].points_mut().push(p);
        }
    }

    /// `removeJunctionOverlap` — reduce each polyline's tail by `line_overlap`.
    fn remove_junction_overlap(result_lines: &mut Vec<Polyline>, line_overlap: Coord) {
        let reduction = line_overlap;
        let mut i = 0;
        while i < result_lines.len() {
            if result_lines[i].points().len() <= 1 {
                result_lines.swap_remove(i);
                continue;
            }
            let new_pts = trim_junction_tail(&result_lines[i], reduction);
            if new_pts.len() <= 1 {
                result_lines.swap_remove(i);
            } else {
                result_lines[i] = Polyline::new(new_pts);
                i += 1;
            }
        }
    }
}

fn trim_junction_tail(polyline: &Polyline, reduction: Coord) -> Vec<Point> {
    let mut to_be_reduced = reduction;
    let pts = polyline.points().to_vec();
    let mut a = *pts.last().unwrap();
    let mut new_pts = pts.clone();
    let mut idx = pts.len() as i64 - 2;
    while idx >= 0 {
        let b = pts[idx as usize];
        let ab_len = dist(a, b);
        if ab_len >= to_be_reduced {
            // move the tail point toward `b` by the remaining reduction
            let (ax, ay) = (a.x() as f64, a.y() as f64);
            let (bx, by) = (b.x() as f64, b.y() as f64);
            let (dx, dy) = (bx - ax, by - ay);
            let t = to_be_reduced as f64 / ab_len as f64;
            *new_pts.last_mut().unwrap() =
                Point::new(a.x() + (dx * t) as Coord, a.y() + (dy * t) as Coord);
            break;
        } else {
            to_be_reduced -= ab_len;
            new_pts.pop();
        }
        a = b;
        idx -= 1;
    }
    new_pts
}

/// `lineSegmentPolygonsIntersection` (TreeNode.cpp) — walk the outline locator
/// along `a→b`, preserving the source visitor's stored-point distance/update
/// sequence and final `within_max_dist` gate.
pub(super) fn line_segment_polygons_intersection(
    a: Point,
    b: Point,
    outline_locator: &EdgeGrid,
    within_max_dist: Coord,
) -> Option<Point> {
    let mut d2min = f64::MAX;
    let mut result = (0.0, 0.0);
    let _ = outline_locator.visit_cells_intersecting_line(a, b, |_row, _col, edges| {
        for &edge in edges {
            let (s1, s2) = outline_locator.segment(edge);
            // TreeNode.cpp supplies second endpoints as v1/v2, not differences.
            let Some(ip) = seg_seg_intersection(
                (s1.x() as f64, s1.y() as f64),
                (s2.x() as f64, s2.y() as f64),
                (a.x() as f64, a.y() as f64),
                (b.x() as f64, b.y() as f64),
            ) else {
                continue;
            };
            // TreeNode.cpp:166 tests the previous intersection, then saves ip.
            let d = (result.0 - b.x() as f64).powi(2) + (result.1 - b.y() as f64).powi(2);
            if d < d2min {
                d2min = d;
                result = ip;
            }
        }
        true
    });
    if d2min < (within_max_dist as f64) * (within_max_dist as f64) {
        let (x, y) = result;
        Some(Point::new(x.round() as Coord, y.round() as Coord))
    } else {
        None
    }
}

/// `Geometry::segment_segment_intersection` (Geometry.hpp:92) — f64 segments.
fn seg_seg_intersection(
    p1: (f64, f64),
    v1: (f64, f64),
    p2: (f64, f64),
    v2: (f64, f64),
) -> Option<(f64, f64)> {
    let mut denom = v1.0 * v2.1 - v2.0 * v1.1;
    if denom.abs() < 1e-4 {
        return None;
    }
    let s12 = (p1.0 - p2.0, p1.1 - p2.1);
    let mut s_numer = v1.0 * s12.1 - v1.1 * s12.0;
    let denom_pos = denom < 0.0;
    if denom_pos {
        denom = -denom;
        s_numer = -s_numer;
    }
    if s_numer < 0.0 {
        return None;
    }
    let mut t_numer = v2.0 * s12.1 - v2.1 * s12.0;
    if !denom_pos {
        t_numer = -t_numer;
    }
    if t_numer < 0.0 || s_numer > denom || t_numer > denom {
        return None;
    }
    let t = t_numer / denom;
    Some((p1.0 + t * v1.0, p1.1 + t * v1.1))
}

pub(super) fn dist(a: Point, b: Point) -> Coord {
    (((a.x() - b.x()) as f64).powi(2) + ((a.y() - b.y()) as f64).powi(2)).sqrt() as Coord
}

/// `inside` (TreeNode.cpp) — `PointInPolygon` parity over all polys, odd count.
pub(super) fn inside(polygons: &[Polygon], p: Point) -> bool {
    let mut count = 0;
    for poly in polygons {
        let r = point_in_polygon(p, poly.points());
        if r == -1 {
            return true;
        }
        count += r;
    }
    count % 2 == 1
}
