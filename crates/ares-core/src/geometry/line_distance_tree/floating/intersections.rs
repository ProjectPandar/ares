//! Intersection queries over the native tree: `AABBTreeLines.hpp:105-160`
//! (left-first recursive walk into AABBs intersecting the query line's
//! box, leaf `line_alg::intersection` at native scalar precision) and
//! `:265-296` (`sorted=true`: ascending squared distance from the query
//! line start).

use super::{Bounds, FloatLineDistanceTree, NativeFloat, NodeKind};

/// `intersections_with_line<true>`: tree-order collection then sort by
/// `(point - line.a).squaredNorm()` ascending
/// (`Floating = LineType::Scalar` for the float tree).
pub(super) fn sorted<F: NativeFloat>(
    tree: &FloatLineDistanceTree<F>,
    line: [[F; 2]; 2],
) -> Vec<([F; 2], usize)> {
    if tree.nodes.is_empty() {
        return Vec::new();
    }
    let line_bounds = bounds_over(line);
    let mut hits = Vec::new();
    visit(tree, 0, line, line_bounds, &mut hits);
    hits.sort_by(|left, right| {
        squared_distance_from_start(line, left.0)
            .partial_cmp(&squared_distance_from_start(line, right.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits
}

fn visit<F: NativeFloat>(
    tree: &FloatLineDistanceTree<F>,
    index: usize,
    line: [[F; 2]; 2],
    line_bounds: Bounds<F>,
    hits: &mut Vec<([F; 2], usize)>,
) {
    let node = tree.nodes[index];
    match node.kind {
        NodeKind::Unused => unreachable!("traversal reaches only valid native tree nodes"),
        NodeKind::Leaf(line_index) => {
            if let Some(point) = segment_intersection(line, tree.lines[line_index]) {
                hits.push((point, line_index));
            }
        }
        NodeKind::Inner => {
            // `Eigen::AlignedBox::intersects` is inclusive on every axis
            // (`min <= other.max && max >= other.min`).
            for child in [index * 2 + 1, index * 2 + 2] {
                if intersects(tree.nodes[child].bounds, line_bounds) {
                    visit(tree, child, line, line_bounds, hits);
                }
            }
        }
    }
}

fn bounds_over<F: NativeFloat>(line: [[F; 2]; 2]) -> Bounds<F> {
    Bounds {
        min: [line[0][0].min(line[1][0]), line[0][1].min(line[1][1])],
        max: [line[0][0].max(line[1][0]), line[0][1].max(line[1][1])],
    }
}

fn intersects<F: NativeFloat>(left: Bounds<F>, right: Bounds<F>) -> bool {
    (0..2).all(|axis| left.min[axis] <= right.max[axis] && left.max[axis] >= right.min[axis])
}

fn squared_distance_from_start<F: NativeFloat>(line: [[F; 2]; 2], point: [F; 2]) -> F {
    let (dx, dy) = (point[0] - line[0][0], point[1] - line[0][1]);
    dx * dx + dy * dy
}

/// `line_alg::intersection` (`Line.hpp:123-149`): native-precision cross
/// products, `fabs(denom) < EPSILON` rejects near-parallel lines (the
/// float `fabs` result promotes to the double `EPSILON` 1e-4), and both
/// segment parameters must fall in `[0, 1]`.
fn segment_intersection<F: NativeFloat>(l1: [[F; 2]; 2], l2: [[F; 2]; 2]) -> Option<[F; 2]> {
    let v1 = [l1[1][0] - l1[0][0], l1[1][1] - l1[0][1]];
    let v2 = [l2[1][0] - l2[0][0], l2[1][1] - l2[0][1]];
    let denom = cross(v1, v2);
    if denom.into().abs() < 1e-4 {
        return None;
    }
    let v12 = [l1[0][0] - l2[0][0], l1[0][1] - l2[0][1]];
    let t1 = cross(v2, v12) / denom;
    let t2 = cross(v1, v12) / denom;
    let one = F::from_f64(1.0);
    let zero = F::ZERO;
    if t1 >= zero && t1 <= one && t2 >= zero && t2 <= one {
        Some([l1[0][0] + t1 * v1[0], l1[0][1] + t1 * v1[1]])
    } else {
        None
    }
}

fn cross<F: NativeFloat>(a: [F; 2], b: [F; 2]) -> F {
    a[0] * b[1] - a[1] * b[0]
}
