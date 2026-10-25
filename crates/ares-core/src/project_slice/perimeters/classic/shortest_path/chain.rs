mod reorder;
mod search;

pub(in crate::project_slice) use reorder::chain_and_reorder_extrusion_paths;
#[cfg(test)]
pub(in crate::project_slice) use reorder::reorder_extrusion_paths;

use super::{kd_tree::KdTree, priority_queue::MutablePriorityQueue};
use crate::project_slice::perimeters::classic::{
    chained_loops::ExtrusionLoop, materialize::ExtrusionPath,
};
use search::CandidateSearch;

pub(super) struct EndPoint {
    pub(super) chain_id: usize,
    pub(super) edge_out: Option<usize>,
    pub(super) distance_out: f64,
    pub(super) heap_idx: usize,
}

impl EndPoint {
    fn new() -> Self {
        Self {
            chain_id: 0,
            edge_out: None,
            distance_out: f64::MAX,
            heap_idx: usize::MAX,
        }
    }
}

pub(super) struct EquivalentChains {
    equivalent_with: Vec<usize>,
}

impl EquivalentChains {
    fn new(capacity: usize) -> Self {
        let mut equivalent_with = Vec::with_capacity(capacity + 1);
        equivalent_with.push(0);
        Self { equivalent_with }
    }

    fn next(&mut self) -> usize {
        let id = self.equivalent_with.len();
        self.equivalent_with.push(id);
        id
    }

    fn equivalent(&mut self, mut chain_id: usize) -> usize {
        if chain_id == 0 {
            return 0;
        }
        let original = chain_id;
        loop {
            let lower = self.equivalent_with[chain_id];
            if lower == chain_id {
                self.equivalent_with[original] = lower;
                break;
            }
            chain_id = lower;
        }
        chain_id
    }

    fn merge(&mut self, first: usize, second: usize) -> usize {
        let first_root = self.equivalent(first);
        let second_root = self.equivalent(second);
        let root = first_root.min(second_root);
        self.equivalent_with[first] = root;
        self.equivalent_with[second] = root;
        root
    }
}

pub(in crate::project_slice) fn chain_extrusion_paths(
    paths: &[ExtrusionPath],
    start_near: Option<[crate::geometry::Coord; 2]>,
) -> Vec<(usize, bool)> {
    match paths.len() {
        0 => Vec::new(),
        1 => {
            let reverse = start_near.is_some_and(|start| {
                squared_coord_delta(last_coord(&paths[0]), start)
                    < squared_coord_delta(first_coord(&paths[0]), start)
            });
            vec![(0, reverse)]
        }
        _ => {
            let positions: Vec<_> = paths
                .iter()
                .flat_map(|path| [first_xy(path), last_xy(path)])
                .collect();
            chain_multiple(&positions, start_near.map(coord_to_f64))
        }
    }
}

pub(in crate::project_slice) fn chain_extrusion_loops(
    loops: &[ExtrusionLoop],
) -> Vec<(usize, bool)> {
    let first_points: Vec<_> = loops
        .iter()
        .map(|loop_| {
            let first = &loop_.paths[0].polyline.points[0];
            [first.x as f64, first.y as f64]
        })
        .collect();
    chain_loops_by_first_points(&first_points)
        .into_iter()
        .map(|index| (index, false))
        .collect()
}

/// First-point loop chaining shared by `chain_extrusion_loops` and the
/// classic materialization child-descent order (`PerimeterGenerator.cpp:238`
/// chains the just-materialized level before recursing into children at
/// `:254-255`).
pub(in crate::project_slice) fn chain_loops_by_first_points(
    first_points: &[[f64; 2]],
) -> Vec<usize> {
    match first_points.len() {
        0 => return Vec::new(),
        1 => return vec![0],
        _ => {}
    }
    let positions = first_points
        .iter()
        .flat_map(|point| [*point, *point])
        .collect::<Vec<_>>();
    chain_multiple(&positions, Some([0.0, 0.0]))
        .into_iter()
        .map(|(index, _)| index)
        .collect()
}
pub(super) fn chain_points(points: &[crate::geometry::Point]) -> Vec<usize> {
    if points.len() < 2 {
        return (0..points.len()).collect();
    }
    let positions = points
        .iter()
        .flat_map(|point| {
            let position = [point.x() as f64, point.y() as f64];
            [position, position]
        })
        .collect::<Vec<_>>();
    chain_multiple(&positions, None)
        .into_iter()
        .map(|(index, _)| index)
        .collect()
}

pub(super) type Chain = Vec<(usize, bool)>;
pub(super) fn chain_multiple(points: &[[f64; 2]], start: Option<[f64; 2]>) -> Chain {
    chain_multiple_constrained(points, start, None)
}

fn chain_multiple_constrained(
    positions: &[[f64; 2]],
    start_near: Option<[f64; 2]>,
    can_reverse: Option<&[bool]>,
) -> Vec<(usize, bool)> {
    let tree = KdTree::new(positions);
    let mut endpoints: Vec<_> = positions.iter().map(|_| EndPoint::new()).collect();
    let segment_count = positions.len() / 2;
    let mut equivalents = EquivalentChains::new(segment_count);

    let mut first_point = start_near.map(|start| {
        tree.find_closest(positions, start, |index| {
            index & 1 == 0 || can_reverse.is_none_or(|reversible| reversible[index >> 1])
        })
    });
    let first_point_idx = first_point.unwrap_or(usize::MAX);
    if let Some(index) = first_point {
        endpoints[index].distance_out = 0.0;
        endpoints[index].chain_id = equivalents.next();
    }

    let mut index = 0;
    while index < endpoints.len() {
        if Some(index) != first_point {
            let next = tree.find_closest(positions, positions[index], |candidate| {
                candidate != first_point_idx && (candidate ^ index) > 1
            });
            endpoints[index].edge_out = Some(next);
            endpoints[index].distance_out = squared_distance(positions[index], positions[next]);
        }
        index += 1;
    }

    let mut queue = MutablePriorityQueue::with_capacity(endpoints.len() * 2 - 1);
    let mut index = 0;
    while index < endpoints.len() {
        if Some(index) != first_point {
            queue.push(index, &mut endpoints);
        }
        index += 1;
    }

    let candidate_search = CandidateSearch {
        tree: &tree,
        positions,
    };
    let mut connections_left = segment_count - 1;
    loop {
        let first = queue.top();
        let second = endpoints[first].edge_out.expect("candidate exists");
        let mut valid = endpoints[second].chain_id == 0;
        let mut first_other_chain = 0;
        let mut second_other_chain = 0;
        if valid {
            first_other_chain = equivalents.equivalent(endpoints[first ^ 1].chain_id);
            second_other_chain = equivalents.equivalent(endpoints[second ^ 1].chain_id);
            valid = first_other_chain == 0 || first_other_chain != second_other_chain;
        }
        if !valid {
            candidate_search.update(first, &mut endpoints, &mut equivalents, &mut queue);
            continue;
        }
        let popped = queue.pop(&mut endpoints);
        debug_assert_eq!(popped, first);
        queue.remove(endpoints[second].heap_idx, &mut endpoints);
        endpoints[second].edge_out = Some(first);
        endpoints[second].distance_out = endpoints[first].distance_out;
        let chain_id = merged_chain_id(first_other_chain, second_other_chain, &mut equivalents);
        endpoints[first].chain_id = chain_id;
        endpoints[second].chain_id = chain_id;
        connections_left -= 1;
        if connections_left != 0 {
            continue;
        }
        if first_point.is_none() {
            let start = queue.pop(&mut endpoints);
            endpoints[start].edge_out = None;
            first_point = Some(start);
        }
        let last = queue.pop(&mut endpoints);
        endpoints[last].edge_out = None;
        debug_assert!(queue.is_empty());
        break;
    }

    let mut output = Vec::with_capacity(segment_count);
    let mut current = first_point;
    while let Some(first) = current {
        output.push((first >> 1, first & 1 != 0));
        current = endpoints[first ^ 1].edge_out;
    }
    if let Some(reversible) = can_reverse
        && output
            .iter()
            .any(|&(segment, reverse)| reverse && !reversible[segment])
    {
        if start_near.is_none() {
            let opposite = output
                .iter()
                .rev()
                .map(|&(segment, reverse)| (segment, !reverse))
                .collect::<Vec<_>>();
            if opposite
                .iter()
                .all(|&(segment, reverse)| !reverse || reversible[segment])
            {
                return opposite;
            }
        }
        let initial = if start_near.is_some() {
            first_point.unwrap()
        } else {
            0
        };
        return chain_closest_point(positions, initial, reversible);
    }
    output
}

fn chain_closest_point(
    positions: &[[f64; 2]],
    first: usize,
    can_reverse: &[bool],
) -> Vec<(usize, bool)> {
    let tree = KdTree::new(positions);
    let mut visited = vec![false; positions.len() / 2];
    let mut output = Vec::with_capacity(visited.len());
    let mut endpoint = first;
    loop {
        let segment = endpoint >> 1;
        visited[segment] = true;
        output.push((segment, endpoint & 1 != 0));
        if output.len() == visited.len() {
            break;
        }
        let cursor = positions[endpoint ^ 1];
        endpoint = tree.find_closest(positions, cursor, |candidate| {
            let segment = candidate >> 1;
            !visited[segment] && (candidate & 1 == 0 || can_reverse[segment])
        });
    }
    output
}

pub(in crate::project_slice) fn chain_segments_constrained(
    endpoints: &[[[crate::geometry::Coord; 2]; 2]],
    start_near: [crate::geometry::Coord; 2],
    can_reverse: &[bool],
) -> Vec<(usize, bool)> {
    debug_assert_eq!(endpoints.len(), can_reverse.len());
    chain_segments_constrained_with_optional_start(endpoints, Some(start_near), can_reverse)
}

pub(in crate::project_slice) fn chain_segments_constrained_with_optional_start(
    endpoints: &[[[crate::geometry::Coord; 2]; 2]],
    start_near: Option<[crate::geometry::Coord; 2]>,
    can_reverse: &[bool],
) -> Vec<(usize, bool)> {
    match endpoints.len() {
        0 => Vec::new(),
        1 => vec![(
            0,
            can_reverse[0]
                && start_near.is_some_and(|start| {
                    squared_coord_delta(endpoints[0][1], start)
                        < squared_coord_delta(endpoints[0][0], start)
                }),
        )],
        _ => {
            let positions = endpoints
                .iter()
                .flat_map(|endpoints| endpoints.map(coord_to_f64))
                .collect::<Vec<_>>();
            chain_multiple_constrained(&positions, start_near.map(coord_to_f64), Some(can_reverse))
        }
    }
}

fn merged_chain_id(first: usize, second: usize, equivalents: &mut EquivalentChains) -> usize {
    match (first, second) {
        (0, 0) => equivalents.next(),
        (0, second) => second,
        (first, 0) => first,
        (first, second) if first == second => first,
        (first, second) => equivalents.merge(first, second),
    }
}

/// `chain_extrusion_entities` (ShortestPath.cpp:1026) — nearest-neighbor chain
/// over a MIXED loop+path entity set. `endpoints[i]` is `[first,last]` in coord
/// units and `is_loop[i]` marks loops (whose reversal is forced `false` — start
/// == end makes it a no-op upstream, :1036). Thin-wall `ExtrusionPath`s may
/// reverse (`could_reverse = is_loop || can_reverse()`); the traversal treats
/// every entity as `can_reverse` here.
pub(in crate::project_slice) fn chain_extrusion_entities(
    endpoints: &[[[crate::geometry::Coord; 2]; 2]],
    is_loop: &[bool],
    start_near: [crate::geometry::Coord; 2],
) -> Vec<(usize, bool)> {
    let can_reverse: Vec<bool> = vec![true; endpoints.len()];
    let mut chain = chain_segments_constrained(endpoints, start_near, &can_reverse);
    for (index, reverse) in &mut chain {
        if is_loop[*index] {
            // Loops' start == end, so reversal is ignored upstream (:1036).
            *reverse = false;
        }
    }
    chain
}

mod coords;
use coords::{coord_to_f64, first_coord, first_xy, last_coord, last_xy, squared_coord_delta};

pub(super) fn squared_distance(first: [f64; 2], second: [f64; 2]) -> f64 {
    let dx = first[0] - second[0];
    let dy = first[1] - second[1];
    dx * dx + dy * dy
}
