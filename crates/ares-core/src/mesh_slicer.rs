use std::ops::Range;

use crate::SliceError;

mod chaining;
mod intersection;
mod slicing_mode;
mod topology;

pub(crate) use chaining::{
    ChainedLayer, LoopedLayer, chain_lines_by_triangle_connectivity, make_loops,
};
pub(crate) use intersection::{
    EndpointReference, FacetEdgeType, IntersectionLine, IntersectionPoint, intersect_facet,
};
pub(crate) use slicing_mode::{SlicingMode, apply_slicing_mode};
pub(crate) use topology::{MeshTopology, index_mesh_edges};

const RAW_INTERSECTION_LIMIT: usize = 1_000_000;
const RAW_INTERSECTION_LIMIT_ERROR: &str =
    "project raw intersection count exceeds supported limit of 1000000";

#[derive(Debug, Default)]
pub(crate) struct RawIntersectionBudget {
    retained_lines: usize,
}

#[derive(Clone, Copy)]
struct MeshPlaneInput<'a> {
    vertices: &'a [[f32; 3]],
    triangles: &'a [[u32; 3]],
    planes: &'a [f32],
}

impl<'a> MeshPlaneInput<'a> {
    const fn new(vertices: &'a [[f32; 3]], triangles: &'a [[u32; 3]], planes: &'a [f32]) -> Self {
        Self {
            vertices,
            triangles,
            planes,
        }
    }
}

impl RawIntersectionBudget {
    pub(crate) const fn new() -> Self {
        Self { retained_lines: 0 }
    }

    fn claim(&mut self, additional: usize) -> Result<(), SliceError> {
        let retained_lines = self
            .retained_lines
            .checked_add(additional)
            .ok_or_else(raw_intersection_limit_error)?;
        if retained_lines > RAW_INTERSECTION_LIMIT {
            return Err(raw_intersection_limit_error());
        }
        self.retained_lines = retained_lines;
        Ok(())
    }
}

pub(crate) fn slice_mesh_on_planes(
    vertices: &[[f32; 3]],
    triangles: &[[u32; 3]],
    planes: &[f32],
    budget: &mut RawIntersectionBudget,
) -> Result<Vec<Vec<IntersectionLine>>, SliceError> {
    let input = MeshPlaneInput::new(vertices, triangles, planes);
    let topology = index_mesh_edges(triangles)?;
    let mut lines = (0..planes.len()).map(|_| Vec::new()).collect::<Vec<_>>();
    dispatch_indexed_mesh_on_planes_with(
        input,
        &topology,
        budget,
        |_, _| {},
        |plane_index, line| lines[plane_index].push(line),
    )?;
    Ok(lines)
}

fn dispatch_mesh_on_planes_with<Visit, Push>(
    input: MeshPlaneInput<'_>,
    budget: &mut RawIntersectionBudget,
    visit: Visit,
    push: Push,
) -> Result<(), SliceError>
where
    Visit: FnMut(usize, usize),
    Push: FnMut(usize, IntersectionLine),
{
    let topology = index_mesh_edges(input.triangles)?;
    dispatch_indexed_mesh_on_planes_with(input, &topology, budget, visit, push)
}

fn dispatch_indexed_mesh_on_planes_with<Visit, Push>(
    input: MeshPlaneInput<'_>,
    topology: &MeshTopology,
    budget: &mut RawIntersectionBudget,
    mut visit: Visit,
    mut push: Push,
) -> Result<(), SliceError>
where
    Visit: FnMut(usize, usize),
    Push: FnMut(usize, IntersectionLine),
{
    for (face_index, triangle) in input.triangles.iter().enumerate() {
        let face_vertices = triangle.map(|vertex_id| input.vertices[vertex_id as usize]);
        let min_z = face_vertices[0][2].min(face_vertices[1][2].min(face_vertices[2][2]));
        let max_z = face_vertices[0][2].max(face_vertices[1][2].max(face_vertices[2][2]));
        if min_z == max_z {
            continue;
        }
        let edge_ids = topology.face_edge_ids()[face_index];

        for plane_index in eligible_plane_range(input.planes, min_z, max_z) {
            visit(face_index, plane_index);
            if let Some(line) = intersect_facet(
                input.planes[plane_index],
                &face_vertices,
                *triangle,
                edge_ids,
            ) {
                budget.claim(1)?;
                push(plane_index, line);
            }
        }
    }
    Ok(())
}

fn eligible_plane_range(planes: &[f32], min_z: f32, max_z: f32) -> Range<usize> {
    let first = planes.partition_point(|plane| *plane < min_z);
    let end = first + planes[first..].partition_point(|plane| *plane <= max_z);
    first..end
}

fn raw_intersection_limit_error() -> SliceError {
    SliceError::InvalidInput(RAW_INTERSECTION_LIMIT_ERROR.to_owned())
}

type FacetIntersectionFn = fn(f32, &[[f32; 3]; 3], [u32; 3], [u32; 3]) -> Option<IntersectionLine>;
type MeshSliceFn = fn(
    &[[f32; 3]],
    &[[u32; 3]],
    &[f32],
    &mut RawIntersectionBudget,
) -> Result<Vec<Vec<IntersectionLine>>, SliceError>;
type MeshDispatchFn = fn(
    MeshPlaneInput<'_>,
    &mut RawIntersectionBudget,
    fn(usize, usize),
    fn(usize, IntersectionLine),
) -> Result<(), SliceError>;
type LoopRepairFn = fn(ChainedLayer, crate::geometry::Coord) -> LoopedLayer;

const _: FacetIntersectionFn = intersect_facet;
const _: fn() -> RawIntersectionBudget = RawIntersectionBudget::new;
const _: MeshSliceFn = slice_mesh_on_planes;
const _: MeshDispatchFn = dispatch_mesh_on_planes_with;
const _: LoopRepairFn = make_loops;
const _: fn(&LoopedLayer) -> &[crate::geometry::Polygon] = LoopedLayer::polygons;
const _: fn(IntersectionLine) -> IntersectionPoint = IntersectionLine::a;
const _: fn(IntersectionLine) -> IntersectionPoint = IntersectionLine::b;
const _: fn(IntersectionLine) -> FacetEdgeType = IntersectionLine::edge_type;
const _: fn(IntersectionPoint) -> crate::geometry::Point = IntersectionPoint::point;
const _: fn(IntersectionPoint) -> EndpointReference = IntersectionPoint::reference;
const _: fn(&[[u32; 3]]) -> Result<MeshTopology, crate::SliceError> = index_mesh_edges;
const _: fn(&MeshTopology) -> &[[u32; 3]] = MeshTopology::face_edge_ids;
const _: fn(&MeshTopology) -> u64 = MeshTopology::edge_count;

#[cfg(test)]
mod tests;
