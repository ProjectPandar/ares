//! `WipePathHelpers.cpp:767-883`: candidate construction and strict selection.

use super::{
    Params,
    math::{Vector, tolerance},
    offset, side,
    validate::{Candidate, Validator},
};
use crate::geometry::Point;

pub(super) struct Selection<'a> {
    pub source: &'a [Point],
    pub validator: Validator<'a>,
    pub support: Vector,
    pub effective_offset: f64,
}

impl Selection<'_> {
    pub fn choose(&self) -> Option<Vec<Point>> {
        let params = self.validator.params;
        let preferred = self.offset_candidate(params.direction);
        let alternate = self.offset_candidate(-params.direction);
        let translated = self.translated(
            self.source,
            params.seam_start,
            params.seam_end,
            self.support,
        );
        let direct = self.direct(params.seam_end, self.support);
        let reversed = self.reversed();
        let margin = tolerance(self.effective_offset, 0.1, params.epsilon);
        let mut selected = preferred;
        if let Some(translated) = translated
            && selected
                .as_ref()
                .is_none_or(|candidate| translated.support_score + margin < candidate.support_score)
        {
            selected = Some(translated);
        }
        let mut selected = selected.or(direct).or(alternate);
        let length_margin = tolerance(params.max_length, 0.1, params.epsilon);
        if let Some(reversed) = reversed
            && selected
                .as_ref()
                .is_none_or(|candidate| longer(&reversed, candidate, length_margin, margin))
        {
            selected = Some(reversed);
        }
        selected.map(|candidate| candidate.path)
    }

    fn offset_candidate(&self, direction: i32) -> Option<Candidate> {
        let params = self.validator.params;
        let path = offset::offset(
            self.source,
            Params {
                direction,
                offset: self.effective_offset,
                ..params
            },
        )?;
        self.validator.candidate(
            path,
            self.validator.geometry(
                params.seam_start,
                self.support / self.support.norm(),
                self.effective_offset,
            ),
        )
    }

    fn translated(
        &self,
        source: &[Point],
        start: Point,
        end: Point,
        support: Vector,
    ) -> Option<Candidate> {
        let params = self.validator.params;
        let distance = support.norm();
        let offset = params.offset.min(distance);
        if offset <= params.epsilon {
            return None;
        }
        let translation = support * (offset / distance);
        let path = offset::translated(
            source,
            translation,
            Params {
                seam_start: start,
                seam_end: end,
                ..params
            },
        )?;
        self.validator.candidate(
            path,
            self.validator.geometry(start, support / distance, offset),
        )
    }

    fn direct(&self, origin: Point, support: Vector) -> Option<Candidate> {
        let params = self.validator.params;
        let distance = support.norm();
        let offset = params.offset.min(distance);
        if offset <= params.epsilon {
            return None;
        }
        let direction = support / distance;
        let destination = (Vector::from_point(origin) + direction * offset).point();
        if destination == params.wipe_start {
            return None;
        }
        let path = offset::store(vec![params.wipe_start, destination], params)?;
        let mut geometry = self.validator.geometry(origin, direction, offset);
        geometry.direction_start = params.wipe_start;
        geometry.require_clearance = false;
        self.validator.candidate(path, geometry)
    }

    fn reversed(&self) -> Option<Candidate> {
        let params = self.validator.params;
        if params.seam_start == params.seam_end || self.source.last() != Some(&params.seam_end) {
            return None;
        }
        let mut source = self.source.to_vec();
        source.reverse();
        let support = side::support_offset(
            &source,
            params.seam_end,
            true,
            self.validator.target,
            params,
        )?;
        let reversed = self.translated(&source, params.seam_end, params.seam_end, support);
        let margin = tolerance(params.max_length, 0.1, params.epsilon);
        if !reversed
            .as_ref()
            .is_none_or(|candidate| candidate.path_length + margin < params.max_length)
        {
            return reversed;
        }
        let Some(candidate) = self.reverse_offset(&source, support) else {
            return reversed;
        };
        let score_margin = tolerance(params.offset.min(support.norm()), 0.1, params.epsilon);
        if reversed
            .as_ref()
            .is_none_or(|current| longer(&candidate, current, margin, score_margin))
        {
            Some(candidate)
        } else {
            reversed
        }
    }

    fn reverse_offset(&self, source: &[Point], support: Vector) -> Option<Candidate> {
        let params = self.validator.params;
        let offset = params.offset.min(support.norm());
        if offset <= params.epsilon {
            return None;
        }
        let mut path = offset::offset(
            source,
            Params {
                seam_start: params.seam_end,
                seam_end: params.seam_start,
                direction: -params.direction,
                offset,
                ..params
            },
        )?;
        path[0] = params.seam_start;
        self.validator.candidate(
            path,
            self.validator
                .geometry(params.seam_end, support.normalized(), offset),
        )
    }
}

fn longer(candidate: &Candidate, current: &Candidate, margin: f64, score_margin: f64) -> bool {
    candidate.path_length > current.path_length + margin
        && candidate.support_score <= current.support_score + score_margin
}
