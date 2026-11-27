use super::{
    camera::{self, Camera, Vector},
    light::{self, Intensity},
    raster::{Raster, Vertex},
};
use crate::{Point3d, Transform3d};

type Position = [f64; 3];
type Indices = [u32; 3];

pub(super) struct Mesh {
    world: Vec<Vector>,
    faces: Vec<Face>,
    color: [u8; 4],
}

struct Face {
    vertices: Indices,
    normals: [Vector; 3],
}

impl Mesh {
    pub(super) fn new(
        local: &[Point3d],
        world: Vec<Vector>,
        indices: &[Indices],
        transform: Transform3d,
        color: [u8; 4],
    ) -> Self {
        let mirror = transform.linear_determinant().is_sign_negative();
        let normal_matrix = normal_matrix(transform, mirror);
        let mut vertex_faces = vec![Vec::new(); local.len()];
        let base = indices
            .iter()
            .enumerate()
            .map(|(face, indices)| {
                let [a, b, c] = indices.map(|i| xyz(local[i as usize]));
                let area_normal = cross(sub(b, a), sub(c, a));
                let unit = normalized(area_normal)?;
                for &vertex in indices {
                    vertex_faces[vertex as usize].push(face);
                }
                Some((area_normal, unit))
            })
            .collect::<Vec<_>>();

        // GLModel.cpp:447-459 and TriangleMesh.cpp:1634-1718: preserve 5° creases and weight each corner by incident face area.
        // Summing neighbors in source order produces the same result regardless of worker count.
        let faces = indices
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(face, mut vertices)| {
                let (_, unit) = base[face]?;
                let mut normals = vertices
                    .map(|v| corner_normal(&base, &vertex_faces[v as usize], unit, normal_matrix));
                if mirror {
                    vertices.swap(1, 2);
                    normals.swap(1, 2);
                }
                Some(Face { vertices, normals })
            })
            .collect();
        Self {
            world,
            faces,
            color,
        }
    }

    pub(super) fn illuminate(self, camera: &Camera) -> LitMesh {
        let vertices: Vec<_> = self.world.iter().map(|&p| (camera.view(p), p[2])).collect();
        let eyes: Vec<_> = vertices
            .iter()
            .map(|&(view, _)| light::viewer(camera.toward_eye(view)))
            .collect();
        let faces = self
            .faces
            .into_iter()
            .map(|Face { vertices, normals }| {
                let intensities = std::array::from_fn(|i| {
                    light::intensity(normals[i], eyes[vertices[i] as usize])
                });
                (vertices, intensities)
            })
            .collect();
        LitMesh {
            vertices,
            faces,
            color: self.color,
        }
    }
}

fn corner_normal(
    faces: &[Option<(Position, Position)>],
    neighbors: &[usize],
    unit: Position,
    matrix: [Position; 3],
) -> Vector {
    let mut normal = [0.0; 3];
    for &neighbor in neighbors {
        let (area, adjacent) = faces[neighbor].unwrap();
        if dot(unit, adjacent) > 0.996_194_698_091_745_5 {
            normal = std::array::from_fn(|axis| normal[axis] + area[axis]);
        }
    }
    // GLCanvas3D.cpp:7151-7154 applies the inverse transpose, then rotates to view space.
    let n = matrix.map(|row| dot(row, normal));
    let n = normalized(n).expect("printable volume has an invertible normal transform");
    camera::rotate(n.map(|component| (component * (1 << 20) as f64).round() as i64))
}

pub(super) struct LitMesh {
    vertices: Vec<(Vector, i64)>,
    faces: Vec<(Indices, [Intensity; 3])>,
    color: [u8; 4],
}

impl LitMesh {
    pub(super) fn draw(&self, camera: &Camera, dims: [u32; 2], raster: &mut Raster) {
        let project = camera.screen(dims);
        let projected: Vec<_> = self
            .vertices
            .iter()
            .map(|&(view, world_z)| Vertex {
                xy: project(view),
                depth: view[2],
                world_z,
            })
            .collect();
        for &(indices, lights) in &self.faces {
            raster.triangle(
                indices.map(|i| projected[i as usize]),
                lights,
                self.color,
                camera.depth_range,
            );
        }
    }
}

pub(super) fn transform_points(local: &[Point3d], transform: Transform3d) -> Vec<Vector> {
    local
        .iter()
        .map(|&point| xyz(transform.transform_point(point)).map(camera::world))
        .collect()
}

fn xyz(p: Point3d) -> Position {
    [p.x, p.y, p.z]
}

fn sub(a: Position, b: Position) -> Position {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: Position, b: Position) -> Position {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: Position, b: Position) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalized(v: Position) -> Option<Position> {
    let squared = dot(v, v);
    (squared.is_finite() && squared > 0.0).then(|| {
        let length = squared.sqrt();
        v.map(|x| x / length)
    })
}

fn normal_matrix(transform: Transform3d, mirror: bool) -> [Position; 3] {
    let matrix = transform.as_matrix();
    let [a, b, c] = std::array::from_fn(|column| std::array::from_fn(|row| matrix[row][column]));
    // The absolute determinant divides every component equally and cancels at normalization.
    let cofactors = [cross(b, c), cross(c, a), cross(a, b)];
    let sign = if mirror { -1.0 } else { 1.0 };
    std::array::from_fn(|row| std::array::from_fn(|column| cofactors[column][row] * sign))
}
