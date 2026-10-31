//! Point specialization of ShortestPath.cpp:182–434 greedy fragments, including
//! the optional fixed start. OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

use super::{
    EndPoint, EquivalentChains, NO_INDEX, Point, kd_tree::KdTree, priority_queue::EndPointHeap,
    squared_distance,
};

pub(super) fn chain(points: &[Point], start: Option<Point>) -> Vec<usize> {
    let mut endpoints = points
        .iter()
        .flat_map(|&point| [EndPoint::new(point), EndPoint::new(point)])
        .collect::<Vec<_>>();
    let positions = endpoints
        .iter()
        .map(|endpoint| endpoint.position)
        .collect::<Vec<_>>();
    let tree = KdTree::new(&positions);
    let mut chains = EquivalentChains::new(points.len());
    let first = start.map_or(NO_INDEX, |point| {
        let first = tree
            .closest(&positions, [point.x() as f64, point.y() as f64], |_| true)
            .expect("nonempty point chain has a nearest start");
        endpoints[first].distance_out = 0.0;
        endpoints[first].chain_id = chains.next();
        first
    });
    initialize_links(&mut endpoints, &positions, &tree, first);
    let mut queue = EndPointHeap::with_capacity(endpoints.len());
    for index in 0..endpoints.len() {
        if index != first {
            queue.push(index, &mut endpoints);
        }
    }
    let first = connect(
        &mut endpoints,
        (&tree, &positions),
        (&mut queue, &mut chains),
        first,
    );
    let mut order = Vec::with_capacity(points.len());
    let mut current = first;
    while current != NO_INDEX {
        order.push(current >> 1);
        current = endpoints[current ^ 1].edge_out;
    }
    order
}

fn initialize_links(
    endpoints: &mut [EndPoint],
    positions: &[[f64; 2]],
    tree: &KdTree,
    first: usize,
) {
    for index in 0..endpoints.len() {
        if index == first {
            continue;
        }
        let closest = tree
            .closest(positions, endpoints[index].position, |candidate| {
                candidate != first && (candidate ^ index) > 1
            })
            .expect("each point has a peer outside its own segment and fixed start");
        endpoints[index].edge_out = closest;
        endpoints[index].distance_out =
            squared_distance(endpoints[index].position, endpoints[closest].position);
    }
}

fn connect(
    endpoints: &mut [EndPoint],
    (tree, positions): (&KdTree, &[[f64; 2]]),
    (queue, chains): (&mut EndPointHeap, &mut EquivalentChains),
    first: usize,
) -> usize {
    let mut remaining = endpoints.len() / 2 - 1;
    loop {
        let point1 = queue.top();
        let point2 = endpoints[point1].edge_out;
        let chain1 = chains.find(endpoints[point1 ^ 1].chain_id);
        let chain2 = chains.find(endpoints[point2 ^ 1].chain_id);
        let valid = endpoints[point2].chain_id == 0 && (chain1 == 0 || chain1 != chain2);
        if valid {
            queue.pop(endpoints);
            queue.remove(endpoints[point2].heap_index, endpoints);
            endpoints[point1].edge_out = point2;
            endpoints[point2].edge_out = point1;
            endpoints[point2].distance_out = endpoints[point1].distance_out;
            let chain = match (chain1, chain2) {
                (0, 0) => chains.next(),
                (0, chain) | (chain, 0) => chain,
                (left, right) if left == right => left,
                (left, right) => chains.merge(left, right),
            };
            endpoints[point1].chain_id = chain;
            endpoints[point2].chain_id = chain;
            remaining -= 1;
            if remaining == 0 {
                return finish(endpoints, queue, first);
            }
        } else {
            let closest = tree
                .closest(positions, endpoints[point1].position, |candidate| {
                    (candidate ^ point1) > 1
                        && endpoints[candidate].chain_id == 0
                        && chains.can_connect(
                            endpoints[point1 ^ 1].chain_id,
                            endpoints[candidate ^ 1].chain_id,
                        )
                })
                .expect("an open chain endpoint has a valid peer");
            endpoints[point1].edge_out = closest;
            endpoints[point1].distance_out =
                squared_distance(endpoints[point1].position, endpoints[closest].position);
            queue.update(endpoints[point1].heap_index, endpoints);
        }
    }
}

// ShortestPath.cpp:390–405 final free/fixed endpoints, in source pop order.
fn finish(endpoints: &mut [EndPoint], queue: &mut EndPointHeap, mut first: usize) -> usize {
    if first == NO_INDEX {
        first = queue.top();
        queue.pop(endpoints);
        endpoints[first].edge_out = NO_INDEX;
    }
    let last = queue.top();
    endpoints[last].edge_out = NO_INDEX;
    queue.pop(endpoints);
    first
}
