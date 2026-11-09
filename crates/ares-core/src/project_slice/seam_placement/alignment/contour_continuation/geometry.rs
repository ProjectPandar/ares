// SeamContour.cpp anonymous geometry helpers and material/boundary predicates.
use super::{Contour, Resolver};

pub(super) struct Foot {
    pub(super) point: [f64; 2],
    pub(super) distance: f64,
}

pub(super) fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

pub(super) fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

pub(super) fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    let delta = sub(a, b);
    dot(delta, delta).sqrt()
}

pub(super) fn box_distance(contour: &Contour, point: [f64; 2]) -> f64 {
    let delta = std::array::from_fn(|axis| {
        (contour.min[axis] - point[axis]).max(0.0) + (point[axis] - contour.max[axis]).max(0.0)
    });
    dot(delta, delta).sqrt()
}

pub(super) fn closest(
    contour: &Contour,
    point: [f64; 2],
    stop: f64,
    source_normals: Option<&[[f64; 2]]>,
    normal_cosine: f64,
) -> Foot {
    let mut result = Foot {
        point: [0.0; 2],
        distance: f64::INFINITY,
    };
    if stop >= 0.0 && source_normals.is_none() {
        for &vertex in &contour.points {
            let distance = distance(vertex, point);
            if distance <= stop {
                return Foot {
                    point: vertex,
                    distance,
                };
            }
        }
    }
    for (index, &a) in contour.points.iter().enumerate() {
        let delta = sub(contour.points[(index + 1) % contour.points.len()], a);
        let length2 = dot(delta, delta);
        if let Some(source_normals) = source_normals {
            if length2 <= 1e-18 {
                continue;
            }
            let sign = if contour.hole { -1.0 } else { 1.0 };
            let normal = [
                sign * delta[1] / length2.sqrt(),
                sign * -delta[0] / length2.sqrt(),
            ];
            if !source_normals
                .iter()
                .any(|&n| dot(n, normal) >= normal_cosine)
            {
                continue;
            }
        }
        let t = if length2 > 0.0 {
            (dot(sub(point, a), delta) / length2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let foot = [a[0] + t * delta[0], a[1] + t * delta[1]];
        let distance = distance(foot, point);
        if distance < result.distance {
            result = Foot {
                point: foot,
                distance,
            };
        }
        if result.distance <= stop {
            break;
        }
    }
    result
}

pub(super) fn normals(contour: &Contour, point: [f64; 2]) -> Vec<[f64; 2]> {
    let mut result = Vec::new();
    for (index, &a) in contour.points.iter().enumerate() {
        let delta = sub(contour.points[(index + 1) % contour.points.len()], a);
        let length2 = dot(delta, delta);
        if length2 <= 1e-18 {
            continue;
        }
        let t = (dot(sub(point, a), delta) / length2).clamp(0.0, 1.0);
        let foot = [a[0] + t * delta[0], a[1] + t * delta[1]];
        if distance(foot, point) <= 1e-6 {
            let sign = if contour.hole { -1.0 } else { 1.0 };
            let length = length2.sqrt();
            result.push([sign * (delta[1] / length), sign * (-delta[0] / length)]);
        }
    }
    result
}

impl Resolver {
    pub(super) fn in_material(&self, layer: usize, point: [f64; 2]) -> bool {
        let mut inside = false;
        for index in self.indices[layer].query(point, 0.0) {
            let contour = &self.layers[layer][index];
            if !contour.reliable {
                continue;
            }
            if (0..2).any(|axis| point[axis] < contour.min[axis] || point[axis] > contour.max[axis])
            {
                continue;
            }
            inside ^= contour
                .points
                .iter()
                .enumerate()
                .fold(false, |parity, (index, &a)| {
                    let b = contour.points[(index + 1) % contour.points.len()];
                    parity
                        ^ ((a[1] > point[1]) != (b[1] > point[1])
                            && point[0] < a[0] + (point[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1]))
                });
        }
        inside
    }

    pub(super) fn near_boundary(&self, layer: usize, point: [f64; 2], noise: f64) -> bool {
        self.indices[layer]
            .query(point, noise)
            .into_iter()
            .any(|index| {
                let contour = &self.layers[layer][index];
                box_distance(contour, point) <= noise
                    && closest(contour, point, noise, None, 0.0).distance <= noise
            })
    }
}
