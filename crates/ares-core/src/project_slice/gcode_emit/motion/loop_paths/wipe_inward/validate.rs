//! `WipePathHelpers.cpp:404-492,740-767`: complete-path support and validation.

use super::{
    Params,
    math::{Distancer, MITER_LIMIT, Vector, project, tolerance},
    offset,
    side::MaterialSide,
};
use crate::geometry::{Line, Point};

pub(super) struct Candidate {
    pub path: Vec<Point>,
    pub support_score: f64,
    pub path_length: f64,
}

#[derive(Clone, Copy)]
pub(super) struct CandidateGeometry {
    pub path_start: Point,
    pub direction_start: Point,
    pub contact_tolerance: f64,
    pub support_direction: Vector,
    pub offset: f64,
    pub require_clearance: bool,
}

pub(super) struct Validator<'a> {
    pub params: Params,
    pub target: &'a Distancer<'a>,
    pub all_support: &'a Distancer<'a>,
    pub current: &'a Distancer<'a>,
}

impl Validator<'_> {
    pub fn candidate(&self, path: Vec<Point>, geometry: CandidateGeometry) -> Option<Candidate> {
        let params = self.params;
        let backtracks = params.seam_start != params.seam_end
            && offset::backtracks(&path, params.wipe_start, params.epsilon);
        let side = MaterialSide {
            target: self.target,
            current: self.current,
            effective_offset: geometry.offset,
            epsilon: params.epsilon,
        };
        let material_side = side.stays(
            &path,
            geometry.direction_start,
            geometry.support_direction,
            geometry.require_clearance && params.seam_start == params.seam_end,
        );
        let connector =
            self.connector_clear(&path, geometry.path_start, geometry.contact_tolerance);
        if backtracks || !material_side || !connector {
            return None;
        }
        let support_score = self.support_score(&path)?;
        let path_length = offset::executable_length(&path, params.wipe_start);
        Some(Candidate {
            path,
            support_score,
            path_length,
        })
    }

    fn support_score(&self, path: &[Point]) -> Option<f64> {
        let params = self.params;
        if path.len() < 2 || self.target.lines.is_empty() || params.support_distance <= 0.0 {
            return None;
        }
        if self.target.nearest(params.wipe_start).distance
            > MITER_LIMIT * params.support_distance + 4.0 * params.epsilon
        {
            return None;
        }
        let mut previous = params.wipe_start;
        for &point in path.iter().skip(1) {
            if !segment_supported(
                previous,
                point,
                self.all_support,
                params.support_distance + 4.0 * params.epsilon,
                params.epsilon,
            ) {
                return None;
            }
            previous = point;
        }
        Some(self.target.nearest(path[1]).distance)
    }

    fn connector_clear(&self, path: &[Point], seam: Point, contact: f64) -> bool {
        let start = self.params.wipe_start;
        if path.len() < 2 || path[1] == start {
            return false;
        }
        if seam == start {
            return true;
        }
        let connector = Line::new(start, path[1]);
        if self
            .current
            .intersections(connector)
            .iter()
            .any(|&(point, _)| Vector::delta(point, start).norm() > contact)
        {
            return false;
        }
        let (squared, closest) = project(connector, seam);
        !(squared <= contact * contact && Vector::delta(closest.point(), start).norm() > contact)
    }

    pub fn geometry(&self, start: Point, direction: Vector, distance: f64) -> CandidateGeometry {
        CandidateGeometry {
            path_start: start,
            direction_start: start,
            contact_tolerance: tolerance(distance, 0.1, self.params.epsilon),
            support_direction: direction,
            offset: distance,
            require_clearance: true,
        }
    }
}

fn segment_supported(
    start: Point,
    end: Point,
    target: &Distancer<'_>,
    maximum: f64,
    epsilon: f64,
) -> bool {
    let midpoint = ((Vector::from_point(start) + Vector::from_point(end)) * 0.5).point();
    let hit = target.nearest(midpoint);
    if hit.distance > maximum {
        return false;
    }
    let line = target.lines[hit.index];
    if line.distance_to(start) <= maximum && line.distance_to(end) <= maximum {
        return true;
    }
    if target.nearest(start).distance > maximum || target.nearest(end).distance > maximum {
        return false;
    }
    if Vector::delta(end, start).norm() <= epsilon {
        return false;
    }
    segment_supported(start, midpoint, target, maximum, epsilon)
        && segment_supported(midpoint, end, target, maximum, epsilon)
}
