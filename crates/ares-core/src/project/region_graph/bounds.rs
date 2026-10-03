use crate::{ProjectMesh, Transform3d};

const EPSILON: f32 = 1e-4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoundingBox3f {
    min: [f32; 3],
    max: [f32; 3],
}

impl BoundingBox3f {
    pub(crate) const fn min(self) -> [f32; 3] {
        self.min
    }
    pub(crate) const fn max(self) -> [f32; 3] {
        self.max
    }
    pub(crate) fn contains_z(self, z: f32) -> bool {
        self.min[2] <= z && z <= self.max[2]
    }
    pub(crate) fn intersects_xy(self, other: Self) -> bool {
        !(self.max[0] < other.min[0]
            || other.max[0] < self.min[0]
            || self.max[1] < other.min[1]
            || other.max[1] < self.min[1])
    }
    pub(crate) fn intersects(self, other: Self) -> bool {
        self.intersects_xy(other) && !(self.max[2] < other.min[2] || other.max[2] < self.min[2])
    }
    pub(crate) fn extend(&mut self, other: Self) {
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(other.min[axis]);
            self.max[axis] = self.max[axis].max(other.max[axis]);
        }
    }
    fn point(point: [f32; 3]) -> Self {
        Self {
            min: point,
            max: point,
        }
    }
    fn inflate(&mut self, offset: f32) {
        for (axis, amount) in [offset, offset, EPSILON].into_iter().enumerate() {
            self.min[axis] -= amount;
            self.max[axis] += amount;
        }
    }
}

pub(crate) fn transformed_mesh_bbox(
    mesh: &ProjectMesh,
    transform: Transform3d,
    offset: f32,
) -> BoundingBox3f {
    let first = mesh.vertices()[mesh.triangles()[0][0] as usize];
    let mut bbox = BoundingBox3f::point(transform.transform_point_f32(first));
    for &index in mesh.triangles().iter().flatten() {
        bbox.extend(BoundingBox3f::point(
            transform.transform_point_f32(mesh.vertices()[index as usize]),
        ));
    }
    bbox.inflate(offset);
    bbox
}

pub(super) fn transformed_mesh_range_bboxes(
    mesh: &ProjectMesh,
    transform: Transform3d,
    ranges: &[(f64, f64)],
    offset: f32,
) -> Vec<Option<BoundingBox3f>> {
    let mut boxes = vec![None; ranges.len()];
    for triangle in mesh.triangles() {
        let points =
            triangle.map(|index| transform.transform_point_f32(mesh.vertices()[index as usize]));
        for (&range, bbox) in ranges.iter().zip(&mut boxes) {
            for (previous, current) in [(2, 0), (0, 1), (1, 2)] {
                extend_edge(bbox, points[previous], points[current], range);
            }
        }
    }
    for bbox in boxes.iter_mut().flatten() {
        bbox.inflate(offset);
    }
    boxes
}

fn extend_point(bbox: &mut Option<BoundingBox3f>, point: [f32; 3]) {
    let point = BoundingBox3f::point(point);
    if let Some(bbox) = bbox {
        bbox.extend(point);
    } else {
        *bbox = Some(point);
    }
}

fn intersection(p1: [f32; 3], p2: [f32; 3], z: f64) -> [f32; 3] {
    // PrintApply.cpp narrows the float Z difference before double division.
    let span = p2[2] - p1[2];
    let t = ((z - f64::from(p1[2])) / f64::from(span)) as f32;
    [
        p1[0] + (p2[0] - p1[0]) * t,
        p1[1] + (p2[1] - p1[1]) * t,
        z as f32,
    ]
}

fn extend_edge(
    bbox: &mut Option<BoundingBox3f>,
    mut p1: [f32; 3],
    mut p2: [f32; 3],
    range: (f64, f64),
) {
    if p1[2] > p2[2] {
        std::mem::swap(&mut p1, &mut p2);
    }
    let (z1, z2) = (f64::from(p1[2]), f64::from(p2[2]));
    if z2 <= range.0 || z1 >= range.1 {
        return;
    }
    if z1 < range.0 {
        extend_point(bbox, intersection(p1, p2, range.0));
        if z2 > range.1 {
            extend_point(bbox, intersection(p1, p2, range.1));
        } else {
            extend_point(bbox, p2);
        }
    } else if z2 > range.1 {
        extend_point(bbox, intersection(p1, p2, range.1));
        extend_point(bbox, p1);
    } else {
        extend_point(bbox, p1);
        extend_point(bbox, p2);
    }
}
