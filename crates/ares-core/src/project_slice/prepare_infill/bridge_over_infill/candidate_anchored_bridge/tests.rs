mod operation_order;
mod production;

use super::{
    CandidateAnchoredBridgeInput, construct_candidate_anchored_bridge,
    construct_candidate_anchored_bridge_using,
};
use crate::{
    geometry::{CoordinateScale, Line, Point, Polygon, Polyline},
    project_slice::perimeters::types::Flow,
};

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn polyline(points: &[(i64, i64)]) -> Polyline {
    Polyline::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn line(ax: i64, ay: i64, bx: i64, by: i64) -> Line {
    Line::new(Point::new(ax, ay), Point::new(bx, by))
}

fn flow() -> Flow {
    Flow {
        width: 0.4,
        height: 0.4,
        spacing: (f64::from(0.4_f32) + 0.05) as f32,
        nozzle_diameter: 0.4,
        bridge: true,
        mm3_per_mm: 1.0,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "keeps injected operation inputs explicit at test call sites"
)]
fn input<'a>(
    area: &'a [Polygon],
    boundaries: Vec<Polyline>,
    anchors: &'a [Polyline],
    lightning: &'a [Polygon],
    flow: Flow,
    angle: f64,
    scale: CoordinateScale,
) -> CandidateAnchoredBridgeInput<'a> {
    CandidateAnchoredBridgeInput {
        area_to_be_bridge: area,
        boundary_polylines: boundaries,
        anchors,
        lightning_area: lightning,
        bridging_flow: flow,
        bridging_angle: angle,
        scale,
    }
}

fn snapshot_polygons(polygons: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
    polygons
        .iter()
        .map(|polygon| {
            polygon
                .points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}

fn snapshot_polylines(polylines: &[Polyline]) -> Vec<Vec<(i64, i64)>> {
    polylines
        .iter()
        .map(|polyline| {
            polyline
                .points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}
