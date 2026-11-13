//! OrcaSlicer 2.5.0 `ExtrusionEntity.cpp:493-519::split_at_slope_end`.

use super::{ExtrusionPath, Point3, lerp};
use crate::project_slice::perimeters::classic::materialize::Polyline3;

pub(super) fn path(
    src: &ExtrusionPath,
    (distance, snap): (f64, f64),
    scale: f64,
) -> (ExtrusionPath, ExtrusionPath) {
    let points = &src.polyline.points;
    let mut traversed = 0.0;
    for (index, segment) in points.windows(2).enumerate() {
        let delta = difference(segment[1], segment[0]);
        let length = dot(delta, delta).sqrt();
        let end_length = traversed + length;
        if end_length >= distance {
            let exact = lerp(segment[0], segment[1], (distance - traversed) / length);
            let short_remainder = end_length - distance < snap;
            let skip_vertex = short_remainder
                && distance >= 0.0001 / scale
                && index + 2 < points.len()
                && chord_distance(exact, points[index + 2], segment[1]) <= 0.05 / scale;
            let point = if short_remainder && !skip_vertex {
                segment[1]
            } else {
                exact
            };
            let mut before = points[..=index].to_vec();
            if before.last() != Some(&point) {
                before.push(point);
            }
            let tail = index + 1 + usize::from(skip_vertex);
            let mut after = Vec::with_capacity(points.len() - tail + 1);
            if skip_vertex || point != segment[1] {
                after.push(point);
            }
            after.extend_from_slice(&points[tail..]);
            return (copy_with_points(src, before), copy_with_points(src, after));
        }
        traversed = end_length;
    }
    (
        src.clone(),
        copy_with_points(src, vec![*points.last().unwrap()]),
    )
}

fn chord_distance(a: Point3, b: Point3, p: Point3) -> f64 {
    let chord = difference(b, a);
    let offset = difference(p, a);
    let squared_length = dot(chord, chord);
    let t = if squared_length > 0.0 {
        (dot(offset, chord) / squared_length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let deviation = std::array::from_fn(|i| offset[i] - t * chord[i]);
    dot(deviation, deviation).sqrt()
}

fn difference(a: Point3, b: Point3) -> [f64; 3] {
    [
        a.x as f64 - b.x as f64,
        a.y as f64 - b.y as f64,
        a.z as f64 - b.z as f64,
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn copy_with_points(path: &ExtrusionPath, points: Vec<Point3>) -> ExtrusionPath {
    ExtrusionPath {
        z_contoured: path.z_contoured,
        polyline: Polyline3 {
            points,
            fitting: Vec::new(),
        },
        role: path.role,
        can_reverse: path.can_reverse,
        mm3_per_mm: path.mm3_per_mm,
        width: path.width,
        height: path.height,
    }
}
