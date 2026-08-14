use super::{kd_tree::KdTree, priority_queue::MutablePriorityQueue};
use crate::project_slice::perimeters::classic::{
    chained_loops::ExtrusionLoop, materialize::ExtrusionPath,
};

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

struct EquivalentChains {
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
    if loops.is_empty() {
        return Vec::new();
    }
    let positions: Vec<_> = loops
        .iter()
        .flat_map(|loop_| {
            let first = loop_
                .paths
                .first()
                .and_then(|path| path.polyline.points.first())
                .expect("O8 loops are nonempty");
            let last = loop_
                .paths
                .last()
                .and_then(|path| path.polyline.points.last())
                .expect("O8 loops are nonempty");
            debug_assert_eq!([first.x, first.y], [last.x, last.y]);
            let point = [first.x as f64, first.y as f64];
            [point, point]
        })
        .collect();
    let chain = if loops.len() == 1 {
        vec![(0, false)]
    } else {
        chain_multiple(&positions, Some([0.0, 0.0]))
    };
    chain.into_iter().map(|(index, _)| (index, false)).collect()
}

fn chain_multiple(positions: &[[f64; 2]], start_near: Option<[f64; 2]>) -> Vec<(usize, bool)> {
    chain_multiple_constrained(positions, start_near, None)
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
        return chain_closest_point(positions, first_point.expect("explicit cursor"), reversible);
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
    match endpoints.len() {
        0 => Vec::new(),
        1 => vec![(
            0,
            can_reverse[0]
                && squared_coord_delta(endpoints[0][1], start_near)
                    < squared_coord_delta(endpoints[0][0], start_near),
        )],
        _ => {
            let positions = endpoints
                .iter()
                .flat_map(|endpoints| endpoints.map(coord_to_f64))
                .collect::<Vec<_>>();
            chain_multiple_constrained(
                &positions,
                Some(coord_to_f64(start_near)),
                Some(can_reverse),
            )
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

struct CandidateSearch<'a> {
    tree: &'a KdTree,
    positions: &'a [[f64; 2]],
}

impl CandidateSearch<'_> {
    fn update(
        &self,
        first: usize,
        endpoints: &mut [EndPoint],
        equivalents: &mut EquivalentChains,
        queue: &mut MutablePriorityQueue,
    ) {
        endpoints[first].edge_out = None;
        let next = self
            .tree
            .find_closest(self.positions, self.positions[first], |candidate| {
                if (candidate ^ first) <= 1 || endpoints[candidate].chain_id != 0 {
                    return false;
                }
                let chain1 = equivalents.equivalent(endpoints[first ^ 1].chain_id);
                let chain2 = equivalents.equivalent(endpoints[candidate ^ 1].chain_id);
                chain1 == 0 || chain1 != chain2
            });
        endpoints[first].edge_out = Some(next);
        endpoints[first].distance_out =
            squared_distance(self.positions[first], self.positions[next]);
        queue.update(endpoints[first].heap_idx, endpoints);
    }
}

pub(in crate::project_slice) fn reorder_extrusion_paths(
    paths: &mut Vec<ExtrusionPath>,
    chain: &[(usize, bool)],
) {
    debug_assert_eq!(paths.len(), chain.len());
    if paths.is_empty() {
        return;
    }
    let mut source: Vec<_> = std::mem::take(paths).into_iter().map(Some).collect();
    paths.reserve(chain.len());
    for &(index, reverse) in chain {
        let mut path = source[index].take().expect("chain indices are unique");
        if reverse {
            path.reverse();
        }
        paths.push(path);
    }
}

pub(in crate::project_slice) fn chain_and_reorder_extrusion_paths(
    paths: &mut Vec<ExtrusionPath>,
    start_near: [crate::geometry::Coord; 2],
) {
    let chain = chain_extrusion_paths(paths, Some(start_near));
    reorder_extrusion_paths(paths, &chain);
}

fn coord_to_f64(point: [crate::geometry::Coord; 2]) -> [f64; 2] {
    [point[0] as f64, point[1] as f64]
}

fn first_coord(path: &ExtrusionPath) -> [crate::geometry::Coord; 2] {
    let point = path.polyline.points.first().expect("O7 paths are valid");
    [point.x, point.y]
}

fn last_coord(path: &ExtrusionPath) -> [crate::geometry::Coord; 2] {
    let point = path.polyline.points.last().expect("O7 paths are valid");
    [point.x, point.y]
}

fn first_xy(path: &ExtrusionPath) -> [f64; 2] {
    coord_to_f64(first_coord(path))
}

fn last_xy(path: &ExtrusionPath) -> [f64; 2] {
    coord_to_f64(last_coord(path))
}

fn squared_coord_delta(
    first: [crate::geometry::Coord; 2],
    second: [crate::geometry::Coord; 2],
) -> f64 {
    let dx = (first[0] - second[0]) as f64;
    let dy = (first[1] - second[1]) as f64;
    dx * dx + dy * dy
}

fn squared_distance(first: [f64; 2], second: [f64; 2]) -> f64 {
    let dx = first[0] - second[0];
    let dy = first[1] - second[1];
    dx * dx + dy * dy
}
