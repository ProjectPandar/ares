//! MutablePolygon.cpp40–68 / MutablePolygon.hpp312–329.

use super::{delta, dot, ring::Ring};
use crate::geometry::{ExPolygon, Polygon};

fn clean(ring: &mut Ring, polygon: &Polygon, distance: i64, angle: f64) -> Polygon {
    ring.assign(polygon.points());
    if ring.len < 3 {
        ring.clear();
        return ring.polygon();
    }
    let begin = ring.head.unwrap();
    let mut index = ring.next(begin);
    let cos2 = angle.cos().powi(2);
    let epsilon2 = distance * distance;
    while index != begin {
        let previous = ring.previous(index);
        let next = ring.next(index);
        let v1 = delta(ring.point(index), ring.point(previous));
        let length2 = dot(v1, v1);
        if length2 < epsilon2 {
            let v2 = delta(ring.point(next), ring.point(previous));
            if (dot(v1, v2) as f64).powi(2) > cos2 * length2 as f64 * dot(v2, v2) as f64 {
                index = ring.remove(index);
                continue;
            }
        }
        index = next;
    }
    if ring.len < 3 {
        ring.clear();
    }
    ring.polygon()
}

pub(crate) fn remove_duplicates(
    expolygons: Vec<ExPolygon>,
    distance: i64,
    angle: f64,
) -> Vec<ExPolygon> {
    let mut ring = Ring::default();
    let mut output = Vec::with_capacity(expolygons.len());
    for polygon in expolygons {
        let (contour, holes) = polygon.into_parts();
        let contour = clean(&mut ring, &contour, distance, angle);
        let holes = holes
            .into_iter()
            .map(|hole| clean(&mut ring, &hole, distance, angle))
            .filter(|hole| !hole.points().is_empty())
            .collect();
        if !contour.points().is_empty() {
            output.push(ExPolygon::new(contour, holes));
        }
    }
    output
}
