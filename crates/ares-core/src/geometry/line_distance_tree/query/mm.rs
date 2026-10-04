//! Unscaled-millimetre nearest queries mirroring upstream
//! `distance_from_lines_extra` on `LinesDistancer<Linef>` —
//! `Linef` stores unscaled `Vec2d` endpoints, so distance is measured
//! in f64 over `coord * scale_factor` endpoints (`AABBTreeLines.hpp:329-346`,
//! `Line.hpp:43-70`).

use crate::geometry::Line;

use super::super::MmQuery;
use super::{Bounds, Node, NodeKind};

#[derive(Clone, Copy)]
pub(crate) struct NearestMm {
    pub(crate) squared_distance: f64,
}

pub(crate) fn nearest_mm(
    lines: &[Line],
    nodes: &[Node],
    origin: [f64; 2],
    scale_factor: f64,
) -> Option<NearestMm> {
    if nodes.is_empty() {
        return None;
    }
    let mut winner = None;
    visit_mm(
        lines,
        nodes,
        0,
        MmQuery {
            point: origin,
            scale: scale_factor,
        },
        &mut winner,
    );
    winner
}

fn best(winner: Option<NearestMm>) -> f64 {
    winner.map_or(f64::INFINITY, |winner| winner.squared_distance)
}

fn visit_mm(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    query: MmQuery,
    winner: &mut Option<NearestMm>,
) {
    let MmQuery {
        point: origin,
        scale: scale_factor,
    } = query;
    let node = nodes[node_index];
    match node.kind {
        NodeKind::Unused => unreachable!("tree traversal only reaches valid nodes"),
        NodeKind::Leaf(line_index) => {
            let (squared_distance, _) =
                segment_distance_mm(lines[line_index], origin, scale_factor);
            if squared_distance < best(*winner) {
                *winner = Some(NearestMm { squared_distance });
            }
        }
        NodeKind::Inner => {
            let left_index = node_index * 2 + 1;
            let right_index = left_index + 1;
            let left = nodes[left_index];
            let right = nodes[right_index];
            let left_distance = exterior_mm(left.bounds, origin, scale_factor);
            let right_distance = exterior_mm(right.bounds, origin, scale_factor);
            let (first, second) = if left_distance < right_distance {
                ((left_index, left_distance), (right_index, right_distance))
            } else {
                ((right_index, right_distance), (left_index, left_distance))
            };
            if first.1 < best(*winner) {
                visit_mm(lines, nodes, first.0, query, winner);
            }
            if second.1 < best(*winner) {
                visit_mm(lines, nodes, second.0, query, winner);
            }
        }
    }
}

fn unscaled(coordinate: i64, scale_factor: f64) -> f64 {
    coordinate as f64 * scale_factor
}

fn exterior_mm(bounds: Bounds, origin: [f64; 2], scale_factor: f64) -> f64 {
    let delta = |minimum: i64, maximum: i64, value: f64| {
        let minimum = unscaled(minimum, scale_factor);
        let maximum = unscaled(maximum, scale_factor);
        if minimum > value {
            minimum - value
        } else if maximum < value {
            value - maximum
        } else {
            0.0
        }
    };
    let x = delta(bounds.min.x(), bounds.max.x(), origin[0]);
    let y = delta(bounds.min.y(), bounds.max.y(), origin[1]);
    x * x + y * y
}

/// `line_alg::distance_to_squared` (`Line.hpp:43-70`): f64 projection,
/// `t = va·v / v²` clamped to the segment; the returned point is the
/// f64 nearest position.
fn segment_distance_mm(line: Line, origin: [f64; 2], scale_factor: f64) -> (f64, [f64; 2]) {
    let a = [
        unscaled(line.a.x(), scale_factor),
        unscaled(line.a.y(), scale_factor),
    ];
    let b = [
        unscaled(line.b.x(), scale_factor),
        unscaled(line.b.y(), scale_factor),
    ];
    super::super::segment::distance_squared(a, b, origin)
}
