mod gates;
mod operation_order;
mod production;

use super::*;
use crate::geometry::Point;
use crate::project_slice::prepare_infill::bridge_over_infill::types::CandidateSource;

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    polygon(&[
        (min_x, min_y),
        (max_x, min_y),
        (max_x, max_y),
        (min_x, max_y),
    ])
}

fn candidate(surface_index: usize, polygons: Vec<Polygon>) -> CandidateSurface {
    CandidateSurface {
        source: CandidateSource {
            layer_index: 8,
            region_index: surface_index % 2,
            surface_index,
        },
        new_polygons: polygons,
        bridge_angle: surface_index as f64 * 0.1,
    }
}

fn flow(spacing: f32) -> Flow {
    Flow {
        width: spacing * 2.0,
        height: 0.2,
        spacing,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 1.0,
    }
}

fn snapshot(polygons: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
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

fn candidate_snapshot(candidates: &[CandidateSurface]) -> Vec<Vec<Vec<(i64, i64)>>> {
    candidates
        .iter()
        .map(|candidate| snapshot(&candidate.new_polygons))
        .collect()
}
