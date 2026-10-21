//! MultiMaterialSegmentation.cpp::build_graph1527–1742.

use super::super::painted_lines::ColoredLine;
use super::{Graph, attach::Attach, math::Bounds, vertices};
use crate::{
    SliceError,
    geometry::{CoordinateScale, voronoi},
};

pub(super) fn build(
    contours: &[Vec<ColoredLine>],
    scale: CoordinateScale,
) -> Result<Graph, SliceError> {
    let colored = contours.iter().flatten().copied().collect::<Vec<_>>();
    let segments = colored.iter().map(|line| line.line).collect::<Vec<_>>();
    let diagram = voronoi::build(&segments).map_err(|e| SliceError::InvalidInput(e.to_string()))?;
    let mut graph = Graph::new(contours);
    let bbox = Bounds::from_points(segments.iter().map(|line| line.a)).unwrap();
    let epsilon = scale.scaled_delta(1e-4);
    let mapping = vertices::append(&mut graph, &diagram, bbox, (epsilon as i64, epsilon));
    let mut processed = vec![false; diagram.num_edges()];
    let mut force = contours
        .iter()
        .map(|lines| lines.iter().all(|line| line.color == lines[0].color))
        .collect::<Vec<_>>();
    let mut context = Attach {
        graph: &mut graph,
        diagram: &diagram,
        processed: &mut processed,
        mapping: &mapping,
        lines: &colored,
        segments: &segments,
        force: &mut force,
        bbox: bbox.inflated(scale.scaled_delta(10.0) as i64),
        epsilon,
    };
    for i in 0..diagram.num_edges() {
        let id = diagram.edge_index_unchecked(i);
        let twin = diagram.edge_get_twin(id).unwrap();
        let source = diagram
            .cell(diagram.edge_get_cell(id).unwrap())
            .unwrap()
            .source_index()
            .usize();
        let other = diagram
            .cell(diagram.edge_get_cell(twin).unwrap())
            .unwrap()
            .source_index()
            .usize();
        if source > other || context.processed[i] {
            continue;
        }
        let v0 = diagram.edge_get_vertex0(id).unwrap();
        let v1 = diagram.edge_get_vertex1(id).unwrap();
        if !diagram.edge_is_finite(id).unwrap() && (v0.is_some() || v1.is_some()) {
            context.infinite(id, source);
        } else if diagram.edge_is_finite(id).unwrap() {
            context.finite(id, source);
        }
    }
    for i in 0..diagram.num_edges() {
        let id = diagram.edge_index_unchecked(i);
        let twin = diagram.edge_get_twin(id).unwrap();
        let source = diagram
            .cell(diagram.edge_get_cell(id).unwrap())
            .unwrap()
            .source_index()
            .usize();
        let other = diagram
            .cell(diagram.edge_get_cell(twin).unwrap())
            .unwrap()
            .source_index()
            .usize();
        if source > other || context.processed[i] {
            continue;
        }
        if diagram.edge_is_finite(id).unwrap() {
            let (a, b) = context.nodes(id);
            let valid = a < context.graph.nodes.len() && b < context.graph.nodes.len();
            if valid && a == b {
                continue;
            }
            if valid {
                context.graph.connect(a, b);
            }
        }
        context.mark(id);
    }
    graph.prune();
    Ok(graph)
}
