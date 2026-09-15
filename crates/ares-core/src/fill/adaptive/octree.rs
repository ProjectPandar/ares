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
    }
    octree
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
const OCTREE_ROT: [f64; 3] = [
    5.0 * std::f64::consts::PI / 4.0,
    215.264f64.to_radians(),
    std::f64::consts::PI / 6.0,
];

/// `AngleAxisd(z, rot2) * AngleAxisd(y, rot1) * AngleAxisd(x, rot0)`
pub(crate) fn octree_rotation_to_world() -> Matrix3 {
    mat_mul(
        mat_mul(rotate_axis(2, OCTREE_ROT[2]), rotate_axis(1, OCTREE_ROT[1])),
        rotate_axis(0, OCTREE_ROT[0]),
    )
}

/// `AngleAxisd(-rot0, x) * AngleAxisd(-rot1, y) * AngleAxisd(-rot2, z)`
fn octree_rotation_to_octree() -> Matrix3 {
    mat_mul(
        mat_mul(
            rotate_axis(0, -OCTREE_ROT[0]),
            rotate_axis(1, -OCTREE_ROT[1]),
        ),
        rotate_axis(2, -OCTREE_ROT[2]),
    )
}

pub(crate) type Matrix3 = [[f64; 3]; 3];

pub(crate) fn mat_mul(a: Matrix3, b: Matrix3) -> Matrix3 {
    let mut out = [[0.0; 3]; 3];
    for (r, row) in a.iter().enumerate() {
        for c in 0..3 {
            out[r][c] = row[0] * b[0][c] + row[1] * b[1][c] + row[2] * b[2][c];
        }
    }
    out
}

pub(crate) fn rotate_point(m: Matrix3, p: [f64; 3]) -> [f64; 3] {
    [dot(m[0], p), dot(m[1], p), dot(m[2], p)]
}

fn rotate_axis(axis: usize, angle: f64) -> Matrix3 {
    let (s, c) = angle.sin_cos();
    match axis {
        0 => [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]],
        1 => [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]],
        _ => [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]],
    }
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn abs(a: [f64; 3]) -> [f64; 3] {
    a.map(f64::abs)
}

/// `FillAdaptive.cpp:42-160` — Ericson's SAT triangle/AABB test,
/// unrolled cross-axis form (`FillAdaptive.cpp:84-160`).
fn triangle_aabb_intersects(
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    aabb: ([f64; 3], [f64; 3]),
) -> bool {
    let t_min = [
        a[0].min(b[0]).min(c[0]),
        a[1].min(b[1]).min(c[1]),
        a[2].min(b[2]).min(c[2]),
    ];
    let t_max = [
        a[0].max(b[0]).max(c[0]),
        a[1].max(b[1]).max(c[1]),
        a[2].max(b[2]).max(c[2]),
    ];
    if t_min[0] >= aabb.1[0]
        || t_max[0] <= aabb.0[0]
        || t_min[1] >= aabb.1[1]
        || t_max[1] <= aabb.0[1]
        || t_min[2] >= aabb.1[2]
        || t_max[2] <= aabb.0[2]
    {
        return false;
    }

    let center = [
        (aabb.0[0] + aabb.1[0]) * 0.5,
        (aabb.0[1] + aabb.1[1]) * 0.5,
        (aabb.0[2] + aabb.1[2]) * 0.5,
    ];
    let h = [
        aabb.1[0] - center[0],
        aabb.1[1] - center[1],
        aabb.1[2] - center[2],
    ];

    let t = [sub(b, a), sub(c, a), sub(c, b)];
    let ac = sub(a, center);
    let bc = sub(b, center);
    let cc = sub(c, center);

    let n = cross(t[0], t[1]);
    let s = dot(n, ac);
    let an = abs(n);
    let r0 = h[0] * an[0] + h[1] * an[1] + h[2] * an[2];
    if s.abs() >= r0 {
        return false;
    }

    let at = [abs(t[0]), abs(t[1]), abs(t[2])];

    // Each test: `if (r + |tc - d1| < |tc|) return false;`
    let sat = |d1: f64, d2: f64, r: f64| -> bool {
        let tc = (d1 + d2) * 0.5;
        !(r + (tc - d1).abs() < tc.abs())
    };

    // eX × t[0], eX × t[1], eX × t[2]
    for (ti, (p1, p2)) in [(0usize, (ac, cc)), (1, (ac, bc)), (2, (ac, bc))] {
        let d1 = t[ti][1] * p1[2] - t[ti][2] * p1[1];
        let d2 = t[ti][1] * p2[2] - t[ti][2] * p2[1];
        let r = (h[1] * at[ti][2] + h[2] * at[ti][1]).abs();
        if !sat(d1, d2, r) {
            return false;
        }
    }
    // eY × t[0..2]
    for (ti, (p1, p2)) in [(0usize, (ac, cc)), (1, (ac, bc)), (2, (ac, bc))] {
        let d1 = t[ti][2] * p1[0] - t[ti][0] * p1[2];
        let d2 = t[ti][2] * p2[0] - t[ti][0] * p2[2];
        let r = (h[0] * at[ti][2] + h[2] * at[ti][0]).abs();
        if !sat(d1, d2, r) {
            return false;
        }
    }
    // eZ × t[0..2]
    for (ti, (p1, p2)) in [(0usize, (ac, cc)), (1, (ac, bc)), (2, (ac, bc))] {
        let d1 = t[ti][0] * p1[1] - t[ti][1] * p1[0];
        let d2 = t[ti][0] * p2[1] - t[ti][1] * p2[0];
        let r = (h[1] * at[ti][0] + h[0] * at[ti][1]).abs();
        if !sat(d1, d2, r) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests;
