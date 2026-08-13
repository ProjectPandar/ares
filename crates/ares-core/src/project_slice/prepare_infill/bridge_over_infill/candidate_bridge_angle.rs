#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the source-cited dependency remains unwired until the bridge transaction"
    )
)]

#[cfg(test)]
mod tests;

use crate::{
    ProcessInfillPattern, RegionOptions,
    geometry::{CoordinateScale, Line, Polygon, Polyline},
};

use super::{
    automatic_bridge_angle::determine_automatic_bridge_angle,
    internal_bridge_angle::apply_internal_bridge_angle_override,
};

struct CandidateBridgeAngleInput<'a> {
    area_to_be_bridge: &'a [Polygon],
    anchors: &'a [Polyline],
    boundary_polylines: &'a [Polyline],
    region: &'a RegionOptions,
    model_rotation_rad: f64,
    scale: CoordinateScale,
}

#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the source candidate-angle inputs without inventing ownership"
)]
pub(in crate::project_slice) fn determine_candidate_bridge_angle(
    area_to_be_bridge: &[Polygon],
    anchors: &[Polyline],
    boundary_polylines: &[Polyline],
    region: &RegionOptions,
    model_rotation_rad: f64,
    scale: CoordinateScale,
) -> f64 {
    determine_candidate_bridge_angle_using(
        CandidateBridgeAngleInput {
            area_to_be_bridge,
            anchors,
            boundary_polylines,
            region,
            model_rotation_rad,
            scale,
        },
        determine_automatic_bridge_angle,
        apply_internal_bridge_angle_override,
    )
}

fn determine_candidate_bridge_angle_using<Detect, Override>(
    input: CandidateBridgeAngleInput<'_>,
    detect: Detect,
    apply_override: Override,
) -> f64
where
    Detect: FnOnce(&[Polygon], &[Line], ProcessInfillPattern, CoordinateScale) -> f64,
    Override: FnOnce(f64, &RegionOptions, f64) -> f64,
{
    let CandidateBridgeAngleInput {
        area_to_be_bridge,
        anchors,
        boundary_polylines,
        region,
        model_rotation_rad,
        scale,
    } = input;
    let (polylines, pattern) = if anchors.is_empty() {
        (boundary_polylines, ProcessInfillPattern::Line)
    } else {
        (anchors, region.sparse_infill_pattern)
    };
    let lines = to_lines(polylines);
    let detected = detect(area_to_be_bridge, &lines, pattern, scale);
    apply_override(detected, region, model_rotation_rad)
}

fn to_lines(polylines: &[Polyline]) -> Vec<Line> {
    let line_count = polylines
        .iter()
        .filter(|polyline| polyline.points().len() > 1)
        .map(|polyline| polyline.points().len() - 1)
        .sum();
    let mut lines = Vec::with_capacity(line_count);
    for polyline in polylines {
        lines.extend(
            polyline
                .points()
                .windows(2)
                .map(|points| Line::new(points[0], points[1])),
        );
    }
    lines
}
