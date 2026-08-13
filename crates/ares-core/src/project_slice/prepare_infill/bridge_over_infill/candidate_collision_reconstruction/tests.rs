mod operation_order;
mod production;

use super::{
    CollisionResolvedCandidateBridge, reconstruct_candidate_bridge_collision,
    reconstruct_candidate_bridge_collision_using,
};
use crate::{
    geometry::{ClipperError, CoordinateScale, Point, Polygon, Polyline},
    project_slice::{
        perimeters::types::Flow,
        prepare_infill::bridge_over_infill::{
            candidate_anchored_bridge::CandidateAnchoredBridge,
            types::{CandidateSource, CandidateSurface},
        },
    },
};

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}

fn polyline(points: &[(i64, i64)]) -> Polyline {
    Polyline::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn flow() -> Flow {
    Flow {
        width: 0.4,
        height: 0.4,
        spacing: 0.45,
        nozzle_diameter: 0.4,
        bridge: true,
        mm3_per_mm: 1.0,
    }
}

fn initial(boundaries: Vec<Polyline>, bridging_area: Vec<Polygon>) -> CandidateAnchoredBridge {
    CandidateAnchoredBridge {
        boundary_polylines: boundaries,
        bridging_area,
    }
}

fn surface(index: usize, polygons: Vec<Polygon>, bridge_angle: f64) -> CandidateSurface {
    CandidateSurface {
        source: CandidateSource {
            layer_index: 7,
            region_index: 3,
            surface_index: index,
        },
        new_polygons: polygons,
        bridge_angle,
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

fn assert_range_error(result: Result<CollisionResolvedCandidateBridge, ClipperError>) {
    assert_eq!(result.unwrap_err(), ClipperError::CoordinateOutOfRange);
}
