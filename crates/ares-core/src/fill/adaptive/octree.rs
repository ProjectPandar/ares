//! OrcaSlicer 2.4.2 `Fill/FillAdaptive.cpp` octree construction
//! (`build_octree`, `make_cubes_properties`, `Octree::insert_triangle`;
//! lines 225-272, 1434-1570).
//!
//! Slice 1 of the FillAdaptive port: the data structure and build only.
//! Line generation (`Filler::fill_surface`) is slice 2; option wiring is
//! slice 3.

/// `FillAdaptive.cpp:238-246`
#[derive(Debug)]
pub(crate) struct Cube {
    center: [f64; 3],
    children: [Option<Box<Cube>>; 8],
}

impl Cube {
    fn new(center: [f64; 3]) -> Self {
        Self {
            center,
            children: [const { None }; 8],
        }
    }
}

/// `FillAdaptive.cpp:248-255`
#[derive(Clone, Copy, Debug)]
pub(crate) struct CubeProperties {
    pub(crate) edge_length: f64,
    pub(crate) height: f64,
    pub(crate) diagonal_length: f64,
    pub(crate) line_z_distance: f64,
    pub(crate) line_xy_distance: f64,
}

/// `FillAdaptive.cpp:257-269` — the pool allocation is replaced by `Box`.
#[derive(Debug)]
pub(crate) struct Octree {
    root_cube: Box<Cube>,
    pub(crate) origin: [f64; 3],
    pub(crate) cubes_properties: Vec<CubeProperties>,
}

mod frame;

use frame::{
    Matrix3, abs, cross, dot, norm, octree_rotation_to_world, sub, triangle_aabb_intersects,
};
pub(crate) use frame::{octree_rotation_to_octree, rotate_point};

impl Octree {
    pub(crate) fn root(&self) -> &Cube {
        &self.root_cube
    }
}

impl Cube {
    pub(crate) fn center(&self) -> [f64; 3] {
        self.center
    }

    pub(crate) fn child(&self, index: usize) -> Option<&Cube> {
        self.children[index].as_deref()
    }
}

/// `FillAdaptive.cpp:226-229` — ordering of children cubes.
const CHILD_CENTERS: [[f64; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0],
];

/// `FillAdaptive.cpp:401` — `EPSILON` from libslic3r.h (SCALED_EPSILON is
/// finer; the octree uses the coarse 1e-4).
const EPSILON: f64 = 1e-4;

/// `FillAdaptive.cpp:1434-1462` — cube ladder from the line spacing up to
/// the object bounding box; the Orca two-level minimum is kept.
pub(crate) fn make_cubes_properties(
    max_cube_edge_length: f64,
    line_spacing: f64,
) -> Vec<CubeProperties> {
    let max_cube_edge_length = max_cube_edge_length + EPSILON;
    let mut cubes_properties = Vec::new();
    let mut edge_length = line_spacing * 2.0;
    loop {
        cubes_properties.push(CubeProperties {
            edge_length,
            height: edge_length * 3.0f64.sqrt(),
            diagonal_length: edge_length * 2.0f64.sqrt(),
            line_z_distance: edge_length / 3.0f64.sqrt(),
            line_xy_distance: edge_length / 6.0f64.sqrt(),
        });
        if edge_length > max_cube_edge_length {
            break;
        }
        edge_length *= 2.0;
    }
    // Orca: Ensure at least 2 levels so build_octree() will insert
    // triangles (fixes disconnected adaptive fill at low densities).
    if cubes_properties.len() == 1 {
        let mut p = *cubes_properties.last().unwrap();
        p.edge_length *= 2.0;
        p.height = p.edge_length * 3.0f64.sqrt();
        p.diagonal_length = p.edge_length * 2.0f64.sqrt();
        p.line_z_distance = p.edge_length / 3.0f64.sqrt();
        p.line_xy_distance = p.edge_length / 6.0f64.sqrt();
        cubes_properties.push(p);
    }
    cubes_properties
}

/// `FillAdaptive.cpp:1483-1531` — mesh already rotated into octree
/// coordinates by the caller (`PrintObject.cpp:994-996`).
pub(crate) fn build_octree(
    mesh_vertices: &[[f64; 3]],
    mesh_triangles: &[[u32; 3]],
    overhang_triangles: &[[f64; 3]],
    line_spacing: f64,
    support_overhangs_only: bool,
    // The XY world-frame bbox center the caller subtracted before
    // rotating the vertices (upstream `m_center_offset`,
    // PrintObject.cpp:110); added back to the rotated cube centers.
    world_offset: [f64; 3],
) -> Octree {
    let mut bbox_min = [f64::INFINITY; 3];
    let mut bbox_max = [f64::NEG_INFINITY; 3];
    for v in mesh_vertices {
        for k in 0..3 {
            bbox_min[k] = bbox_min[k].min(v[k]);
            bbox_max[k] = bbox_max[k].max(v[k]);
        }
    }
    let cube_center = [
        0.5 * (bbox_min[0] + bbox_max[0]),
        0.5 * (bbox_min[1] + bbox_max[1]),
        0.5 * (bbox_min[2] + bbox_max[2]),
    ];
    let bbox_size = (bbox_max[0] - bbox_min[0])
        .max(bbox_max[1] - bbox_min[1])
        .max(bbox_max[2] - bbox_min[2]);
    let cubes_properties = make_cubes_properties(bbox_size, line_spacing);
    let mut octree = Octree {
        root_cube: Box::new(Cube::new(cube_center)),
        origin: cube_center,
        cubes_properties,
    };

    if octree.cubes_properties.len() > 1 {
        let edge_length_half = 0.5 * octree.cubes_properties.last().unwrap().edge_length;
        let max_depth = octree.cubes_properties.len() as i32 - 1;
        let root_center = octree.root_cube.center;
        let root_bbox = (
            [
                root_center[0] - edge_length_half,
                root_center[1] - edge_length_half,
                root_center[2] - edge_length_half,
            ],
            [
                root_center[0] + edge_length_half,
                root_center[1] + edge_length_half,
                root_center[2] + edge_length_half,
            ],
        );
        // `FillAdaptive.cpp:1512` — the up vector in octree coordinates
        // (inverse of the world rotation applied to +Z).
        let up_vector = if support_overhangs_only {
            rotate_point(octree_rotation_to_octree(), [0.0, 0.0, 1.0])
        } else {
            [0.0, 0.0, 0.0]
        };
        for tri in mesh_triangles {
            let a = mesh_vertices[tri[0] as usize];
            let b = mesh_vertices[tri[1] as usize];
            let c = mesh_vertices[tri[2] as usize];
            if !support_overhangs_only || is_overhang_triangle(a, b, c, up_vector) {
                insert_triangle(
                    &octree.cubes_properties,
                    a,
                    b,
                    c,
                    &mut octree.root_cube,
                    root_bbox,
                    max_depth,
                );
            }
        }
        for chunk in overhang_triangles.chunks_exact(3) {
            insert_triangle(
                &octree.cubes_properties,
                chunk[0],
                chunk[1],
                chunk[2],
                &mut octree.root_cube,
                root_bbox,
                max_depth,
            );
        }
        // `FillAdaptive.cpp:1524-1528` — rotate the centers back to world
        // coordinates and add the centering translation back so the cube
        // centers share the frame of the layer surfaces.
        let rot = octree_rotation_to_world();
        let offset = world_offset;
        transform_center(&mut octree.root_cube, rot, offset);
        octree.origin = rotate_point(rot, octree.origin);
    }
    octree
}

/// `FillAdaptive.cpp:1472-1480` — recursive center transform, plus the
/// centering translation the upstream build applies through the mesh
/// transform (the caller passed centered vertices).
fn transform_center(cube: &mut Cube, rot: Matrix3, offset: [f64; 3]) {
    let rotated = rotate_point(rot, cube.center);
    cube.center = [
        rotated[0] + offset[0],
        rotated[1] + offset[1],
        rotated[2] + offset[2],
    ];
    for child in &mut cube.children {
        if let Some(child) = child {
            transform_center(child, rot, offset);
        }
    }
}

/// `FillAdaptive.cpp:1533-1566` — recursive subdivision; the expanded
/// child bbox (±EPSILON) and the SAT triangle-AABB test are upstream.
fn insert_triangle(
    properties: &[CubeProperties],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    current_cube: &mut Cube,
    current_bbox: ([f64; 3], [f64; 3]),
    depth: i32,
) {
    let depth = depth - 1;

    for (i, child_center_dir) in CHILD_CENTERS.iter().enumerate() {
        let mut bbox = ([0.0; 3], [0.0; 3]);
        for k in 0..3 {
            if child_center_dir[k] == -1.0 {
                bbox.0[k] = current_bbox.0[k];
                bbox.1[k] = current_cube.center[k] + EPSILON;
            } else {
                bbox.0[k] = current_cube.center[k] - EPSILON;
                bbox.1[k] = current_bbox.1[k];
            }
        }
        let half = properties[depth as usize].edge_length / 2.0;
        let child_center = [
            current_cube.center[0] + child_center_dir[0] * half,
            current_cube.center[1] + child_center_dir[1] * half,
            current_cube.center[2] + child_center_dir[2] * half,
        ];
        if triangle_aabb_intersects(a, b, c, bbox) {
            let child =
                current_cube.children[i].get_or_insert_with(|| Box::new(Cube::new(child_center)));
            if depth > 0 {
                insert_triangle(properties, a, b, c, child, bbox, depth);
            }
        }
    }
}

/// `FillAdaptive.cpp:1465-1469`
fn is_overhang_triangle(a: [f64; 3], b: [f64; 3], c: [f64; 3], up: [f64; 3]) -> bool {
    let n = cross(sub(b, a), sub(c, b));
    dot(n, up) > 0.707 * norm(n)
}

/// `FillAdaptive.cpp:401-412` — the fixed octree frame rotations.
#[cfg(test)]
mod tests;
