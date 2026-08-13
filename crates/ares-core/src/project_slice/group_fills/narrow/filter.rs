use std::collections::VecDeque;

use crate::geometry::Line;

#[derive(Clone, Copy, Default)]
struct State {
    min_skips_taken: i32,
    total_short_lines: i32,
    initial_touches_long_lines: bool,
    initialized: bool,
}

struct Node {
    line: Line,
    previous: Vec<(usize, usize)>,
    next: Vec<(usize, usize)>,
    removed: bool,
    state: State,
}

#[expect(
    clippy::excessive_nesting,
    reason = "the source vibration pass resets and propagates state in section order"
)]
pub(super) fn apply(sections: Vec<Vec<Line>>, maximum_short_length: i64) -> Vec<Vec<Line>> {
    let mut nodes = sections
        .into_iter()
        .map(|section| {
            section
                .into_iter()
                .map(|line| Node {
                    line,
                    previous: Vec::new(),
                    next: Vec::new(),
                    removed: false,
                    state: State::default(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    connect_adjacent_sections(&mut nodes);

    for initial_section in 0..nodes.len() {
        for node_index in 0..nodes[initial_section].len() {
            if nodes[initial_section][node_index].removed
                || !is_short(
                    nodes[initial_section][node_index].line,
                    maximum_short_length,
                )
            {
                continue;
            }
            let touches_long = touches_live_long_before(
                &nodes,
                (initial_section, node_index),
                maximum_short_length,
            );
            nodes[initial_section][node_index].state = State {
                total_short_lines: 1,
                initial_touches_long_lines: touches_long,
                initialized: true,
                ..State::default()
            };
        }

        for section_index in initial_section..nodes.len() {
            if section_index + 1 < nodes.len() {
                for node in &mut nodes[section_index + 1] {
                    node.state = State::default();
                }
            }
            for node_index in 0..nodes[section_index].len() {
                let id = (section_index, node_index);
                if nodes[section_index][node_index].removed
                    || !nodes[section_index][node_index].state.initialized
                {
                    continue;
                }
                propagate_forward(&mut nodes, id, maximum_short_length);
                if removable(&nodes, id, maximum_short_length) {
                    nodes[section_index][node_index].removed = true;
                    propagate_removal(&mut nodes, id, maximum_short_length);
                }
            }
        }
    }

    nodes
        .into_iter()
        .map(|section| {
            section
                .into_iter()
                .filter_map(|node| (!node.removed).then_some(node.line))
                .collect()
        })
        .collect()
}

#[expect(
    clippy::excessive_nesting,
    reason = "the source pass links every overlapping line in adjacent sections"
)]
fn connect_adjacent_sections(nodes: &mut [Vec<Node>]) {
    for section_index in 0..nodes.len().saturating_sub(1) {
        let (left, right) = nodes.split_at_mut(section_index + 1);
        let current = &mut left[section_index];
        let next = &mut right[0];
        for (current_index, current_node) in current.iter_mut().enumerate() {
            for (next_index, next_node) in next.iter_mut().enumerate() {
                if overlaps_in_y(current_node.line, next_node.line) {
                    current_node.next.push((section_index + 1, next_index));
                    next_node.previous.push((section_index, current_index));
                }
            }
        }
    }
}

fn overlaps_in_y(first: Line, second: Line) -> bool {
    (second.a.y() <= first.a.y() && first.a.y() <= second.b.y())
        || (second.a.y() <= first.b.y() && first.b.y() <= second.b.y())
        || (first.a.y() <= second.a.y() && second.a.y() <= first.b.y())
        || (first.a.y() <= second.b.y() && second.b.y() <= first.b.y())
}

fn is_short(line: Line, maximum: i64) -> bool {
    line.length() < maximum as f64
}

fn touches_live_long_before(nodes: &[Vec<Node>], id: (usize, usize), maximum: i64) -> bool {
    nodes[id.0][id.1].previous.iter().any(|&(section, index)| {
        let node = &nodes[section][index];
        !node.removed && !is_short(node.line, maximum)
    })
}

fn propagated_initial_touches_long(nodes: &[Vec<Node>], id: (usize, usize)) -> bool {
    nodes[id.0][id.1]
        .previous
        .iter()
        .any(|&(section, index)| nodes[section][index].state.initial_touches_long_lines)
}

fn has_live_next(nodes: &[Vec<Node>], id: (usize, usize)) -> bool {
    nodes[id.0][id.1]
        .next
        .iter()
        .any(|&(section, index)| !nodes[section][index].removed)
}

fn removable(nodes: &[Vec<Node>], id: (usize, usize), maximum: i64) -> bool {
    let node = &nodes[id.0][id.1];
    is_short(node.line, maximum)
        && (node.state.total_short_lines > 5
            || (!propagated_initial_touches_long(nodes, id) && !has_live_next(nodes, id)))
}

fn propagate_forward(nodes: &mut [Vec<Node>], id: (usize, usize), maximum: i64) {
    let state = nodes[id.0][id.1].state;
    let next_len = nodes[id.0][id.1].next.len();
    for next_index in 0..next_len {
        let (section, index) = nodes[id.0][id.1].next[next_index];
        if nodes[section][index].removed {
            continue;
        }
        let short = is_short(nodes[section][index].line, maximum);
        if !short && state.min_skips_taken >= 2 {
            continue;
        }
        let total_short_lines = state.total_short_lines + i32::from(short);
        let min_skips_taken = state.min_skips_taken + i32::from(!short);
        let neighbour = &mut nodes[section][index];
        if neighbour.state.initialized {
            neighbour.state.min_skips_taken = neighbour
                .state
                .min_skips_taken
                .max(total_short_lines)
                .min(min_skips_taken);
            if neighbour.state.initial_touches_long_lines {
                neighbour.state.initial_touches_long_lines = state.initial_touches_long_lines;
            }
        } else {
            neighbour.state = State {
                min_skips_taken,
                total_short_lines,
                initial_touches_long_lines: state.initial_touches_long_lines,
                initialized: true,
            };
        }
    }
}

#[expect(
    clippy::excessive_nesting,
    reason = "the source backward FIFO preserves duplicate predecessor visits"
)]
fn propagate_removal(nodes: &mut [Vec<Node>], origin: (usize, usize), maximum: i64) {
    let mut queue = VecDeque::new();
    let previous_len = nodes[origin.0][origin.1].previous.len();
    for previous_index in 0..previous_len {
        let id = nodes[origin.0][origin.1].previous[previous_index];
        if !nodes[id.0][id.1].removed {
            queue.push_back(id);
        }
    }
    while let Some(id) = queue.pop_front() {
        if removable(nodes, id, maximum) {
            nodes[id.0][id.1].removed = true;
            let previous_len = nodes[id.0][id.1].previous.len();
            for previous_index in 0..previous_len {
                let previous_id = nodes[id.0][id.1].previous[previous_index];
                if !nodes[previous_id.0][previous_id.1].removed {
                    queue.push_back(previous_id);
                }
            }
        }
    }
}
