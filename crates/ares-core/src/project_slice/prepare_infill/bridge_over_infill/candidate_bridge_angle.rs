use crate::{
    RegionOptions,
    geometry::{CoordinateScale, Line, Polygon, Polyline},
};

use super::{
    automatic_bridge_angle::determine_automatic_bridge_angle,
    internal_bridge_angle::apply_internal_bridge_angle_override,
};

pub(in crate::project_slice) struct CandidateBridgeAngleInput<'a> {
    pub(in crate::project_slice) area_to_be_bridge: &'a [Polygon],
    pub(in crate::project_slice) anchors: &'a [Polyline],
    pub(in crate::project_slice) boundary_polylines: &'a [Polyline],
    pub(in crate::project_slice) region: &'a RegionOptions,
    pub(in crate::project_slice) model_rotation_rad: f64,
    pub(in crate::project_slice) configured_angle: Option<f64>,
    pub(in crate::project_slice) scale: CoordinateScale,
}

pub(in crate::project_slice) fn determine_candidate_bridge_angle(
    input: CandidateBridgeAngleInput<'_>,
) -> f64 {
    determine_candidate_bridge_angle_using(
        input,
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
    Detect: FnOnce(&[Polygon], &[Line], CoordinateScale) -> f64,
    Override: FnOnce(f64, &RegionOptions, f64) -> f64,
{
    let CandidateBridgeAngleInput {
        area_to_be_bridge,
        anchors,
        boundary_polylines,
        region,
        model_rotation_rad,
        configured_angle,
        scale,
    } = input;
    let detected = configured_angle.unwrap_or_else(|| {
        let polylines = if anchors.is_empty() {
            boundary_polylines
        } else {
            anchors
        };
        detect(area_to_be_bridge, &to_lines(polylines), scale)
    });
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
