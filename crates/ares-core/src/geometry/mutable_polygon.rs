//! `MutablePolygon.cpp/.hpp::smooth_outward`, required by Organic drawing.

mod duplicates;
mod narrow;
mod ring;

pub(crate) use duplicates::remove_duplicates;

use super::{Coord, CoordinateScale, Point, Polygon};
use ring::{Ring, Unprocessed};

pub(crate) fn smooth_outward(
    mut polygons: Vec<Polygon>,
    distance: Coord,
    scale: CoordinateScale,
) -> Vec<Polygon> {
    let mut ring = Ring::default();
    for polygon in &mut polygons {
        ring.assign(polygon.points());
        smooth(&mut ring, distance, scale);
        *polygon = ring.polygon();
    }
    polygons.retain(|polygon| !polygon.points().is_empty());
    polygons
}

fn smooth(ring: &mut Ring, distance: Coord, scale: CoordinateScale) {
    ring.remove_duplicates(scale.scaled_delta(0.01));
    let distance2 = distance * distance;
    let epsilon = scale.scaled_delta(1e-4) as i64;
    let expanded2 = (distance + epsilon) * (distance + epsilon);
    let mut range = Unprocessed::new(ring);
    while let Some(index) = range.process_next(ring) {
        if ring.len <= 2 {
            break;
        }
        let before = ring.previous(index);
        let after = ring.next(index);
        let p0 = ring.point(before);
        let p1 = ring.point(index);
        let p2 = ring.point(after);
        let v1 = delta(p0, p1);
        let v2 = delta(p2, p1);
        if cross(v1, v2) <= 0 {
            continue;
        }
        let product = dot(v1, v2);
        let l1 = dot(v1, v1) as f64;
        let l2 = dot(v2, v2) as f64;
        if product <= 0 && (product as f64).powi(2) * 2.0 >= l1 * l2 {
            continue;
        }
        let connecting = delta(p2, p0);
        let length2 = dot(connecting, connecting);
        ring.remove(index);
        if length2 < distance2 {
            if narrow::clip(
                ring,
                &mut range,
                (p1, before, after),
                length2,
                (distance, epsilon),
            ) {
                return;
            }
        } else if length2 > expanded2 {
            obtuse(ring, after, p1, ([v1, v2], [l1, l2]), (distance2, scale));
        }
    }
    close_triangle(ring, distance2);
}

fn obtuse(
    ring: &mut Ring,
    after: usize,
    origin: Point,
    (vectors, [mut l1, mut l2]): ([[i64; 2]; 2], [f64; 2]),
    (distance2, scale): (i64, CoordinateScale),
) {
    let [mut v1, mut v2] = vectors.map(|vector| vector.map(|value| value as f64));
    let swap = l1 > l2;
    if swap {
        std::mem::swap(&mut v1, &mut v2);
        std::mem::swap(&mut l1, &mut l2);
    }
    let length1 = l1.sqrt();
    let length2 = l2.sqrt();
    let bisector = [
        v1[0] / length1 + v2[0] / length2,
        v1[1] / length1 + v2[1] / length2,
    ];
    let product = dot_f64(v1, bisector);
    let height2 = l1 - product * product / norm2(bisector);
    let epsilon = scale.scaled_delta(1e-4);
    if height2 < epsilon * epsilon {
        return;
    }
    if height2 < 0.25 * distance2 as f64 + epsilon {
        let b = -2.0 * dot_f64(v1, v2);
        let discriminant = b * b - 4.0 * l2 * (l1 - distance2 as f64);
        let t = (-b + discriminant.sqrt()) / (2.0 * l2);
        ring.insert(after, translated(origin, v2, t));
    } else {
        let t = (0.25 * distance2 as f64 / height2).sqrt();
        let t2 = t * length1 / length2;
        let mut p0 = translated(origin, v1, t);
        let mut p2 = translated(origin, v2, t2);
        if swap {
            std::mem::swap(&mut p0, &mut p2);
        }
        let new = ring.insert(after, p2);
        ring.insert(new, p0);
    }
}

fn close_triangle(ring: &mut Ring, distance2: i64) {
    if ring.len < 3 {
        ring.clear();
        return;
    }
    if ring.len != 3 {
        return;
    }
    let begin = ring.head.expect("triangle has a head");
    let p0 = ring.point(ring.previous(begin));
    let p1 = ring.point(begin);
    let p2 = ring.point(ring.next(begin));
    let mut v1 = delta(p0, p1);
    let mut v2 = delta(p2, p1);
    if cross(v1, v2) <= 0 {
        return;
    }
    let v3 = delta(p2, p0);
    let mut l1 = dot(v1, v1);
    let mut l2 = dot(v2, v2);
    let l3 = dot(v3, v3);
    if l2 > l1 && l2 > l3 {
        std::mem::swap(&mut v1, &mut v2);
        std::mem::swap(&mut l1, &mut l2);
    } else if l3 > l1 && l3 > l2 {
        v1 = v3;
        l1 = l3;
    }
    let product = dot(v1, v2) as f64;
    let height2 = l2 as f64 - product * product / l1 as f64;
    if height2 < distance2 as f64 {
        ring.clear();
    }
}

fn delta(a: Point, b: Point) -> [i64; 2] {
    [a.x() - b.x(), a.y() - b.y()]
}
fn dot(a: [i64; 2], b: [i64; 2]) -> i64 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross(a: [i64; 2], b: [i64; 2]) -> i64 {
    a[0] * b[1] - a[1] * b[0]
}
fn dot_f64(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn norm2(a: [f64; 2]) -> f64 {
    dot_f64(a, a)
}
fn translated(origin: Point, vector: [f64; 2], t: f64) -> Point {
    Point::new(
        origin.x() + (vector[0] * t) as i64,
        origin.y() + (vector[1] * t) as i64,
    )
}
