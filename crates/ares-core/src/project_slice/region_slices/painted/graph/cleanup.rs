//! MultiMaterialSegmentation.cpp:1744–1822 (longest near-straight branch).

use super::{ArcType, Graph, math::*};
use crate::geometry::fixed_gcc_sort_by;
use crate::project_slice::region_slices::painted::painted_lines::{ColoredLine, segments};

fn length(graph: &Graph, start: usize, first: usize) -> f64 {
    let mut used = vec![false; graph.arcs.len()];
    used[first] = true;
    let mut current = first;
    let mut from = start;
    let mut total = norm(sub(
        graph.nodes[graph.arcs[current].to].point,
        graph.nodes[from].point,
    ));
    while graph.nodes[graph.arcs[current].to].arcs.len() == 2 {
        let to = graph.arcs[current].to;
        let mut found = None;
        for &id in &graph.nodes[to].arcs {
            let candidate = graph.arcs[id];
            if candidate.kind != ArcType::Interior || used[id] || candidate.to == from {
                continue;
            }
            let a = unit(sub(graph.nodes[from].point, graph.nodes[to].point));
            let b = unit(sub(graph.nodes[candidate.to].point, graph.nodes[to].point));
            if (angle(a, b) - std::f64::consts::PI).abs() >= std::f64::consts::PI / 12.0 {
                continue;
            }
            found = Some(id);
            break;
        }
        let Some(id) = found else {
            break;
        };
        from = to;
        current = id;
        total += norm(sub(
            graph.nodes[graph.arcs[id].to].point,
            graph.nodes[from].point,
        ));
        used[id] = true;
    }
    total
}

pub(super) fn remove_multiple(graph: &mut Graph, contours: &[Vec<ColoredLine>]) {
    for (contour, lines) in contours.iter().enumerate() {
        for (first, _) in segments(lines) {
            let first = graph.offsets[contour] + first;
            if graph.nodes[first].arcs.len() < 3 {
                continue;
            }
            let mut choices = graph.nodes[first]
                .arcs
                .iter()
                .filter(|&&id| graph.arcs[id].kind == ArcType::Interior)
                .map(|&id| (id, length(graph, first, id)))
                .collect::<Vec<_>>();
            fixed_gcc_sort_by(&mut choices, |a, b| a.1 > b.1);
            while choices.len() > 1 {
                let (id, _) = choices.pop().unwrap();
                graph.remove_edge(first, graph.arcs[id].to);
            }
        }
    }
}
