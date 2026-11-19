//! `libslic3r/LayOnFace.cpp:21–242`: hull planes, face choice, and instance drop.

use std::collections::HashMap;

use super::{
    Point3d, Project, ProjectObject, ProjectVolumeType, Transform3d, placement::world_vertices,
};
use crate::{
    SliceError,
    geometry::{ConvexHull3d, convex_hull_3d, mesh_normals},
    plater::{rotation_from_two_vectors, rotation_matrix_transform},
};

const EPSILON: f64 = 1e-4; // libslic3r.h:57
const UP: [f64; 3] = [0., 0., 1.];
const DOWN: [f64; 3] = [0., 0., -1.];
type Vec3 = [f64; 3];

pub(crate) enum FaceChoice {
    Largest,
    Normal(Vec3),
    Point(Vec3),
}

impl FaceChoice {
    pub(crate) fn parse(key: &str, raw: &str, normal: bool) -> Result<Self, SliceError> {
        let invalid = || {
            SliceError::InvalidCliGroundFace(format!(
                "{key} expects three comma-separated numbers, got \"{raw}\""
            ))
        };
        let mut tokens = raw.split(',');
        let mut point = [0.; 3];
        for value in &mut point {
            *value = tokens
                .next()
                .and_then(|token| {
                    token
                        .trim_start_matches(|ch: char| ch.is_ascii_whitespace())
                        .parse::<f64>()
                        .ok()
                })
                .filter(|v| v.is_finite())
                .ok_or_else(invalid)?;
        }
        if tokens.next().is_some() || (normal && norm(point) < EPSILON) {
            return Err(invalid());
        }
        Ok(if normal {
            Self::Normal(unit(point))
        } else {
            Self::Point(point)
        })
    }
}

impl Project {
    pub(crate) fn ground_cli_face(&mut self, choice: FaceChoice) -> Result<(), SliceError> {
        let (mut laid, mut missed) = (false, false);
        for object in &mut self.objects {
            let (object_laid, object_missed) = ground_object(object, &choice);
            laid |= object_laid;
            missed |= object_missed;
        }
        if !laid && missed {
            return Err(SliceError::InvalidCliGroundFace(
                "ground_face_point: no face of any object contains the point".into(),
            ));
        }
        Ok(())
    }
}

fn ground_object(object: &mut ProjectObject, choice: &FaceChoice) -> (bool, bool) {
    let Some(hull) = object_hull(object) else {
        return (false, false);
    };
    let faces = hull_faces(&hull);
    let (mut laid, mut missed) = (false, false);
    for index in 0..object.instances.len() {
        let transform = object.instances[index].transform;
        let linear = transform.without_translation();
        let planes = instance_planes(&faces, &hull, linear);
        if planes.is_empty() {
            continue;
        }
        let Some(plane) = pick(&planes, choice, linear) else {
            missed = true;
            continue;
        };
        let rotation = if dot(plane.instance_normal, DOWN) < -1. + 1e-9 {
            let (sin, cos) = std::f64::consts::PI.sin_cos();
            rotation_matrix_transform([[1., 0., 0.], [0., cos, -sin], [0., sin, cos]])
        } else {
            from_to(plane.instance_normal, DOWN)
        };
        let offset = transform.transform_point(Point3d::new(0., 0., 0.));
        let rotated = rotation.then(linear).pretranslated(offset);
        let min_z = world_vertices(object, rotated)
            .map(|p| p.z)
            .fold(f64::INFINITY, f64::min);
        object.instances[index].transform = rotated.pretranslated(Point3d::new(0., 0., -min_z));
        laid = true;
    }
    (laid, missed)
}

fn object_hull(object: &ProjectObject) -> Option<ConvexHull3d> {
    // The hull of the union of part vertices equals the hull of the union of their hulls.
    let vertices: Vec<_> = object
        .volumes
        .iter()
        .filter(|v| v.volume_type == ProjectVolumeType::ModelPart)
        .flat_map(|v| {
            v.mesh
                .vertices()
                .iter()
                .map(|&p| v.transform.transform_point_f32(p))
        })
        .collect();
    convex_hull_3d(&vertices)
}

struct Face {
    normal: Vec3,
    vertices: Vec<u32>,
}

fn hull_faces(hull: &ConvexHull3d) -> Vec<Face> {
    let normals: Vec<_> = hull
        .facets
        .iter()
        .map(|facet| {
            let [a, b, c] = facet.map(|id| hull.vertices[id as usize]);
            mesh_normals::face_normal(a, b, c)
        })
        .collect();
    let mut neighbors = vec![[None; 3]; hull.facets.len()];
    let mut edges = HashMap::new();
    for (index, facet) in hull.facets.iter().enumerate() {
        for slot in 0..3 {
            let a = facet[slot];
            let b = facet[(slot + 1) % 3];
            if let Some((other, other_slot)) = edges.remove(&(b, a)) {
                neighbors[index][slot] = Some(other);
                let other_edges: &mut [Option<usize>; 3] = &mut neighbors[other];
                other_edges[other_slot] = Some(index);
            } else {
                edges.insert((a, b), (index, slot));
            }
        }
    }
    let mut visited = vec![false; hull.facets.len()];
    let mut faces = Vec::new();
    let mut stack = Vec::new();
    for (start, seed) in normals.iter().enumerate() {
        if visited[start] {
            continue;
        }
        let mut vertices = Vec::new();
        stack.push(start);
        while let Some(index) = stack.pop() {
            if visited[index]
                || !(0..3).all(|i| f64::from((normals[index][i] - seed[i]).abs()) < 0.001)
            {
                continue;
            }
            visited[index] = true;
            vertices.extend(hull.facets[index]);
            stack.extend(neighbors[index].into_iter().flatten());
        }
        vertices.sort_unstable();
        vertices.dedup();
        faces.push(Face {
            normal: seed.map(f64::from),
            vertices,
        });
    }
    faces
}

struct Plane {
    normal: Vec3,
    instance_normal: Vec3,
    area: f32,
    frame: Transform3d,
    outline: Vec<Vec3>,
}

fn instance_planes(faces: &[Face], hull: &ConvexHull3d, linear: Transform3d) -> Vec<Plane> {
    let determinant = linear.linear_determinant();
    if determinant == 0. {
        return Vec::new();
    }
    let columns: [Vec3; 3] =
        std::array::from_fn(|c| std::array::from_fn(|r| linear.as_matrix()[r][c]));
    // Columns of the inverse transpose (before division by the determinant).
    let cofactors: [Vec3; 3] =
        std::array::from_fn(|i| cross(columns[(i + 1) % 3], columns[(i + 2) % 3]));
    let mut planes: Vec<_> = faces
        .iter()
        .filter_map(|face| {
            let instance_normal = unit(std::array::from_fn(|r| {
                (cofactors[0][r] * face.normal[0]
                    + cofactors[1][r] * face.normal[1]
                    + cofactors[2][r] * face.normal[2])
                    / determinant
            }));
            let frame = from_to(instance_normal, UP);
            let mut points: Vec<_> = face
                .vertices
                .iter()
                .map(|&i| transform(linear, hull.vertices[i as usize].map(f64::from)))
                .collect();
            if let [a, b, c] = points[..]
                && [sub(a, b), sub(b, c), sub(c, a)]
                    .into_iter()
                    .any(|edge| norm(edge) < 1.)
            {
                return None;
            }
            for point in &mut points {
                *point = transform(frame, *point);
            }
            let outline = outline_hull(points);
            let edges = outline
                .iter()
                .copied()
                .zip(outline.iter().copied().cycle().skip(1));
            let area = edges
                .clone()
                .fold(0f32, |sum, (a, b)| (f64::from(sum) + cross_xy(a, b)) as f32)
                .abs()
                * 0.5;
            if area < 5.
                || edges
                    .clone()
                    .zip(edges.cycle().skip(1))
                    .any(|((a, b), (_, c))| {
                        dot(unit(sub(a, b)), unit(sub(c, b))) > 1f64.to_radians().cos()
                    })
            {
                return None;
            }
            Some(Plane {
                normal: face.normal,
                instance_normal,
                area,
                frame,
                outline,
            })
        })
        .collect();
    planes.sort_by(|a, b| b.area.total_cmp(&a.area));
    planes
}

// `Geometry/ConvexHull.cpp:50–105`: monotone chain with fixed-point XY tests,
// keeping the original (rescaled) 3D points for the plane-distance test.
fn outline_hull(mut points: Vec<Vec3>) -> Vec<Vec3> {
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    let width = points[points.len() - 1][0] - points[0][0];
    let (low, high) = points
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p[1]), hi.max(p[1]))
        });
    let scale = f64::from((1. / width.max(high - low)) as f32);
    for point in &mut points {
        point[0] *= scale;
        point[1] *= scale;
    }
    let fixed: Vec<_> = points
        .iter()
        .map(|p| [(p[0] * 1e6) as i64, (p[1] * 1e6) as i64])
        .collect();
    let ccw = |a: usize, b: usize, c: usize| {
        let [[ax, ay], [bx, by], [cx, cy]] = [fixed[a], fixed[b], fixed[c]];
        (i128::from(bx) - i128::from(ax)) * (i128::from(cy) - i128::from(ay))
            > (i128::from(by) - i128::from(ay)) * (i128::from(cx) - i128::from(ax))
    };
    let mut outline = Vec::with_capacity(points.len());
    let count = points.len();
    for reverse in [false, true] {
        let floor = if reverse { outline.len() + 1 } else { 2 };
        for i in 0..count - usize::from(reverse) {
            let index = if reverse { count - 2 - i } else { i };
            while outline.len() >= floor
                && !ccw(
                    outline[outline.len() - 2],
                    outline[outline.len() - 1],
                    index,
                )
            {
                outline.pop();
            }
            outline.push(index);
        }
    }
    outline.pop();
    outline
        .into_iter()
        .map(|i| [points[i][0] / scale, points[i][1] / scale, points[i][2]])
        .collect()
}

fn pick<'a>(planes: &'a [Plane], choice: &FaceChoice, linear: Transform3d) -> Option<&'a Plane> {
    let first = &planes[0];
    match choice {
        FaceChoice::Largest => Some(
            planes
                .iter()
                .take_while(|p| f64::from(p.area) >= f64::from(first.area) * (1. - 1e-3))
                .fold(first, |best, p| {
                    if p.instance_normal[2] < best.instance_normal[2] {
                        p
                    } else {
                        best
                    }
                }),
        ),
        FaceChoice::Normal(direction) => Some(planes.iter().fold(first, |best, p| {
            if dot(p.normal, *direction) > dot(best.normal, *direction) {
                p
            } else {
                best
            }
        })),
        FaceChoice::Point(point) => {
            let point = transform(linear, *point);
            planes.iter().find(|plane| contains(plane, point))
        }
    }
}

fn contains(plane: &Plane, point: Vec3) -> bool {
    const TOLERANCE: f64 = 0.01; // OrcaSlicer.cpp:5107
    let p = transform(plane.frame, point);
    let z = plane.outline.iter().map(|point| point[2]).sum::<f64>() / plane.outline.len() as f64;
    if (p[2] - z).abs() > TOLERANCE {
        return false;
    }
    let (mut left, mut right) = (false, false);
    for (&a, &b) in plane
        .outline
        .iter()
        .zip(plane.outline.iter().cycle().skip(1))
    {
        let edge = sub(b, a);
        let len = edge[0].hypot(edge[1]);
        if len < EPSILON {
            continue;
        }
        let side = cross_xy(edge, sub(p, a)) / len;
        left |= side > TOLERANCE;
        right |= side < -TOLERANCE;
        if left && right {
            return false;
        }
    }
    true
}

fn from_to(from: Vec3, to: Vec3) -> Transform3d {
    rotation_matrix_transform(rotation_from_two_vectors(from, to).2)
}

fn transform(matrix: Transform3d, [x, y, z]: Vec3) -> Vec3 {
    let p = matrix.transform_point(Point3d::new(x, y, z));
    [p.x, p.y, p.z]
}

fn cross_xy(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[(i + 1) % 3] * b[(i + 2) % 3] - a[(i + 2) % 3] * b[(i + 1) % 3])
}
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + (a[1] * b[1] + a[2] * b[2])
}
fn norm(v: Vec3) -> f64 {
    dot(v, v).sqrt()
}
fn unit(v: Vec3) -> Vec3 {
    let length = norm(v);
    if length > 0. {
        v.map(|x| x / length)
    } else {
        v
    }
}
