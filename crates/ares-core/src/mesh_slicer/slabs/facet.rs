//! TriangleMeshSlicer.cpp::slice_facet_with_slabs<ProjectionFromTop>.

use super::IntersectionLine;
use super::{
    EndpointReference, FacetEdgeType, IntersectionPoint, Kind, SlabLine, SlabLines, line, point,
    reverse, top_reference,
};
use crate::mesh_slicer::intersect_facet;

pub(super) struct Facet {
    pub(super) vertices: [[f32; 3]; 3],
    pub(super) indices: [u32; 3],
    pub(super) neighbors: [Option<usize>; 3],
    pub(super) edges: [u32; 3],
}

fn reverse_when(input: IntersectionLine, reverse_order: bool) -> IntersectionLine {
    if reverse_order { reverse(input) } else { input }
}

fn horizontal<const TOP: bool>(facet: &Facet, first: usize, zs: &[f32], lines: &mut SlabLines) {
    let (vertices, indices, neighbors) = (facet.vertices, facet.indices, facet.neighbors);
    let coplanar = first != zs.len()
        && vertices.iter().all(|vertex| vertex[2] == vertices[0][2])
        && zs[first] == vertices[0][2];
    if coplanar {
        let target = if TOP { first } else { first + 1 };
        if target == zs.len() {
            return;
        }
        for edge in (0..3).filter(|&edge| neighbors[edge].is_none()) {
            let next = (edge + 1) % 3;
            lines.at_slice[target].push(SlabLine {
                line: line(
                    point(vertices[edge], indices[edge]),
                    point(vertices[next], indices[next]),
                ),
                kind: if TOP { Kind::Bottom } else { Kind::Top },
            });
        }
    } else {
        if (TOP && first == 0) || (!TOP && first == zs.len()) {
            return;
        }
        let slab = first - usize::from(TOP);
        for edge in (0..3).filter(|&edge| neighbors[edge].is_none()) {
            let next = (edge + 1) % 3;
            lines.between[slab].push(reverse_when(
                line(
                    point(vertices[edge], indices[edge]),
                    point(vertices[next], indices[next]),
                ),
                !TOP,
            ));
        }
    }
}

fn record_slice<const TOP: bool>(
    current: Option<SlabLine>,
    indices: [u32; 3],
    neighbors: [Option<usize>; 3],
    slab: usize,
    lines: &mut SlabLines,
) -> Option<SlabLine> {
    let mut input = current?;
    let cutting = input.kind == Kind::Bottom;
    let mut retain = !cutting;
    if matches!(input.kind, Kind::Top | Kind::Bottom) {
        let vertex = [input.line.b(), input.line.a()][usize::from(cutting)];
        let EndpointReference::Vertex(index) = vertex.reference() else {
            unreachable!()
        };
        let edge = indices.iter().position(|&vertex| vertex == index).unwrap();
        if neighbors[edge].is_none() {
            retain = true;
        } else if input.kind == Kind::Top {
            input.kind = Kind::TopBottom;
        }
    }
    if retain {
        lines.at_slice[slab].push(SlabLine {
            line: reverse_when(input.line, !TOP),
            kind: input.kind,
        });
    }
    Some(input)
}

fn intersection(
    z: f32,
    vertices: &[[f32; 3]; 3],
    indices: [u32; 3],
    edges: [u32; 3],
) -> Option<SlabLine> {
    if let Some(input) = intersect_facet(z, vertices, indices, edges) {
        let kind = if input.edge_type() == FacetEdgeType::Top {
            Kind::Top
        } else {
            Kind::General
        };
        return Some(SlabLine { line: input, kind });
    }
    for edge in 0..3 {
        let next = (edge + 1) % 3;
        if vertices[edge][2] == z && vertices[next][2] == z {
            return Some(SlabLine {
                line: line(
                    point(vertices[edge], indices[edge]),
                    point(vertices[next], indices[next]),
                ),
                kind: Kind::Bottom,
            });
        }
    }
    None
}

fn on_edge(input: Option<SlabLine>, edge: u32) -> Option<IntersectionPoint> {
    let input = input?;
    [input.line.a(), input.line.b()]
        .into_iter()
        .find(|point| point.reference() == EndpointReference::Edge(edge))
}

pub(super) fn slice<const TOP: bool>(
    facet: Facet,
    edge_count: u32,
    zs: &[f32],
    lines: &mut SlabLines,
) {
    let Facet {
        vertices,
        indices,
        neighbors,
        edges,
    } = facet;
    let min_z = vertices[0][2].min(vertices[1][2].min(vertices[2][2]));
    let max_z = vertices[0][2].max(vertices[1][2].max(vertices[2][2]));
    let first = zs.partition_point(|&z| z < min_z);
    let last = zs.partition_point(|&z| z <= max_z);
    if first == last || min_z == max_z {
        horizontal::<TOP>(&facet, first, zs, lines);
        return;
    }
    let mut previous = None;
    for plane in first..last {
        let current = record_slice::<TOP>(
            intersection(zs[plane], &vertices, indices, edges),
            indices,
            neighbors,
            plane,
            lines,
        );
        let slab = plane - usize::from(TOP && plane > 0);
        for edge in (0..3).filter(|_| !TOP || plane > 0) {
            if neighbors[edge].is_some() {
                continue;
            }
            let next = (edge + 1) % 3;
            let now = on_edge(current, edges[edge]);
            let before = on_edge(previous, edges[edge]);
            let upwards = vertices[next][2] > vertices[edge][2];
            let low = if upwards { edge } else { next };
            let high = if upwards { next } else { edge };
            let segment = match (before, now) {
                (Some(a), Some(b)) => Some(line(a, top_reference(b, edge_count))),
                (None, Some(b)) => Some(line(
                    point(vertices[low], indices[low]),
                    top_reference(b, edge_count),
                )),
                (Some(a), None) => Some(line(a, point(vertices[high], indices[high]))),
                (None, None)
                    if (vertices[edge][2] < zs[plane] || vertices[next][2] < zs[plane])
                        && (plane == first
                            || vertices[edge][2] > zs[plane - 1]
                            || vertices[next][2] > zs[plane - 1]) =>
                {
                    lines.between[slab].push(reverse_when(
                        line(
                            point(vertices[edge], indices[edge]),
                            point(vertices[next], indices[next]),
                        ),
                        !TOP,
                    ));
                    None
                }
                (None, None) => None,
            };
            if let Some(segment) = segment {
                lines.between[slab].push(reverse_when(segment, TOP != upwards));
            }
        }
        previous = current;
    }
    if TOP || last != zs.len() {
        let slab = last - usize::from(TOP);
        for edge in 0..3 {
            if neighbors[edge].is_some() {
                continue;
            }
            let next = (edge + 1) % 3;
            if let Some(a) = on_edge(previous, edges[edge]) {
                let upwards = vertices[next][2] > vertices[edge][2];
                let high = [edge, next][usize::from(upwards)];
                let segment = line(a, point(vertices[high], indices[high]));
                lines.between[slab].push(reverse_when(segment, TOP != upwards));
            } else if vertices[edge][2] > zs[last - 1] || vertices[next][2] > zs[last - 1] {
                lines.between[slab].push(reverse_when(
                    line(
                        point(vertices[edge], indices[edge]),
                        point(vertices[next], indices[next]),
                    ),
                    !TOP,
                ));
            }
        }
    }
}
