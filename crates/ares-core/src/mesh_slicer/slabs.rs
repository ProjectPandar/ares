//! TriangleMeshSlicer.cpp::slice_mesh_slabs / make_slab_loops.

use super::{
    EndpointReference, FacetEdgeType, IntersectionLine, IntersectionPoint,
    chain_lines_by_triangle_connectivity, index_mesh_edges,
};
use crate::{SliceError, geometry::Polygon};

mod facet;

#[derive(Clone, Copy, PartialEq)]
enum Orientation {
    Up,
    Down,
    Vertical,
    Degenerate,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    General,
    Top,
    Bottom,
    TopBottom,
}

#[derive(Clone, Copy)]
struct SlabLine {
    line: IntersectionLine,
    kind: Kind,
}

struct SlabLines {
    at_slice: Vec<Vec<SlabLine>>,
    between: Vec<Vec<IntersectionLine>>,
}

/// Ordinal of upstream `FacetEdgeType::Slab`, the type of every in-between slab line.
const SLAB_EDGE_TYPE: u8 = 5;

impl SlabLines {
    /// TriangleMeshSlicer.cpp:1077–1093: the facet loop leaves the lines in scheduling order.
    fn sort_canonically(&mut self) {
        for slab in &mut self.at_slice {
            slab.sort_unstable_by_key(|entry| entry.line.canonical_key(entry.kind as u8));
        }
        for slab in &mut self.between {
            slab.sort_unstable_by_key(|line| line.canonical_key(SLAB_EDGE_TYPE));
        }
    }
}

pub(crate) struct ProjectedSlabs {
    pub(crate) top: Vec<Vec<Polygon>>,
    pub(crate) bottom: Vec<Vec<Polygon>>,
    pub(crate) vertical: Vec<([f32; 3], [f32; 3])>,
}

pub(crate) struct BottomSlabs {
    pub(crate) polygons: Vec<Vec<Polygon>>,
    pub(crate) vertical: Vec<([f32; 3], [f32; 3])>,
}

fn point(vertex: [f32; 3], index: u32) -> IntersectionPoint {
    IntersectionPoint::new(
        crate::geometry::Point::new(vertex[0] as i64, vertex[1] as i64),
        EndpointReference::Vertex(index),
    )
}

fn line(a: IntersectionPoint, b: IntersectionPoint) -> IntersectionLine {
    IntersectionLine::new(a, b, FacetEdgeType::General)
}

fn reverse(input: IntersectionLine) -> IntersectionLine {
    line(input.b(), input.a())
}

fn top_reference(input: IntersectionPoint, edge_count: u32) -> IntersectionPoint {
    let reference = match input.reference() {
        EndpointReference::Vertex(index) => EndpointReference::Vertex(index),
        EndpointReference::Edge(index) => EndpointReference::Edge(index + edge_count),
    };
    IntersectionPoint::new(input.point(), reference)
}

fn vertical_facet(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let u: [f32; 3] = std::array::from_fn(|axis| b[axis] - a[axis]);
    let v: [f32; 3] = std::array::from_fn(|axis| c[axis] - a[axis]);
    let normal = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let squared = normal[0] * normal[0] + (normal[1] * normal[1] + normal[2] * normal[2]);
    let normal = if squared > 0.0 {
        let length = squared.sqrt();
        normal.map(|value| value / length)
    } else {
        normal
    };
    (
        std::array::from_fn(|axis| (a[axis] + b[axis] + c[axis]) / 3.0),
        normal,
    )
}

fn supported_neighbors(
    neighbors: [Option<usize>; 3],
    orientations: &[Orientation],
    wanted: Orientation,
) -> [Option<usize>; 3] {
    neighbors.map(|neighbor| {
        neighbor.filter(|&index| {
            orientations[index] == wanted || orientations[index] == Orientation::Degenerate
        })
    })
}

pub(crate) fn slice_bottom_slabs(
    vertices: &[[f32; 3]],
    triangles: &[[u32; 3]],
    zs: &[f32],
    mirrored: bool,
) -> Result<BottomSlabs, SliceError> {
    let ProjectedSlabs {
        bottom, vertical, ..
    } = slice_mesh_slabs(vertices, triangles, zs, mirrored, (false, true, true))?;
    Ok(BottomSlabs {
        polygons: bottom,
        vertical,
    })
}

pub(crate) fn slice_mesh_slabs(
    vertices: &[[f32; 3]],
    triangles: &[[u32; 3]],
    zs: &[f32],
    mirrored: bool,
    (top, bottom, include_vertical): (bool, bool, bool),
) -> Result<ProjectedSlabs, SliceError> {
    let topology = index_mesh_edges(triangles)?;
    let face_neighbors = topology.build_face_neighbors();
    let edge_count = topology.edge_count() as u32;
    let mut orientations = Vec::with_capacity(triangles.len());
    let mut vertical = Vec::new();
    for triangle in triangles {
        let [a, b, c] = triangle.map(|index| vertices[index as usize]);
        let [ax, ay] = [a[0] as i64, a[1] as i64];
        let [bx, by] = [b[0] as i64, b[1] as i64];
        let [cx, cy] = [c[0] as i64, c[1] as i64];
        let determinant =
            ((bx - ax) * (cy - by) - (by - ay) * (cx - bx)) * if mirrored { -1 } else { 1 };
        let orientation = if determinant > 0 {
            Orientation::Up
        } else if determinant < 0 {
            Orientation::Down
        } else if a == b || a == c || b == c {
            Orientation::Degenerate
        } else {
            Orientation::Vertical
        };
        if orientation == Orientation::Vertical && include_vertical {
            vertical.push(vertical_facet(a, b, c));
        }
        orientations.push(orientation);
    }
    let new_lines = || SlabLines {
        at_slice: vec![Vec::new(); zs.len()],
        between: vec![Vec::new(); zs.len()],
    };
    let (mut top, mut bottom) = (top.then(new_lines), bottom.then(new_lines));
    for (face, &triangle) in triangles.iter().enumerate() {
        let make_facet = |wanted| facet::Facet {
            vertices: triangle.map(|index| vertices[index as usize]),
            indices: triangle,
            neighbors: supported_neighbors(face_neighbors[face], &orientations, wanted),
            edges: topology.face_edge_ids()[face],
        };
        if matches!(
            orientations[face],
            Orientation::Up | Orientation::Degenerate
        ) && let Some(lines) = top.as_mut()
        {
            facet::slice::<true>(make_facet(Orientation::Up), edge_count, zs, lines);
        }
        if matches!(
            orientations[face],
            Orientation::Down | Orientation::Degenerate
        ) && let Some(lines) = bottom.as_mut()
        {
            facet::slice::<false>(make_facet(Orientation::Down), edge_count, zs, lines);
        }
    }
    for lines in [top.as_mut(), bottom.as_mut()].into_iter().flatten() {
        lines.sort_canonically();
    }
    Ok(ProjectedSlabs {
        top: top
            .map(|lines| make_loops::<true>(lines, edge_count))
            .unwrap_or_default(),
        bottom: bottom
            .map(|lines| make_loops::<false>(lines, edge_count))
            .unwrap_or_default(),
        vertical,
    })
}

fn make_loops<const TOP: bool>(mut lines: SlabLines, edge_count: u32) -> Vec<Vec<Polygon>> {
    let count = lines.at_slice.len();
    let mut polygons = Vec::with_capacity(count);
    for slab in 0..count {
        let mut input = Vec::new();
        let below = if TOP { Some(slab) } else { slab.checked_sub(1) };
        let above = if !TOP || slab + 1 < count {
            Some(slab + usize::from(TOP))
        } else {
            None
        };
        if let Some(index) = below {
            input.extend(
                lines.at_slice[index]
                    .iter()
                    .filter(|entry| entry.kind != Kind::Top)
                    .map(|entry| entry.line),
            );
        }
        input.append(&mut lines.between[slab]);
        if let Some(index) = above {
            input.extend(
                lines.at_slice[index]
                    .iter()
                    .filter(|entry| entry.kind != Kind::Bottom)
                    .map(|entry| {
                        let reversed = reverse(entry.line);
                        line(
                            top_reference(reversed.a(), edge_count),
                            top_reference(reversed.b(), edge_count),
                        )
                    }),
            );
        }
        let (loops, open) = chain_lines_by_triangle_connectivity(input, &[]).into_parts();
        debug_assert!(open.is_empty(), "slab facet connectivity must close");
        polygons.push(loops);
    }
    polygons
}
