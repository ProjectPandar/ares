//! `WipeTower.cpp:228–244,329–508`: ray-selected wall-entry gaps.
use super::super::geometry::scaled;
use crate::geometry::{CoordinateScale, Point, Polygon, Polyline};

#[derive(Clone, Copy)]
struct Vertex {
    point: [f32; 2],
    pair: Option<usize>,
    forward: bool,
}
struct Intersection {
    edge: usize,
    point: [f32; 2],
    distance: f32,
    pair: usize,
    forward: bool,
}
fn length(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}
fn along(
    points: &[[f32; 2]],
    start: [f32; 2],
    edge: usize,
    distance: f32,
    (forward, pair): (bool, usize),
) -> Intersection {
    let count = points.len();
    let next = (edge + 1) % count;
    let first = length(start, points[if forward { next } else { edge }]);
    if distance <= first {
        let direction = if forward {
            [points[next][0] - start[0], points[next][1] - start[1]]
        } else {
            [
                points[edge][0] - points[next][0],
                points[edge][1] - points[next][1],
            ]
        };
        let norm = (direction[0] * direction[0] + direction[1] * direction[1]).sqrt();
        let point = [
            start[0] + direction[0] / norm * distance,
            start[1] + direction[1] / norm * distance,
        ];
        return Intersection {
            edge,
            point,
            distance: length(points[edge], point),
            pair,
            forward,
        };
    }
    let mut remaining = distance - first;
    let mut i = if forward {
        next
    } else {
        (edge + count - 1) % count
    };
    while i != edge {
        let next = (i + 1) % count;
        let segment = length(points[i], points[next]);
        if remaining <= segment {
            let ratio = remaining / segment;
            let delta = [
                points[next][0] - points[i][0],
                points[next][1] - points[i][1],
            ];
            let point = if forward {
                [
                    points[i][0] + ratio * delta[0],
                    points[i][1] + ratio * delta[1],
                ]
            } else {
                [
                    points[next][0] - ratio * delta[0],
                    points[next][1] - ratio * delta[1],
                ]
            };
            return Intersection {
                edge: i,
                point,
                distance: if forward {
                    remaining
                } else {
                    segment - remaining
                },
                pair,
                forward,
            };
        }
        remaining -= segment;
        i = if forward {
            next
        } else {
            (i + count - 1) % count
        };
    }
    let i = if forward {
        (edge + count - 1) % count
    } else {
        edge
    };
    Intersection {
        edge: i,
        point: points[edge],
        distance: if forward {
            length(points[i], points[edge])
        } else {
            0.0
        },
        pair,
        forward,
    }
}

pub(super) fn split(
    polygon: &Polygon,
    gaps: &[[f32; 2]],
    width: f32,
    range: f32,
    scale: CoordinateScale,
) -> (Vec<Polyline>, Polygon) {
    if gaps.is_empty() {
        let mut points = polygon.points().to_vec();
        points.push(points[0]);
        return (vec![Polyline::new(points)], polygon.clone());
    }
    let left = gaps[0][0].abs() < width / 2.0;
    let min_x = polygon.points().iter().map(|p| p.x()).min().unwrap();
    let max_x = polygon.points().iter().map(|p| p.x()).max().unwrap();
    let min_y = polygon.points().iter().map(|p| p.y()).min().unwrap();
    let anchor = Point::new(if left { max_x } else { min_x }, min_y);
    let start = polygon
        .points()
        .iter()
        .enumerate()
        .min_by_key(|(_, p)| {
            let x = p.x() - anchor.x();
            let y = p.y() - anchor.y();
            i128::from(x) * i128::from(x) + i128::from(y) * i128::from(y)
        })
        .unwrap()
        .0;
    let count = polygon.points().len();
    let points = (0..count)
        .map(|i| {
            let p = polygon.points()[(start + i) % count];
            [scale.unscale(p.x()) as f32, scale.unscale(p.y()) as f32]
        })
        .collect::<Vec<_>>();
    let ray = if left { -1.0_f32 } else { 1.0 };
    let mut intersections = Vec::new();
    for (pair, &gap) in gaps.iter().enumerate() {
        for edge in 0..count {
            let a = points[edge];
            let b = points[(edge + 1) % count];
            let delta = [b[0] - a[0], b[1] - a[1]];
            let denom = f64::from(ray * delta[1]);
            if denom.abs() < 1e-4 {
                continue;
            }
            let offset = [gap[0] - a[0], gap[1] - a[1]];
            let t1 = f64::from(delta[0] * offset[1] - delta[1] * offset[0]) / denom;
            let t2 = f64::from(ray * offset[1]) / denom;
            if t1 >= 0.0 && (0.0..=1.0).contains(&t2) {
                let point = [gap[0] + t1 as f32 * ray, gap[1]];
                intersections.push(along(&points, point, edge, range, (false, pair)));
                intersections.push(along(&points, point, edge, range, (true, pair)));
                break;
            }
        }
    }
    intersections.sort_by(|a, b| {
        a.edge
            .cmp(&b.edge)
            .then_with(|| a.distance.total_cmp(&b.distance))
    });
    let mut vertices = points
        .into_iter()
        .map(|point| Vertex {
            point,
            pair: None,
            forward: false,
        })
        .collect::<Vec<_>>();
    for intersection in intersections.into_iter().rev() {
        let i = intersection.edge;
        let next = (i + 1) % vertices.len();
        let near = |p: [f32; 2]| {
            let dx = p[0] - intersection.point[0];
            let dy = p[1] - intersection.point[1];
            f64::from(dx * dx + dy * dy) < 1e-4
        };
        let vertex = Vertex {
            point: intersection.point,
            pair: Some(intersection.pair),
            forward: intersection.forward,
        };
        if near(vertices[i].point) {
            vertices[i].pair = vertex.pair;
            vertices[i].forward = vertex.forward;
        } else if near(vertices[next].point) {
            vertices[next].pair = vertex.pair;
            vertices[next].forward = vertex.forward;
        } else {
            vertices.insert(i + 1, vertex);
        }
    }
    let inserted = Polygon::new(vertices.iter().map(|v| scaled(v.point, scale)).collect());
    let mut result = Vec::new();
    let mut path = Vec::new();
    let mut skip = true;
    let mut i = 0;
    loop {
        if skip || vertices[i].pair.is_none() {
            path.push(scaled(vertices[i].point, scale));
            i = (i + 1) % vertices.len();
            skip = false;
        } else {
            if !path.is_empty() {
                path.push(scaled(vertices[i].point, scale));
                result.push(Polyline::new(std::mem::take(&mut path)));
            }
            let mut pair = vertices[i].pair;
            let mut j = (i + 1) % vertices.len();
            while j != 0 && vertices[j].pair != pair {
                pair = vertices[j].pair.filter(|_| !vertices[j].forward).or(pair);
                j = (j + 1) % vertices.len();
            }
            i = j;
            skip = true;
        }
        if i == 0 {
            break;
        }
    }
    if !path.is_empty() {
        if vertices[0].pair.is_none() {
            path.push(scaled(vertices[0].point, scale));
        }
        result.push(Polyline::new(path));
    }
    (result, inserted)
}
