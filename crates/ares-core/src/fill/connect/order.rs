//! FillBase.cpp:1830–1897: simulate joins and choose a contour-phase order.
use super::{
    contour::path_length_along_contour_ccw,
    hooks::HookLimits,
    types::{Arc, WorkingGraph},
};
use crate::geometry::fixed_gcc_sort_by;

pub(super) fn sort_arcs(
    arcs: &mut Vec<Arc>,
    graph: &WorkingGraph,
    spacing: f64,
    limits: HookLimits,
) {
    fixed_gcc_sort_by(arcs, |a, b| a.length < b.length);
    let mut best = count_joins(arcs, graph, &limits);
    let bucket = 0.5 * spacing;
    for backwards in [false, true] {
        let mut candidate = arcs.clone();
        fixed_gcc_sort_by(&mut candidate, |a, b| {
            let ba = (a.length / bucket).round() as i64;
            let bb = (b.length / bucket).round() as i64;
            let a = &graph.intersections[a.intersection_index];
            let b = &graph.intersections[b.intersection_index];
            if ba != bb {
                ba < bb
            } else if a.contour_index != b.contour_index {
                a.contour_index < b.contour_index
            } else if backwards {
                a.param > b.param
            } else {
                a.param < b.param
            }
        });
        let joins = count_joins(&candidate, graph, &limits);
        if joins > best {
            best = joins;
            *arcs = candidate;
        }
    }
}
fn root(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}
fn count_joins(arcs: &[Arc], graph: &WorkingGraph, limits: &HookLimits) -> usize {
    let mut consumed = graph
        .intersections
        .iter()
        .map(|p| p.consumed)
        .collect::<Vec<_>>();
    let mut parent = (0..graph.paths.len()).collect::<Vec<_>>();
    let mut joins = 0;
    for arc in arcs {
        let a = arc.intersection_index;
        let b = graph.intersections[a].next.unwrap();
        if consumed[a]
            || consumed[b]
            || limits
                .multiline_reach
                .is_some_and(|reach| arc.length < reach)
        {
            continue;
        }
        let ra = root(&mut parent, a / 2);
        let rb = root(&mut parent, b / 2);
        if ra == rb {
            continue;
        }
        if arc.length < limits.anchor_max {
            parent[rb] = ra;
            joins += 1;
        } else if limits.anchor <= limits.epsilon {
            continue;
        }
        consumed[a] = true;
        consumed[b] = true;
    }
    for e in 0..consumed.len() {
        let cp = &graph.intersections[e];
        let Some(contour) = cp.contour_index else {
            continue;
        };
        if consumed[e] {
            continue;
        }
        let length = *graph.boundary[contour].params.last().unwrap();
        let prev = cp.prev.unwrap();
        let next = cp.next.unwrap();
        let lprev = if prev != e
            && !consumed[prev]
            && !(cp.prev_trimmed || graph.intersections[prev].next_trimmed)
        {
            path_length_along_contour_ccw(&graph.intersections[prev], cp, length)
        } else {
            f64::MAX
        };
        let lnext = if next != e
            && !consumed[next]
            && !(cp.next_trimmed || graph.intersections[next].prev_trimmed)
        {
            path_length_along_contour_ccw(cp, &graph.intersections[next], length)
        } else {
            f64::MAX
        };
        consumed[e] = limits.anchor > limits.epsilon;
        for l in [lprev.min(lnext), lprev.max(lnext)] {
            if l > limits.anchor_max {
                break;
            }
            let other = if l == lprev { prev } else { next };
            let ra = root(&mut parent, e / 2);
            let rb = root(&mut parent, other / 2);
            if ra == rb {
                continue;
            }
            parent[rb] = ra;
            consumed[e] = true;
            consumed[other] = true;
            joins += 1;
            break;
        }
    }
    joins
}
