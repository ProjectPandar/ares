//! TriangleSelector.cpp::deserialize / get_facets / get_facets_strict.

use crate::{ProjectMesh, SliceError, mesh_slicer::index_mesh_edges};

mod neighbors;
mod split;
mod strict;

#[derive(Clone, Copy)]
struct Triangle {
    vertices: [u32; 3],
    code: u8,
    state: u8,
    children: [usize; 4],
}

impl Triangle {
    fn new(vertices: [u32; 3]) -> Self {
        Self {
            vertices,
            code: 0,
            state: 0,
            children: [0; 4],
        }
    }

    fn splits(self) -> usize {
        usize::from(self.code & 3)
    }

    fn special(self) -> usize {
        usize::from(self.code >> 2)
    }
}

pub(crate) struct SelectedFacets {
    pub(crate) vertices: Vec<[f32; 3]>,
    pub(crate) triangles: Vec<[u32; 3]>,
}

struct Selector {
    vertices: Vec<[f32; 3]>,
    triangles: Vec<Triangle>,
}

enum DecodeWork {
    Read(usize, [Option<usize>; 3]),
    Child(usize, [Option<usize>; 3], usize),
}

fn decode_step(
    selector: &mut Selector,
    codes: &[u8],
    cursor: &mut usize,
    work: DecodeWork,
    pending: &mut Vec<DecodeWork>,
) {
    let (index, neighbors) = match work {
        DecodeWork::Read(index, neighbors) => (index, neighbors),
        DecodeWork::Child(parent, neighbors, child) => {
            let triangle = selector.triangles[parent];
            let child_neighbors = selector.child_neighbors(triangle, neighbors, child);
            if child != 0 {
                pending.push(DecodeWork::Child(parent, neighbors, child - 1));
            }
            pending.push(DecodeWork::Read(triangle.children[child], child_neighbors));
            return;
        }
    };
    let code = codes[*cursor];
    *cursor += 1;
    selector.triangles[index].code = code;
    if code & 3 != 0 {
        selector.perform_split(index, neighbors);
        let triangle = selector.triangles[index];
        pending.push(DecodeWork::Child(index, neighbors, triangle.splits()));
        return;
    }
    selector.triangles[index].state = if code >> 2 == 3 {
        let state = codes[*cursor] + 3;
        *cursor += 1;
        state
    } else {
        code >> 2
    };
}

fn export_step(
    selector: &Selector,
    frame: (usize, [Option<usize>; 3]),
    state: u8,
    output: &mut Vec<[u32; 3]>,
    pending: &mut Vec<(usize, [Option<usize>; 3])>,
) {
    let (index, neighbors) = frame;
    let triangle = selector.triangles[index];
    if triangle.splits() == 0 {
        if triangle.state == state {
            selector.split_tjoints(triangle.vertices, neighbors, output);
        }
        return;
    }
    for child in (0..=triangle.splits()).rev() {
        pending.push((
            triangle.children[child],
            selector.child_neighbors(triangle, neighbors, child),
        ));
    }
}

type FaceNeighbors = Vec<[Option<usize>; 3]>;

fn deserialize(
    mesh: &ProjectMesh,
    annotation: &super::FacetsAnnotation,
) -> Result<(Selector, FaceNeighbors), SliceError> {
    let topology = index_mesh_edges(mesh.triangles())?;
    let face_neighbors = topology.build_face_neighbors();
    let mut selector = Selector {
        vertices: mesh
            .vertices()
            .iter()
            .map(|point| [point.x as f32, point.y as f32, point.z as f32])
            .collect(),
        triangles: mesh
            .triangles()
            .iter()
            .copied()
            .map(Triangle::new)
            .collect(),
    };
    selector.triangles.reserve(annotation.codes.len());
    let mut decode = Vec::new();
    for &(root, offset) in &annotation.mapping {
        let mut cursor = offset;
        decode.push(DecodeWork::Read(root, face_neighbors[root]));
        while let Some(work) = decode.pop() {
            decode_step(
                &mut selector,
                &annotation.codes,
                &mut cursor,
                work,
                &mut decode,
            );
        }
    }
    Ok((selector, face_neighbors))
}

pub(crate) fn strict_facets(mesh: &ProjectMesh, state: u8) -> Result<SelectedFacets, SliceError> {
    strict_annotated_facets(mesh, &mesh.supported_facets, state)
}

pub(crate) fn strict_annotated_facets(
    mesh: &ProjectMesh,
    annotation: &super::FacetsAnnotation,
    state: u8,
) -> Result<SelectedFacets, SliceError> {
    let (selector, face_neighbors) = deserialize(mesh, annotation)?;
    let mut triangles = Vec::new();
    let mut stack = Vec::new();
    for (root, &neighbors) in face_neighbors.iter().enumerate() {
        stack.push((root, neighbors));
        while let Some(frame) = stack.pop() {
            export_step(&selector, frame, state, &mut triangles, &mut stack);
        }
    }
    // get_facets_strict retains vertices referenced by all selector leaves,
    // not just vertices referenced by this selected state.
    let mut referenced = vec![false; selector.vertices.len()];
    for triangle in &selector.triangles {
        if triangle.splits() == 0 {
            for vertex in triangle.vertices {
                referenced[vertex as usize] = true;
            }
        }
    }
    let mut mapping = vec![0; selector.vertices.len()];
    let mut vertices = Vec::new();
    for (index, point) in selector.vertices.into_iter().enumerate() {
        if referenced[index] {
            mapping[index] = vertices.len() as u32;
            vertices.push(point);
        }
    }
    for triangle in &mut triangles {
        *triangle = triangle.map(|vertex| mapping[vertex as usize]);
    }
    Ok(SelectedFacets {
        vertices,
        triangles,
    })
}

/// TriangleSelector.cpp1474–1493: ordinary leaves, in selector storage order.
pub(crate) fn ordinary_facets(
    mesh: &ProjectMesh,
    annotation: &super::FacetsAnnotation,
    state: u8,
) -> Result<SelectedFacets, SliceError> {
    let (selector, _) = deserialize(mesh, annotation)?;
    let mut mapping = vec![u32::MAX; selector.vertices.len()];
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for triangle in selector
        .triangles
        .iter()
        .filter(|triangle| triangle.splits() == 0 && triangle.state == state)
    {
        triangles.push(triangle.vertices.map(|index| {
            let index = index as usize;
            if mapping[index] == u32::MAX {
                mapping[index] = vertices.len() as u32;
                vertices.push(selector.vertices[index]);
            }
            mapping[index]
        }));
    }
    Ok(SelectedFacets {
        vertices,
        triangles,
    })
}
