use super::{
    LayerPlan, Vec3,
    loop_normal::{step, xy},
};
use crate::{
    ProcessSeamPosition,
    project_slice::{
        seam_candidates::{PaintState, SeamCandidate},
        seam_placement::paint::OVERSAMPLING,
    },
};

pub(super) fn priority(
    placement: ProcessSeamPosition,
    a: &SeamCandidate,
    b: &SeamCandidate,
) -> Option<bool> {
    if matches!(
        placement,
        ProcessSeamPosition::Aligned | ProcessSeamPosition::AlignedBack
    ) && a.central_enforcer != b.central_enforcer
    {
        Some(a.central_enforcer)
    } else if a.paint != b.paint {
        Some(a.paint > b.paint)
    } else {
        None
    }
}

pub(super) fn enforced(candidate: &SeamCandidate) -> bool {
    candidate.paint == PaintState::Enforced
}

// SeamPlacer.cpp:1450–1487, 2060–2075.
pub(super) fn corner_blend(layer: &LayerPlan, index: usize, blend: f32) -> f32 {
    let candidate = &layer.candidates.points[index];
    if !enforced(candidate) {
        return blend;
    }
    let perimeter = &layer.candidates.perimeters[candidate.perimeter_index];
    let mut protect = perimeter.flow_width.max(layer.max_nozzle_diameter as f32);
    if protect <= 0.0 {
        protect = 0.5;
    }
    let min = 15.0 * std::f32::consts::PI / 180.0;
    let max = 30.0 * std::f32::consts::PI / 180.0;
    let smooth = |value: f32| {
        let x = value.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    let points = &layer.candidates.points[perimeter.start_index..perimeter.end_index];
    let contribution = |i: usize, distance: f32| {
        let corner = smooth((points[i].local_ccw_angle.abs() - min) / (max - min));
        let weight = if distance <= protect {
            1.0
        } else {
            1.0 - smooth((distance - protect) / protect)
        };
        corner * weight
    };
    let position = |i: usize| Vec3::new(points[i].position.x, points[i].position.y, 0.0);
    let index = index - perimeter.start_index;
    let mut influence = contribution(index, 0.0);
    for forward in [false, true] {
        let mut i = index;
        let mut distance = 0.0;
        for _ in 1..points.len() {
            if influence >= 1.0 {
                break;
            }
            let j = step(i, points.len(), forward);
            distance += (position(j) - position(i)).norm();
            if distance >= 2.0 * protect {
                break;
            }
            influence = influence.max(contribution(j, distance));
            i = j;
        }
    }
    blend.max(0.4 * influence)
}

// SeamPlacer.cpp:1523–1555: walk the allowed arcs from the chosen point.
pub(super) fn allowed_arc(
    points: &[SeamCandidate],
    index: usize,
    forward: bool,
    exact_ends: bool,
) -> (f64, [f64; 2]) {
    let enforced = enforced(&points[index]);
    let blocked = points[index].paint == PaintState::Blocked;
    let mut extension = if enforced && !exact_ends {
        f64::from(OVERSAMPLING)
    } else {
        0.0
    };
    let mut arc = 0.0;
    let mut in_run = true;
    let mut i = index;
    for _ in 0..points.len() {
        let j = step(i, points.len(), forward);
        let state = points[j].paint;
        if enforced && state != PaintState::Enforced
            || !enforced && !blocked && state == PaintState::Blocked
        {
            in_run = false;
        }
        let a = xy(&points[i]);
        if !in_run && (!enforced || state == PaintState::Blocked) {
            return (arc, a);
        }
        let b = xy(&points[j]);
        let delta = [b[0] - a[0], b[1] - a[1]];
        let length = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
        if !in_run {
            let reach = length.min(extension);
            extension -= reach;
            if reach < length {
                return (
                    arc + reach,
                    [
                        a[0] + reach / length * delta[0],
                        a[1] + reach / length * delta[1],
                    ],
                );
            }
        }
        arc += length;
        i = j;
    }
    (f64::INFINITY, xy(&points[index]))
}
