mod queue;
mod types;

use super::super::kd_tree::KdTree;
use queue::Queue;
use types::{Chains, EndPoint, distance};

const SCALED_EPSILON: f64 = 1.0e-4;

// OrcaSlicer 2.4.2 `chain_segments_greedy_constrained_reversals2_`, reached by
// `chain_segments_greedy2` for freely reversible open polylines.
pub(super) fn chain(positions: &[[f64; 2]], start_near: Option<[f64; 2]>) -> Vec<(usize, bool)> {
    let segment_count = positions.len() / 2;
    match segment_count {
        0 => return Vec::new(),
        1 => {
            let reverse = start_near.is_some_and(|start| {
                squared_distance(positions[1], start) < squared_distance(positions[0], start)
            });
            return vec![(0, reverse)];
        }
        _ => {}
    }

    let tree = KdTree::new(positions);
    let mut endpoints = vec![EndPoint::new(); positions.len()];
    let mut chains = Chains::new(segment_count);
    let mut first_point = start_near.map(|start| tree.find_closest(positions, start, |_| true));
    let first_index = first_point.unwrap_or(usize::MAX);
    if let Some(first) = first_point {
        endpoints[first].distance = 0.0;
        endpoints[first].chain_id = chains.next_id();
    }
    initialize_candidates(&tree, positions, first_point, first_index, &mut endpoints);
    let mut queue = initialize_queue(first_point, &mut endpoints);
    let mut connections_left = segment_count - 1;
    let mut iterations_left = segment_count * 16;

    while connections_left > 0 && iterations_left > 0 {
        iterations_left -= 1;
        let first = queue.top();
        let second = endpoints[first].candidate.expect("candidate exists");
        let first_flips = endpoints[first].chain_id > 0;
        let second_flips = endpoints[second].chain_id > 0;
        let (valid, first_chain, second_chain) = connection_state(
            first,
            second,
            first_flips,
            second_flips,
            positions,
            first_point,
            &endpoints,
            &mut chains,
        );
        if !valid {
            update_endpoint(
                first,
                first_index,
                first_point,
                &tree,
                positions,
                &mut endpoints,
                &mut chains,
                &mut queue,
            );
            continue;
        }
        connect(
            first,
            second,
            first_flips,
            second_flips,
            first_chain,
            second_chain,
            connections_left == 1,
            first_point,
            first_index,
            &tree,
            positions,
            &mut endpoints,
            &mut chains,
            &mut queue,
        );
        connections_left -= 1;
    }
    assert_eq!(
        connections_left, 0,
        "source greedy2 iteration bound exhausted"
    );

    if first_point.is_none() {
        first_point = Some(pop_open_endpoint(&mut queue, &mut endpoints));
    }
    let _last_point = pop_open_endpoint(&mut queue, &mut endpoints);
    while !queue.is_empty() {
        queue.pop(&mut endpoints);
    }
    collect_chain(
        first_point.expect("chain has a first endpoint"),
        &endpoints,
        segment_count,
    )
}

fn initialize_candidates(
    tree: &KdTree,
    positions: &[[f64; 2]],
    first: Option<usize>,
    first_index: usize,
    endpoints: &mut [EndPoint],
) {
    for index in 0..endpoints.len() {
        if Some(index) == first {
            continue;
        }
        let next = tree.find_closest(positions, positions[index], |candidate| {
            candidate != first_index && (candidate ^ index) > 1
        });
        endpoints[index].candidate = Some(next);
        endpoints[index].distance = distance(positions[index], positions[next]);
    }
}

fn initialize_queue(first: Option<usize>, endpoints: &mut [EndPoint]) -> Queue {
    let mut queue = Queue::with_capacity(endpoints.len() * 2);
    for index in 0..endpoints.len() {
        if Some(index) != first {
            queue.push(index, endpoints);
        }
    }
    queue
}

#[allow(clippy::too_many_arguments)]
fn connection_state(
    first: usize,
    second: usize,
    first_flips: bool,
    second_flips: bool,
    positions: &[[f64; 2]],
    first_point: Option<usize>,
    endpoints: &[EndPoint],
    chains: &mut Chains,
) -> (bool, usize, usize) {
    if endpoints[second].chain_id > 0 && endpoints[second ^ 1].chain_id > 0 {
        return (false, 0, 0);
    }
    let first_chain =
        chains.equivalent(endpoints[if first_flips { first } else { first ^ 1 }].chain_id);
    let second_chain =
        chains.equivalent(endpoints[if second_flips { second } else { second ^ 1 }].chain_id);
    if first_chain != 0 && first_chain == second_chain {
        return (false, first_chain, second_chain);
    }
    let mut current_distance = distance(positions[first], positions[second]);
    if first_flips {
        current_distance += chains.flip_penalty(first_chain);
    }
    if second_flips {
        current_distance += chains.flip_penalty(second_chain);
    }
    if (current_distance - endpoints[first].distance).abs() > SCALED_EPSILON {
        return (false, first_chain, second_chain);
    }
    if let Some(first_point) = first_point {
        if first_flips
            && (chains.begin(first_chain) == first_point || chains.end(first_chain) == first_point)
        {
            return (false, first_chain, second_chain);
        }
        if second_flips
            && (chains.begin(second_chain) == first_point
                || chains.end(second_chain) == first_point)
        {
            return (false, first_chain, second_chain);
        }
    }
    (true, first_chain, second_chain)
}

#[allow(clippy::too_many_arguments)]
fn connect(
    first: usize,
    second: usize,
    first_flips: bool,
    second_flips: bool,
    first_chain: usize,
    second_chain: usize,
    final_connection: bool,
    first_point: Option<usize>,
    first_index: usize,
    tree: &KdTree,
    positions: &[[f64; 2]],
    endpoints: &mut [EndPoint],
    chains: &mut Chains,
    queue: &mut Queue,
) {
    queue.pop(endpoints);
    queue.remove(endpoints[second].heap_index, endpoints);
    endpoints[first].candidate = None;
    if first_flips {
        chains.flip(first_chain, endpoints);
    }
    if second_flips {
        chains.flip(second_chain, endpoints);
    }
    let first_id = (first_chain != 0).then_some(first_chain);
    let second_id = (second_chain != 0).then_some(second_chain);
    let chain_id = chains.merge(first_chain, second_chain);
    chains.assign(chain_id, (first_id, second_id), (first, second), positions);
    let begin = chains.begin(chain_id);
    let end = chains.end(chain_id);
    remove_internal_endpoint(begin, first ^ 1, endpoints, queue);
    remove_internal_endpoint(end, second ^ 1, endpoints, queue);
    endpoints[first].edge_out = Some(second);
    endpoints[second].edge_out = Some(first);
    for index in [first, second, first ^ 1, second ^ 1] {
        endpoints[index].chain_id = chain_id;
    }
    if Some(begin) != first_point {
        endpoints[begin].chain_id = 0;
    }
    if Some(end) != first_point {
        endpoints[end].chain_id = 0;
    }
    if final_connection {
        return;
    }
    update_endpoint(
        begin ^ 1,
        first_index,
        first_point,
        tree,
        positions,
        endpoints,
        chains,
        queue,
    );
    update_endpoint(
        end ^ 1,
        first_index,
        first_point,
        tree,
        positions,
        endpoints,
        chains,
        queue,
    );
    if first_flips {
        update_endpoint(
            begin,
            first_index,
            first_point,
            tree,
            positions,
            endpoints,
            chains,
            queue,
        );
    }
    if second_flips {
        update_endpoint(
            end,
            first_index,
            first_point,
            tree,
            positions,
            endpoints,
            chains,
            queue,
        );
    }
}

fn remove_internal_endpoint(
    chain_end: usize,
    old_opposite: usize,
    endpoints: &mut [EndPoint],
    queue: &mut Queue,
) {
    if chain_end != old_opposite && endpoints[old_opposite].heap_index != usize::MAX {
        queue.remove(endpoints[old_opposite].heap_index, endpoints);
    }
}

#[allow(clippy::too_many_arguments)]
fn update_endpoint(
    index: usize,
    first_index: usize,
    first_point: Option<usize>,
    tree: &KdTree,
    positions: &[[f64; 2]],
    endpoints: &mut [EndPoint],
    chains: &mut Chains,
    queue: &mut Queue,
) {
    endpoints[index].candidate = None;
    if first_index == index || (endpoints[index].chain_id > 0 && first_index == (index ^ 1)) {
        if endpoints[index].heap_index != usize::MAX {
            queue.remove(endpoints[index].heap_index, endpoints);
        }
        return;
    }
    let this_chain =
        chains.equivalent(endpoints[index].chain_id.max(endpoints[index ^ 1].chain_id));
    let next = tree.find_closest(positions, positions[index], |candidate| {
        if (candidate ^ index) <= 1 || candidate == first_index {
            return false;
        }
        let candidate_chain = endpoints[candidate].chain_id;
        let opposite_chain = endpoints[candidate ^ 1].chain_id;
        if candidate_chain > 0 && opposite_chain > 0 {
            return false;
        }
        let other_chain = chains.equivalent(candidate_chain.max(opposite_chain));
        if this_chain == other_chain {
            return this_chain == 0;
        }
        if candidate_chain > 0
            && first_point.is_some_and(|first| {
                chains.begin(other_chain) == first || chains.end(other_chain) == first
            })
        {
            return false;
        }
        true
    });
    endpoints[index].candidate = Some(next);
    let mut candidate_distance = distance(positions[index], positions[next]);
    if endpoints[index].chain_id > 0 {
        candidate_distance += chains.flip_penalty(this_chain);
    }
    if endpoints[next].chain_id > 0 {
        candidate_distance += chains.flip_penalty(endpoints[next].chain_id);
    }
    endpoints[index].distance = candidate_distance;
    if endpoints[index].heap_index == usize::MAX {
        queue.push(index, endpoints);
    } else {
        queue.update(endpoints[index].heap_index, endpoints);
    }
}

fn pop_open_endpoint(queue: &mut Queue, endpoints: &mut [EndPoint]) -> usize {
    loop {
        let endpoint = queue.pop(endpoints);
        if endpoints[endpoint].edge_out.is_none() {
            return endpoint;
        }
    }
}

fn collect_chain(first: usize, endpoints: &[EndPoint], segment_count: usize) -> Vec<(usize, bool)> {
    let mut output = Vec::with_capacity(segment_count);
    let mut current = Some(first);
    while let Some(endpoint) = current {
        output.push((endpoint >> 1, endpoint & 1 != 0));
        current = endpoints[endpoint ^ 1].edge_out;
    }
    assert_eq!(output.len(), segment_count);
    output
}

fn squared_distance(first: [f64; 2], second: [f64; 2]) -> f64 {
    let x = second[0] - first[0];
    let y = second[1] - first[1];
    x * x + y * y
}
