use std::cmp::Ordering;

use crate::{Layer, Model, Point3, SliceError};

const EPSILON: f64 = 1e-6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    x: f64,
    y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: round_6(x),
            y: round_6(y),
        }
    }

    pub const fn x(&self) -> f64 {
        self.x
    }

    pub const fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment2 {
    start: Point2,
    end: Point2,
}

impl Segment2 {
    pub fn new(a: Point2, b: Point2) -> Self {
        if compare_points(a, b).is_gt() {
            Self { start: b, end: a }
        } else {
            Self { start: a, end: b }
        }
    }

    pub const fn start(&self) -> Point2 {
        self.start
    }

    pub const fn end(&self) -> Point2 {
        self.end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerSlice {
    layer_id: usize,
    print_z: f64,
    segments: Vec<Segment2>,
}

impl LayerSlice {
    pub fn new(layer_id: usize, print_z: f64, segments: Vec<Segment2>) -> Self {
        Self {
            layer_id,
            print_z: round_6(print_z),
            segments,
        }
    }

    pub const fn layer_id(&self) -> usize {
        self.layer_id
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }

    pub fn segments(&self) -> &[Segment2] {
        &self.segments
    }
}

pub fn slice_layers(model: &Model, layers: &[Layer]) -> Result<Vec<LayerSlice>, SliceError> {
    let mut out = Vec::with_capacity(layers.len());
    for layer in layers {
        let z = layer.print_z();
        let mut segments = Vec::new();
        for triangle in model.triangles() {
            if let Some(segment) = intersect_triangle(triangle.vertices(), z)? {
                segments.push(segment);
            }
        }
        segments.sort_by(compare_segments);
        out.push(LayerSlice::new(layer.id(), z, segments));
    }
    Ok(out)
}

fn intersect_triangle(vertices: &[Point3; 3], z: f64) -> Result<Option<Segment2>, SliceError> {
    let zs = vertices.map(|vertex| f64::from(vertex.z));
    if zs.iter().all(|vertex_z| (*vertex_z - z).abs() <= EPSILON) {
        return Ok(None);
    }

    let edges = [(0, 1), (1, 2), (2, 0)];
    let mut points = Vec::new();
    for (a, b) in edges {
        if let Some(point) = intersect_edge(vertices[a], vertices[b], z)? {
            push_unique(&mut points, point);
        }
    }

    match points.as_slice() {
        [a, b] => Ok(Some(Segment2::new(*a, *b))),
        [] | [_] => Ok(None),
        _ => Err(SliceError::InvalidInput(
            "triangle produced ambiguous slice segment".to_owned(),
        )),
    }
}

fn intersect_edge(a: Point3, b: Point3, z: f64) -> Result<Option<Point2>, SliceError> {
    let z0 = f64::from(a.z);
    let z1 = f64::from(b.z);
    if !z0.is_finite() || !z1.is_finite() {
        return Err(SliceError::InvalidInput(
            "triangle contains non-finite z".to_owned(),
        ));
    }
    let d0 = z0 - z;
    let d1 = z1 - z;

    if d0.abs() <= EPSILON && d1.abs() <= EPSILON {
        return Ok(None);
    }
    if d0.abs() <= EPSILON {
        return Ok(Some(Point2::new(f64::from(a.x), f64::from(a.y))));
    }
    if d1.abs() <= EPSILON {
        return Ok(Some(Point2::new(f64::from(b.x), f64::from(b.y))));
    }
    if (d0 > 0.0 && d1 > 0.0) || (d0 < 0.0 && d1 < 0.0) {
        return Ok(None);
    }

    let t = (z - z0) / (z1 - z0);
    let x0 = f64::from(a.x);
    let x1 = f64::from(b.x);
    let y0 = f64::from(a.y);
    let y1 = f64::from(b.y);
    Ok(Some(Point2::new(x0 + t * (x1 - x0), y0 + t * (y1 - y0))))
}

fn push_unique(points: &mut Vec<Point2>, point: Point2) {
    if !points.iter().any(|existing| points_equal(*existing, point)) {
        points.push(point);
    }
}

fn points_equal(a: Point2, b: Point2) -> bool {
    (a.x - b.x).abs() <= EPSILON && (a.y - b.y).abs() <= EPSILON
}

fn compare_points(a: Point2, b: Point2) -> Ordering {
    a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y))
}

fn compare_segments(a: &Segment2, b: &Segment2) -> Ordering {
    compare_points(a.start, b.start).then_with(|| compare_points(a.end, b.end))
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
