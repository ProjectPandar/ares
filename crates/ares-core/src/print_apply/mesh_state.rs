use super::transform_state::StagedTransform3f;

const ORCA_EPSILON_F32: f32 = 1e-4;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedIndexedTriangleSet {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[usize; 3]>,
}

impl StagedIndexedTriangleSet {
    pub(super) fn new(vertices: Vec<[f32; 3]>, indices: Vec<[usize; 3]>) -> Self {
        Self { vertices, indices }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedBoundingBox3f {
    min: [f32; 3],
    max: [f32; 3],
}

impl StagedBoundingBox3f {
    fn new(point: [f32; 3]) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    fn empty() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }

    pub(super) fn from_min_max(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    pub(super) fn min(&self) -> [f32; 3] {
        self.min
    }

    pub(super) fn max(&self) -> [f32; 3] {
        self.max
    }

    fn extend(&mut self, point: [f32; 3]) {
        for (axis, value) in point.into_iter().enumerate() {
            self.min[axis] = self.min[axis].min(value);
            self.max[axis] = self.max[axis].max(value);
        }
    }

    fn expand(&mut self, offset: f32) {
        self.min[0] -= offset;
        self.min[1] -= offset;
        self.min[2] -= ORCA_EPSILON_F32;
        self.max[0] += offset;
        self.max[1] += offset;
        self.max[2] += ORCA_EPSILON_F32;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedLayerHeightRange {
    start: f64,
    end: f64,
}

impl StagedLayerHeightRange {
    pub(super) fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    pub(super) fn first(&self) -> f64 {
        self.start
    }

    pub(super) fn second(&self) -> f64 {
        self.end
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedRangeBoundingBox3f {
    bbox: StagedBoundingBox3f,
    populated: bool,
}

impl StagedRangeBoundingBox3f {
    pub(super) fn new_populated(bbox: StagedBoundingBox3f) -> Self {
        Self {
            bbox,
            populated: true,
        }
    }

    pub(super) fn new_empty() -> Self {
        Self::empty()
    }

    fn empty() -> Self {
        Self {
            bbox: StagedBoundingBox3f::empty(),
            populated: false,
        }
    }

    pub(super) fn is_populated(&self) -> bool {
        self.populated
    }

    pub(super) fn min(&self) -> [f32; 3] {
        self.bbox.min()
    }

    pub(super) fn max(&self) -> [f32; 3] {
        self.bbox.max()
    }
}

pub(super) fn staged_transformed_its_bbox2d(
    its: &StagedIndexedTriangleSet,
    transform: &StagedTransform3f,
    offset: f32,
) -> StagedBoundingBox3f {
    assert!(!its.indices.is_empty());

    let first_triangle = its.indices[0];
    let mut bbox =
        StagedBoundingBox3f::new(transform_point(transform, its.vertices[first_triangle[0]]));
    for triangle in &its.indices {
        for vertex_index in triangle {
            bbox.extend(transform_point(transform, its.vertices[*vertex_index]));
        }
    }
    bbox.expand(offset);
    bbox
}

pub(super) fn staged_transformed_its_bboxes_in_z_ranges(
    its: &StagedIndexedTriangleSet,
    transform: &StagedTransform3f,
    z_ranges: &[StagedLayerHeightRange],
    offset: f32,
) -> Vec<StagedRangeBoundingBox3f> {
    let mut bboxes = vec![StagedRangeBoundingBox3f::empty(); z_ranges.len()];
    for triangle in &its.indices {
        let pts =
            triangle.map(|vertex_index| transform_point(transform, its.vertices[vertex_index]));
        for (z_range, bbox) in z_ranges.iter().zip(&mut bboxes) {
            extend_triangle_edges_in_range(pts, z_range, bbox);
        }
    }

    for bbox in &mut bboxes {
        bbox.bbox.expand(offset);
    }
    bboxes
}

fn extend_triangle_edges_in_range(
    pts: [[f32; 3]; 3],
    z_range: &StagedLayerHeightRange,
    bbox: &mut StagedRangeBoundingBox3f,
) {
    let mut previous = 2;
    for edge in 0..3 {
        extend_edge_in_range(pts[previous], pts[edge], z_range, bbox);
        previous = edge;
    }
}

fn extend_edge_in_range(
    mut p1: [f32; 3],
    mut p2: [f32; 3],
    z_range: &StagedLayerHeightRange,
    bbox: &mut StagedRangeBoundingBox3f,
) {
    if p1[2] > p2[2] {
        std::mem::swap(&mut p1, &mut p2);
    }

    let p1_z = f64::from(p1[2]);
    let p2_z = f64::from(p2[2]);
    if p2_z <= z_range.start || p1_z >= z_range.end {
        return;
    }
    if p1_z < z_range.start {
        extend_lower_crossing_edge(p1, p2, z_range, bbox);
    } else if p2_z > z_range.end {
        let t = ((z_range.end - p1_z) / (p2_z - p1_z)) as f32;
        extend_ranged_bbox(bbox, interpolate_xy_at_z(p1, p2, t, z_range.end));
        extend_ranged_bbox(bbox, p1);
    } else {
        extend_ranged_bbox(bbox, p1);
        extend_ranged_bbox(bbox, p2);
    }
}

fn extend_lower_crossing_edge(
    p1: [f32; 3],
    p2: [f32; 3],
    z_range: &StagedLayerHeightRange,
    bbox: &mut StagedRangeBoundingBox3f,
) {
    let p1_z = f64::from(p1[2]);
    let p2_z = f64::from(p2[2]);
    if p2_z > z_range.end {
        let zspan = p2_z - p1_z;
        let t1 = ((z_range.start - p1_z) / zspan) as f32;
        let t2 = ((z_range.end - p1_z) / zspan) as f32;
        extend_ranged_bbox(bbox, interpolate_xy_at_z(p1, p2, t1, z_range.start));
        extend_ranged_bbox(bbox, interpolate_xy_at_z(p1, p2, t2, z_range.end));
    } else {
        let t = ((z_range.start - p1_z) / (p2_z - p1_z)) as f32;
        extend_ranged_bbox(bbox, interpolate_xy_at_z(p1, p2, t, z_range.start));
        extend_ranged_bbox(bbox, p2);
    }
}

fn transform_point(transform: &StagedTransform3f, point: [f32; 3]) -> [f32; 3] {
    let rows = transform.rows();
    std::array::from_fn(|row| {
        rows[row][0] * point[0] + rows[row][1] * point[1] + rows[row][2] * point[2] + rows[row][3]
    })
}

fn extend_ranged_bbox(bbox: &mut StagedRangeBoundingBox3f, point: [f32; 3]) {
    if bbox.populated {
        bbox.bbox.extend(point);
    } else {
        bbox.bbox = StagedBoundingBox3f::new(point);
        bbox.populated = true;
    }
}

fn interpolate_xy_at_z(p1: [f32; 3], p2: [f32; 3], t: f32, z: f64) -> [f32; 3] {
    [
        p1[0] + (p2[0] - p1[0]) * t,
        p1[1] + (p2[1] - p1[1]) * t,
        z as f32,
    ]
}
