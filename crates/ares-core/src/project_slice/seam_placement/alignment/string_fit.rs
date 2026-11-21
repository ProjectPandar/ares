use super::{LayerPlan, Vec3, candidate_position, loop_normal, paint_rules};
use crate::ProcessSeamPosition;
use crate::project_slice::seam_placement::{angle_penalty, spline::CubicSpline};

const MILLIMETERS_PER_SEGMENT: f32 = 4.0;
const SHARP_ANGLE_THRESHOLD: f32 = 55.0 * std::f32::consts::PI / 180.0;

pub(super) fn finalize_string(layers: &mut [LayerPlan], seam_string: &[(usize, usize)]) {
    let observations = seam_string
        .iter()
        .map(|&point| {
            let position = candidate_position(layers, point);
            (position.x, position.y)
        })
        .collect::<Vec<_>>();
    let points = seam_string
        .iter()
        .map(|&point| candidate_position(layers, point).z)
        .collect::<Vec<_>>();
    let weights = seam_string
        .iter()
        .map(|&point| {
            let angle_weight = 1.0 / (0.1 + angle_penalty(candidate_angle(layers, point)));
            if paint_rules::enforced(&layers[point.0].candidates.points[point.1]) {
                angle_weight + 3.0
            } else {
                angle_weight
            }
        })
        .collect::<Vec<_>>();
    let mut total_length = string_length(layers, seam_string);
    // SeamPlacer.cpp:1364-1366 — fewer spline segments for spRear strings.
    if layers[seam_string[0].0].placement == ProcessSeamPosition::Back {
        total_length *= 0.3;
    }
    let segment_count = ((total_length.max(0.0) / MILLIMETERS_PER_SEGMENT) as usize).max(1);
    let curve = CubicSpline::fit(&observations, &points, &weights, segment_count);
    let finals = seam_string
        .iter()
        .map(|&point| {
            let current = candidate_position(layers, point);
            let fitted = curve.value(current.z);
            let angle_ratio = candidate_angle(layers, point).abs() / SHARP_ANGLE_THRESHOLD;
            // SeamPlacer.cpp:1377-1378: round the cube to float once;
            // target-specific powf otherwise changes the blended seam.
            let blend = (f64::from(angle_ratio).powi(3) as f32).min(1.0);
            if blend >= 1.0 {
                return current;
            }
            let blend = paint_rules::corner_blend(&layers[point.0], point.1, blend);
            let target = Vec3::new(
                blend * current.x + (1.0 - blend) * fitted.0,
                blend * current.y + (1.0 - blend) * fitted.1,
                current.z,
            );
            // SeamPlacer.cpp:2050–2075: project the blended target onto the loop.
            loop_normal::place_on_loop(&layers[point.0].candidates, point.1, target)
        })
        .collect::<Vec<_>>();
    for (&point, final_position) in seam_string.iter().zip(finals) {
        let perimeter_index = layers[point.0].candidates.points[point.1].perimeter_index;
        let choice = &mut layers[point.0].choices[perimeter_index];
        choice.seam_index = point.1;
        choice.final_position = Some(final_position);
        choice.finalized = true;
    }
}

fn string_length(layers: &[LayerPlan], seam_string: &[(usize, usize)]) -> f32 {
    let mut total = 0.0;
    let mut last = candidate_position(layers, seam_string[0]);
    for (index, &point) in seam_string.iter().enumerate() {
        let current = candidate_position(layers, point);
        let layer_angle = if index == 0 || index + 1 == seam_string.len() {
            0.0
        } else {
            let previous = current - candidate_position(layers, seam_string[index - 1]);
            let next = candidate_position(layers, seam_string[index + 1]) - current;
            previous.normalized().dot(next.normalized()).acos().abs()
        };
        let influence = if !paint_rules::enforced(&layers[point.0].candidates.points[point.1])
            && layer_angle > 2.0 * candidate_angle(layers, point).abs()
        {
            -0.8
        } else {
            1.0
        };
        total += influence * (last - current).norm();
        last = current;
    }
    total
}

fn candidate_angle(layers: &[LayerPlan], point: (usize, usize)) -> f32 {
    layers[point.0].candidates.points[point.1].local_ccw_angle
}
