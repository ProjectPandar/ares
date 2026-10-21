//! MultiMaterialSegmentation.cpp:327–479 (left-most traversal and rollback).

use super::{ArcType, Graph, math::*};
use crate::geometry::{ExPolygon, Polygon, fixed_gcc_sort_by};

fn next(graph: &Graph, used: &[bool], original: usize, color: usize) -> usize {
    let arc = graph.arcs[original];
    let start = graph.nodes[arc.from].point;
    let end = graph.nodes[arc.to].point;
    let incoming = unit(sub(start, end));
    let mut choices = Vec::new();
    for &id in &graph.nodes[arc.to].arcs {
        let candidate = graph.arcs[id];
        if graph.nodes[candidate.to].point == start
            || used[id]
            || (arc.kind == ArcType::Border && arc.color != color)
            || (candidate.kind == ArcType::Border && candidate.color != color)
        {
            continue;
        }
        let direction = unit(sub(
            graph.nodes[candidate.to].point,
            graph.nodes[candidate.from].point,
        ));
        choices.push((id, angle(direction, incoming)));
    }
    fixed_gcc_sort_by(&mut choices, |a, b| a.1 < b.1);
    choices.first().map_or(original, |&(id, _)| id)
}

fn polygon(graph: &Graph, ids: &[usize], closing: Option<[f64; 2]>) -> Polygon {
    let mut points = ids
        .iter()
        .map(|&id| rounded(graph.nodes[graph.arcs[id].from].point))
        .collect::<Vec<_>>();
    if let Some(p) = closing {
        points.push(rounded(p));
    }
    Polygon::new(points)
}

fn self_interacts(poly: &Polygon) -> bool {
    let lines = poly.lines();
    for i in 0..lines.len() {
        for j in i + 2..lines.len().min(lines.len() + i - 1) {
            if lines[i].intersection(lines[j]).is_some() {
                return true;
            }
        }
    }
    false
}

pub(super) fn extract(graph: &Graph, states: usize) -> Vec<Vec<ExPolygon>> {
    let mut used = vec![false; graph.arcs.len()];
    let mut output = vec![Vec::new(); states];
    for &first in graph.nodes[..graph.border_count]
        .iter()
        .flat_map(|node| &node.arcs)
    {
        let arc = graph.arcs[first];
        if arc.kind == ArcType::Interior || used[first] {
            continue;
        }
        used[first] = true;
        let start = graph.nodes[arc.from].point;
        let mut ids = vec![first];
        let mut current = first;
        loop {
            let id = next(graph, &used, current, arc.color);
            if used[id] {
                break;
            }
            ids.push(id);
            used[id] = true;
            current = id;
            let end = graph.arcs[current].to;
            if graph.nodes[end].point == start && graph.nodes[end].arcs.iter().all(|&id| used[id]) {
                break;
            }
        }
        let poly = polygon(graph, &ids, None);
        if poly.is_counter_clockwise() && poly.points().len() >= 3 {
            output[arc.color].push(ExPolygon::new(poly, Vec::new()));
        } else {
            rollback(graph, &mut used, &mut ids, &mut output[arc.color]);
        }
    }
    output
}

fn rollback(graph: &Graph, used: &mut [bool], ids: &mut Vec<usize>, output: &mut Vec<ExPolygon>) {
    while ids.len() > 1 {
        used[ids.pop().unwrap()] = false;
        let last = graph.arcs[*ids.last().unwrap()].to;
        let poly = polygon(graph, ids, Some(graph.nodes[last].point));
        if !self_interacts(&poly) && poly.is_counter_clockwise() && poly.points().len() >= 3 {
            output.push(ExPolygon::new(poly, Vec::new()));
            break;
        }
    }
}
