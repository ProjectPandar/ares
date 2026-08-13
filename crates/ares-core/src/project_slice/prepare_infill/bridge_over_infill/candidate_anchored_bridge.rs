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
    geometry::{
        ClipperError, CoordinateScale, Line, Polygon, Polyline, intersection_open_polylines,
        intersection_polygons_paths,
    },
    project_slice::perimeters::types::Flow,
};

use super::anchored_polygon::construct_anchored_polygon;

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct CandidateAnchoredBridge {
    pub(in crate::project_slice) boundary_polylines: Vec<Polyline>,
    pub(in crate::project_slice) bridging_area: Vec<Polygon>,
}

struct CandidateAnchoredBridgeInput<'a> {
    area_to_be_bridge: &'a [Polygon],
    boundary_polylines: Vec<Polyline>,
    anchors: &'a [Polyline],
    lightning_area: &'a [Polygon],
    bridging_flow: Flow,
    bridging_angle: f64,
    scale: CoordinateScale,
}

#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the source candidate construction inputs without inventing ownership"
)]
pub(in crate::project_slice) fn construct_candidate_anchored_bridge(
    area_to_be_bridge: &[Polygon],
    boundary_polylines: Vec<Polyline>,
    anchors: &[Polyline],
    lightning_area: &[Polygon],
    bridging_flow: Flow,
    bridging_angle: f64,
    scale: CoordinateScale,
) -> Result<CandidateAnchoredBridge, ClipperError> {
    construct_candidate_anchored_bridge_using(
        CandidateAnchoredBridgeInput {
            area_to_be_bridge,
            boundary_polylines,
            anchors,
            lightning_area,
            bridging_flow,
            bridging_angle,
            scale,
        },
        intersection_polygons_paths,
        |paths, delta| {
            crate::geometry::offset_paths(paths, delta, crate::geometry::JoinType::Miter, 3.0)
        },
        intersection_open_polylines,
        construct_anchored_polygon,
    )
}

fn construct_candidate_anchored_bridge_using<Closed, Expand, Open, Construct>(
    input: CandidateAnchoredBridgeInput<'_>,
    closed_intersection: Closed,
    expand: Expand,
    open_intersection: Open,
    construct: Construct,
) -> Result<CandidateAnchoredBridge, ClipperError>
where
    Closed: FnOnce(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
    Expand: FnOnce(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
    Open: FnOnce(&[Polyline], &[Polygon]) -> Result<Vec<Polyline>, ClipperError>,
    Construct: FnOnce(
        &[Polygon],
        &[Line],
        Flow,
        f64,
        CoordinateScale,
    ) -> Result<Vec<Polygon>, ClipperError>,
{
    let CandidateAnchoredBridgeInput {
        area_to_be_bridge,
        mut boundary_polylines,
        anchors,
        lightning_area,
        bridging_flow,
        bridging_angle,
        scale,
    } = input;
    boundary_polylines.extend_from_slice(anchors);

    if !lightning_area.is_empty() {
        let overlap = closed_intersection(area_to_be_bridge, lightning_area)?;
        if !overlap.is_empty() {
            let delta = (10.0_f64 / scale.factor()) as f32;
            let expanded_area = expand(area_to_be_bridge, delta)?;
            boundary_polylines = open_intersection(&boundary_polylines, &expanded_area)?;
        }
    }

    let lines = to_lines(&boundary_polylines);
    let bridging_area = construct(
        area_to_be_bridge,
        &lines,
        bridging_flow,
        bridging_angle,
        scale,
    )?;
    Ok(CandidateAnchoredBridge {
        boundary_polylines,
        bridging_area,
    })
}

pub(super) fn to_lines(polylines: &[Polyline]) -> Vec<Line> {
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
