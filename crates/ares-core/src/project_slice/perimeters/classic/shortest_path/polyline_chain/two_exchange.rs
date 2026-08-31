use std::ops::Range;

use crate::geometry::{Point, Polyline, fixed_gcc_sort_by};

#[derive(Clone, Copy)]
struct Edge {
    first: [f64; 2],
    second: [f64; 2],
    source_index: usize,
}

impl Edge {
    fn flip(&mut self) {
        std::mem::swap(&mut self.first, &mut self.second);
    }
}

#[derive(Clone, Copy, Default)]
struct ConnectionCost {
    cost: f64,
    flipped: f64,
}

impl std::ops::Sub for ConnectionCost {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            cost: self.cost - rhs.cost,
            flipped: self.flipped - rhs.flipped,
        }
    }
}

// `improve_ordering_by_two_exchanges_with_segment_flipping` from
// OrcaSlicer 2.4.2 `ShortestPath.cpp:1911-1956`.
pub(super) fn improve(polylines: &mut Vec<Polyline>) {
    if polylines.len() < 2 {
        return;
    }
    let mut edges = polylines
        .iter()
        .enumerate()
        .map(|(source_index, polyline)| Edge {
            first: coordinates(polyline.front().expect("polyline is valid")),
            second: coordinates(polyline.back().expect("polyline is valid")),
            source_index,
        })
        .collect::<Vec<_>>();
    reorder(&mut edges);
    let mut source = std::mem::take(polylines)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    polylines.reserve(edges.len());
    for edge in edges {
        let mut polyline = source[edge.source_index]
            .take()
            .expect("edge indices are unique");
        if edge.second == coordinates(polyline.front().expect("polyline is valid")) {
            polyline.reverse();
        }
        polylines.push(polyline);
    }
}

fn reorder(edges: &mut Vec<Edge>) {
    let mut connections = vec![ConnectionCost::default(); edges.len()];
    let mut temporary = edges.clone();
    let mut lengths = vec![(0.0, 0usize); edges.len() - 1];
    let mut tried = vec![false; edges.len()];
    for _ in 0..edges.len().min(100) {
        initialize_costs(edges, &mut connections, &mut lengths);
        fixed_gcc_sort_by(&mut lengths, |left, right| left.0 > right.0);
        tried.fill(false);
        let Some(crossover) = find_crossover(edges, &connections, &lengths, &mut tried) else {
            break;
        };
        apply_crossover(edges, &mut temporary, crossover.0, crossover.1, crossover.2);
        std::mem::swap(edges, &mut temporary);
    }
}

fn initialize_costs(
    edges: &[Edge],
    connections: &mut [ConnectionCost],
    lengths: &mut [(f64, usize)],
) {
    connections[0] = ConnectionCost::default();
    for index in 1..edges.len() {
        let first = edges[index - 1];
        let second = edges[index];
        let mut cost = connections[index - 1];
        let length = distance(second.first, first.second);
        cost.cost += length;
        cost.flipped += distance(second.second, first.first);
        connections[index] = cost;
        lengths[index - 1] = (length, index);
    }
}

fn find_crossover(
    edges: &[Edge],
    connections: &[ConnectionCost],
    lengths: &[(f64, usize)],
    tried: &mut [bool],
) -> Option<(usize, usize, usize)> {
    for &(_, longest) in lengths {
        tried[longest] = true;
        let mut best_position = usize::MAX;
        let mut best_cost = connections.last().unwrap().cost;
        let mut best_flip = 0;
        for (position, &was_tried) in tried.iter().enumerate().take(connections.len()).skip(1) {
            if was_tried {
                continue;
            }
            let (a, b) = if position < longest {
                (position, longest)
            } else {
                (longest, position)
            };
            let candidate = minimum_crossover_cost(
                edges,
                0..a,
                connections[a - 1],
                a..b,
                connections[b - 1] - connections[a],
                b..edges.len(),
                *connections.last().unwrap() - connections[b],
                connections.last().unwrap().cost,
            );
            if candidate.1 > 0 && candidate.0 < best_cost {
                best_position = position;
                best_cost = candidate.0;
                best_flip = candidate.1;
            }
        }
        if best_cost < connections.last().unwrap().cost {
            return Some((longest, best_position, best_flip));
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn minimum_crossover_cost(
    edges: &[Edge],
    first_span: Range<usize>,
    first_cost: ConnectionCost,
    second_span: Range<usize>,
    second_cost: ConnectionCost,
    third_span: Range<usize>,
    third_cost: ConnectionCost,
    current_cost: f64,
) -> (f64, usize) {
    let spans = [first_span, second_span, third_span];
    let costs = [first_cost, second_cost, third_cost];
    let mut best_cost = current_cost;
    let mut best_flip = 0;
    for mask in 0..64 {
        let first = if mask == 0 {
            current_cost
        } else {
            arrangement_cost(edges, &spans, &costs, [0, 1, 2], mask)
        };
        let second = arrangement_cost(edges, &spans, &costs, [0, 2, 1], mask);
        let third = arrangement_cost(edges, &spans, &costs, [1, 0, 2], mask);
        for (permutation, candidate) in [first, second, third].into_iter().enumerate() {
            if candidate < best_cost {
                best_cost = candidate;
                best_flip = mask + (permutation << 6);
            }
        }
    }
    (best_cost, best_flip)
}

fn arrangement_cost(
    edges: &[Edge],
    spans: &[Range<usize>; 3],
    costs: &[ConnectionCost; 3],
    order: [usize; 3],
    mask: usize,
) -> f64 {
    let settings = [
        (mask & 1 != 0, mask & 2 != 0),
        (mask & 4 != 0, mask & 8 != 0),
        (mask & 16 != 0, mask & 32 != 0),
    ];
    if order
        .iter()
        .enumerate()
        .any(|(position, &span)| spans[span].len() == 1 && settings[position].0)
    {
        return f64::MAX;
    }
    let oriented = order.map(|span| span);
    let span1 = &spans[oriented[0]];
    let span2 = &spans[oriented[1]];
    let span3 = &spans[oriented[2]];
    let (reverse1, flip1) = settings[0];
    let (reverse2, flip2) = settings[1];
    let (reverse3, flip3) = settings[2];
    selected_cost(costs[oriented[0]], flip1)
        + selected_cost(costs[oriented[1]], flip2)
        + selected_cost(costs[oriented[2]], flip3)
        + distance(
            span_point(edges, span1, reverse1, flip1),
            span_point(edges, span2, !reverse2, flip2),
        )
        + distance(
            span_point(edges, span2, reverse2, flip2),
            span_point(edges, span3, !reverse3, flip3),
        )
}

fn span_point(edges: &[Edge], span: &Range<usize>, start: bool, flipped: bool) -> [f64; 2] {
    if start {
        if flipped {
            edges[span.start].second
        } else {
            edges[span.start].first
        }
    } else if flipped {
        edges[span.end - 1].first
    } else {
        edges[span.end - 1].second
    }
}

fn selected_cost(cost: ConnectionCost, flipped: bool) -> f64 {
    if flipped { cost.flipped } else { cost.cost }
}

fn apply_crossover(
    edges: &[Edge],
    temporary: &mut [Edge],
    mut first: usize,
    mut second: usize,
    mask: usize,
) {
    if first > second {
        std::mem::swap(&mut first, &mut second);
    }
    let spans = [0..first, first..second, second..edges.len()];
    let order = match mask >> 6 {
        0 => [0, 1, 2],
        1 => [0, 2, 1],
        2 => [1, 0, 2],
        _ => unreachable!("three source permutations"),
    };
    let settings = [
        (mask & 1 != 0, mask & 2 != 0),
        (mask & 4 != 0, mask & 8 != 0),
        (mask & 16 != 0, mask & 32 != 0),
    ];
    let mut output = 0;
    for position in 0..3 {
        output = copy_span(
            edges,
            temporary,
            output,
            spans[order[position]].clone(),
            settings[position],
        );
    }
}

fn copy_span(
    edges: &[Edge],
    output: &mut [Edge],
    mut output_index: usize,
    span: Range<usize>,
    settings: (bool, bool),
) -> usize {
    let (reversed, flipped) = settings;
    let indices: Box<dyn Iterator<Item = usize>> = if reversed {
        Box::new(span.rev())
    } else {
        Box::new(span)
    };
    for index in indices {
        let mut edge = edges[index];
        if reversed != flipped {
            edge.flip();
        }
        output[output_index] = edge;
        output_index += 1;
    }
    output_index
}

fn coordinates(point: Point) -> [f64; 2] {
    [point.x() as f64, point.y() as f64]
}

fn distance(first: [f64; 2], second: [f64; 2]) -> f64 {
    (second[0] - first[0]).hypot(second[1] - first[1])
}
