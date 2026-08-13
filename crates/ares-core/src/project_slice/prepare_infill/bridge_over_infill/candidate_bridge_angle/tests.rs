mod operation_order;
mod production;

use super::{CandidateBridgeAngleInput, determine_candidate_bridge_angle};
use crate::{
    ProjectSettings, RegionOptions,
    geometry::{CoordinateScale, Line, Point, Polygon, Polyline},
};

fn region() -> RegionOptions {
    RegionOptions::from_base(&ProjectSettings::default().process.region)
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn polyline(points: &[(i64, i64)]) -> Polyline {
    Polyline::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn line(ax: i64, ay: i64, bx: i64, by: i64) -> Line {
    Line::new(Point::new(ax, ay), Point::new(bx, by))
}

#[expect(
    clippy::too_many_arguments,
    reason = "keeps injected operation inputs visible at each test call"
)]
fn input<'a>(
    area: &'a [Polygon],
    anchors: &'a [Polyline],
    boundaries: &'a [Polyline],
    region: &'a RegionOptions,
    rotation: f64,
    scale: CoordinateScale,
) -> CandidateBridgeAngleInput<'a> {
    CandidateBridgeAngleInput {
        area_to_be_bridge: area,
        anchors,
        boundary_polylines: boundaries,
        region,
        model_rotation_rad: rotation,
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
