//! `construct_gap_for_skip_points` (`WipeTower.cpp:330-346, 422-708`, v2.5.0-beta `345efec64`):
//! wall openings at ray-selected skip points.
use super::scaled;
use crate::geometry::{BoundingBox, CoordinateScale, Point, Polygon, Polyline};

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

/// `add_extra_point` (`WipeTower.cpp:517-606`): the edge whose midpoint is nearest the bottom
/// centre gains its midpoint and a point `scale_range` to either side of it.
fn add_extra_point(polygon: &Polygon, scale_range: i32) -> Polygon {
    let points = polygon.points();
    let bounds = BoundingBox::from_polygon(polygon).expect("the wall has points");
    let anchor = [
        ((bounds.min().x() + bounds.max().x()) / 2) as f32,
        bounds.min().y() as f32,
    ];
    let float = |p: Point| [p.x() as f32, p.y() as f32];
    let mid = |a: [f32; 2], b: [f32; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let (mut closest, mut min_distance) = (0, f32::MAX);
    for i in 0..points.len() {
        let m = mid(float(points[i]), float(points[(i + 1) % points.len()]));
        let distance =
            (anchor[0] - m[0]) * (anchor[0] - m[0]) + (anchor[1] - m[1]) * (anchor[1] - m[1]);
        if distance < min_distance {
            (closest, min_distance) = (i, distance);
        }
    }
    let a = float(points[closest]);
    let b = float(points[(closest + 1) % points.len()]);
    let m = mid(a, b);
    let to_a = [a[0] - m[0], a[1] - m[1]];
    let to_b = [b[0] - m[0], b[1] - m[1]];
    let len_a = (to_a[0] * to_a[0] + to_a[1] * to_a[1]).sqrt();
    let len_b = (to_b[0] * to_b[0] + to_b[1] * to_b[1]).sqrt();
    if f64::from(len_a) < 1e-4 || f64::from(len_b) < 1e-4 {
        return polygon.clone();
    }
    let range = (scale_range as f32).min(len_a.min(len_b) * 0.9);
    let round = |p: [f32; 2]| Point::new(p[0].round() as i64, p[1].round() as i64);
    let offset =
        |to: [f32; 2], len: f32| round([m[0] + to[0] / len * range, m[1] + to[1] / len * range]);
    let mut result = points[..=closest].to_vec();
    result.extend([offset(to_a, len_a), round(m), offset(to_b, len_b)]);
    result.extend_from_slice(&points[closest + 1..]);
    Polygon::new(result)
}

/// `construct_gap_for_skip_points` (`WipeTower.cpp:695-708`) and `remove_points_from_polygon`
/// (`:610-693`): the wall paths left after cutting `range` either side of each skip point, and
/// the polygon with the cut points inserted.
pub(in crate::project_slice::gcode_emit::wipe_tower) fn split(
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
    let polygon = add_extra_point(polygon, (f64::from(range) / scale.factor()) as i32);
    let bounds = BoundingBox::from_polygon(&polygon).expect("the wall has points");
    let anchor = Point::new((bounds.min().x() + bounds.max().x()) / 2, bounds.min().y());
    let distance = |p: Point| {
        let (x, y) = ((p.x() - anchor.x()) as f64, (p.y() - anchor.y()) as f64);
        (x * x + y * y).sqrt()
    };
    let mut start = 0;
    for (i, &p) in polygon.points().iter().enumerate().skip(1) {
        if distance(p) < distance(polygon.points()[start]) {
            start = i;
        }
    }
    let count = polygon.points().len();
    let points = (0..count)
        .map(|i| {
            let p = polygon.points()[(start + i) % count];
            [scale.unscale(p.x()) as f32, scale.unscale(p.y()) as f32]
        })
        .collect::<Vec<_>>();
    let mut intersections = Vec::new();
    for (pair, &gap) in gaps.iter().enumerate() {
        let ray = if gap[0].abs() < width / 2.0 {
            -1.0_f32
        } else {
            1.0
        };
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
