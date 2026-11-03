//! `WipePathHelpers.cpp:494-675`: local support and material-side validation.

use super::{
    Params,
    math::{Distancer, EPSILON, MIN_ALIGNMENT, Vector, project, sample, tolerance},
};
use crate::geometry::{Line, Point};

pub(super) fn support_offset(
    source: &[Point],
    origin: Point,
    disambiguate: bool,
    target: &Distancer<'_>,
    params: Params,
) -> Option<Vector> {
    if source.len() < 2 {
        return None;
    }
    let query = if disambiguate {
        sample(source, 2.0 * params.support_distance, params.epsilon)
    } else {
        source[0]
    };
    let hit = target.nearest(query);
    let line = target.lines[hit.index];
    let mut sampled = hit.point - Vector::from_point(query);
    if disambiguate {
        let edge = Vector::delta(line.b, line.a);
        if edge.norm() > params.epsilon {
            let tangent = edge.normalized();
            sampled = sampled - tangent * sampled.dot(tangent);
        }
    }
    if sampled.norm() <= params.epsilon {
        return None;
    }
    if !disambiguate {
        return Some(sampled);
    }
    Some(local_support(origin, sampled, target, params))
}

fn local_support(origin: Point, sampled: Vector, target: &Distancer<'_>, params: Params) -> Vector {
    let direction = sampled.normalized();
    let mut local = sampled;
    let mut best_score = f64::INFINITY;
    for index in target.radius(origin, 2.0 * params.support_distance + 4.0 * params.epsilon) {
        let (squared_distance, closest) = project(target.lines[index], origin);
        let offset = Vector::delta(closest.point(), origin);
        let distance = squared_distance.sqrt();
        if distance <= params.epsilon {
            continue;
        }
        let alignment = offset.normalized().dot(direction);
        if alignment < MIN_ALIGNMENT {
            continue;
        }
        let score = distance / alignment;
        if score < best_score {
            best_score = score;
            local = offset;
        }
    }
    local
}

pub(super) fn material_support(
    path: &[Point],
    seam: Point,
    dir: i32,
    support: &[Line],
    epsilon: f64,
) -> Vec<Line> {
    if path.len() < 4 || path.first() != path.last() {
        return Vec::new();
    }
    let Some(&outgoing) = path.iter().skip(1).find(|&&point| point != seam) else {
        return Vec::new();
    };
    let Some(&incoming) = path.iter().rev().skip(1).find(|&&point| point != seam) else {
        return Vec::new();
    };
    let outgoing = Vector::delta(outgoing, seam).normalized();
    let incoming = Vector::delta(seam, incoming).normalized();
    let material = (outgoing.left() + incoming.left()) * f64::from(dir);
    if material.norm() <= EPSILON {
        return Vec::new();
    }
    support
        .iter()
        .copied()
        .filter(|&line| {
            let (_, closest) = project(line, seam);
            Vector::delta(closest.point(), seam).dot(material) > epsilon
        })
        .collect()
}

pub(super) struct MaterialSide<'a> {
    pub target: &'a Distancer<'a>,
    pub current: &'a Distancer<'a>,
    pub effective_offset: f64,
    pub epsilon: f64,
}

impl MaterialSide<'_> {
    pub fn stays(&self, path: &[Point], start: Point, direction: Vector, clearance: bool) -> bool {
        if path.len() < 2
            || direction.norm() <= EPSILON
            || self.target.lines.is_empty()
            || self.current.lines.is_empty()
            || self.effective_offset <= self.epsilon
        {
            return false;
        }
        let initial = Vector::delta(path[1], start);
        if initial.norm() <= self.epsilon
            || initial.normalized().dot(direction.normalized()) < MIN_ALIGNMENT
        {
            return false;
        }
        if !self.point_on_side(path[1], clearance) {
            return false;
        }
        let mut previous = path[1];
        for &next in path.iter().skip(2) {
            let segment = Vector::delta(next, previous);
            let samples = (segment.norm() / self.effective_offset).ceil() as usize;
            let samples = samples.max(1);
            for sample in 1..=samples {
                let point = (Vector::from_point(previous)
                    + segment * (sample as f64 / samples as f64))
                    .point();
                if !self.point_on_side(point, clearance) {
                    return false;
                }
            }
            previous = next;
        }
        true
    }

    fn point_on_side(&self, point: Point, clearance: bool) -> bool {
        let hit = self.current.nearest(point);
        let lines = self.current.lines;
        if hit.index >= lines.len() {
            return false;
        }
        let line = lines[hit.index];
        let mut normal = left_normal(line, self.epsilon);
        let previous = lines[(hit.index + lines.len() - 1) % lines.len()];
        let next = lines[(hit.index + 1) % lines.len()];
        if (hit.point - Vector::from_point(line.a)).norm() <= self.epsilon && previous.b == line.a {
            normal = normal + left_normal(previous, self.epsilon);
        }
        if (hit.point - Vector::from_point(line.b)).norm() <= self.epsilon && next.a == line.b {
            normal = normal + left_normal(next, self.epsilon);
        }
        if normal.norm() <= EPSILON {
            return false;
        }
        normal = normal.normalized();
        let support = self.target.nearest(hit.point.point()).point;
        let support_side = (support - hit.point).dot(normal);
        if support_side.abs() <= 4.0 * self.epsilon {
            return false;
        }
        let side = (Vector::from_point(point) - hit.point).dot(normal)
            * if support_side > 0.0 { 1.0 } else { -1.0 };
        let minimum = self.effective_offset - tolerance(self.effective_offset, 0.25, self.epsilon);
        side >= -4.0 * self.epsilon && (!clearance || hit.distance + 4.0 * self.epsilon >= minimum)
    }
}

fn left_normal(line: Line, epsilon: f64) -> Vector {
    let edge = Vector::delta(line.b, line.a);
    if edge.norm() <= epsilon {
        Vector::default()
    } else {
        edge.left().normalized()
    }
}
