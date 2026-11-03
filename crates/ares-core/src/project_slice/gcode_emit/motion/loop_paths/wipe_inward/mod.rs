//! `WipePathHelpers.cpp:677-884`: supported atomic wipe-path replacement.

mod build;
mod math;
mod offset;
mod select;
mod side;
mod support;
mod validate;

use crate::geometry::Point;
use math::{Distancer, lines};
use select::Selection;
use validate::Validator;

pub(in crate::project_slice::gcode_emit::motion) use support::WipeInwardSupport;

#[derive(Clone, Copy)]
pub(super) struct Params {
    pub seam_start: Point,
    pub seam_end: Point,
    pub wipe_start: Point,
    pub direction: i32,
    pub offset: f64,
    pub max_length: f64,
    pub support_distance: f64,
    pub epsilon: f64,
}

pub(super) fn offset_direction(is_ccw: bool, is_hole: bool) -> i32 {
    let inside = if is_ccw { 1 } else { -1 };
    if is_hole { -inside } else { inside }
}

pub(super) fn apply(path: &mut Vec<Point>, support: &WipeInwardSupport, params: Params) -> bool {
    let current_lines = lines(path);
    if path.len() < 2
        || support.inner_lines.is_empty()
        || current_lines.is_empty()
        || params.offset <= params.epsilon
        || params.max_length <= params.epsilon
        || params.support_distance <= params.epsilon
    {
        return false;
    }
    let material;
    let support_lines = if params.seam_start == params.seam_end {
        material = side::material_support(
            path,
            params.seam_start,
            params.direction,
            &support.inner_lines,
            params.epsilon,
        );
        if material.is_empty() {
            return false;
        }
        &material
    } else {
        &support.inner_lines
    };
    let target = Distancer::new(support_lines);
    let Some(toward_support) = side::support_offset(
        path,
        params.seam_end,
        params.seam_start != params.seam_end,
        &target,
        params,
    ) else {
        return false;
    };
    let effective_offset = params.offset.min(toward_support.norm());
    if effective_offset <= params.epsilon {
        return false;
    }
    let mut all_lines = support.printed_lines.clone();
    all_lines.extend_from_slice(&current_lines);
    let all_support = Distancer::new(&all_lines);
    let current = Distancer::new(&current_lines);
    let selection = Selection {
        source: path,
        support: toward_support,
        effective_offset,
        validator: Validator {
            params,
            target: &target,
            all_support: &all_support,
            current: &current,
        },
    };
    let Some(selected) = selection.choose() else {
        return false;
    };
    *path = selected;
    true
}
