use super::{
    contour::{
        closed_contour_distance_ccw, closed_contour_distance_cw, path_length_along_contour_ccw,
    },
    types::WorkingGraph,
};

pub(super) struct HookLimits {
    pub(super) anchor: f64,
    pub(super) anchor_max: f64,
    pub(super) multiline_reach: Option<f64>,
    pub(super) epsilon: f64,
}

// FillBase.cpp:1947-1963: look through at most 64 ends, including consumed hooks.
pub(super) fn distance_to_open_end(
    graph: &mut WorkingGraph,
    open_end: &[bool],
    start: usize,
    path_index: usize,
    backwards: bool,
) -> f64 {
    let contour = graph.intersections[start].contour_index.unwrap();
    let length = *graph.boundary[contour].params.last().unwrap();
    let mut distance = 0.0;
    let mut current = start;
    for _ in 0..64 {
        let other = if backwards {
            graph.intersections[current].prev
        } else {
            graph.intersections[current].next
        };
        let Some(other) = other else { break };
        if other == start {
            break;
        }
        let from = graph.intersections[current].param;
        let to = graph.intersections[other].param;
        distance += if backwards {
            closed_contour_distance_cw(from, to, length)
        } else {
            closed_contour_distance_ccw(from, to, length)
        };
        if open_end[other] && graph.root(other / 2) != path_index {
            return distance;
        }
        current = other;
    }
    f64::MAX
}

// FillBase.cpp:2029-2041: stop a multiline hook short of its neighboring copy.
pub(super) fn multiline_hook_length(
    graph: &WorkingGraph,
    start: usize,
    path_index: usize,
    backwards: bool,
    limits: &HookLimits,
) -> f64 {
    let anchor_length = limits.anchor;
    let Some(reach) = limits.multiline_reach else {
        return anchor_length;
    };
    let epsilon = limits.epsilon;
    let intersection = &graph.intersections[start];
    let other = if backwards {
        intersection.prev
    } else {
        intersection.next
    }
    .unwrap();
    if other == start {
        return anchor_length;
    }
    let end = graph.point(start);
    let path = graph.paths[path_index].as_ref().unwrap();
    let mut direction = (0.0_f64, 0.0_f64);
    for i in 1..path.len() {
        if direction.0 * direction.0 + direction.1 * direction.1 >= (2.0 * reach).powi(2) {
            break;
        }
        let point = path[if path[0] == end {
            i
        } else {
            path.len() - 1 - i
        }];
        direction = ((end.x() - point.x()) as f64, (end.y() - point.y()) as f64);
    }
    let copy = graph.point(other);
    let to_copy = ((copy.x() - end.x()) as f64, (copy.y() - end.y()) as f64);
    let norm = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
    let across = if norm > 0.0 {
        ((direction.0 / norm) * to_copy.1 - (direction.1 / norm) * to_copy.0).abs()
    } else {
        0.0
    };
    if across > epsilon && across < reach {
        let contour = intersection.contour_index.unwrap();
        let length = *graph.boundary[contour].params.last().unwrap();
        let gap = if backwards {
            path_length_along_contour_ccw(&graph.intersections[other], intersection, length)
        } else {
            path_length_along_contour_ccw(intersection, &graph.intersections[other], length)
        };
        let copy_norm = (to_copy.0 * to_copy.0 + to_copy.1 * to_copy.1).sqrt();
        anchor_length.min(gap - graph.line_half_width * (1.0 + copy_norm / across))
    } else {
        anchor_length
    }
}
