mod operation_order;
mod production;

use super::*;
use crate::{
    geometry::{Point, Polygon, Polyline},
    project_slice::prepare_infill::bridge_over_infill::candidate_collision_reconstruction::CollisionResolvedCandidateBridge,
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

fn collision(
    boundary_polylines: Vec<Polyline>,
    bridging_area: Vec<Polygon>,
    bridging_angle: f64,
) -> CollisionResolvedCandidateBridge {
    CollisionResolvedCandidateBridge {
        boundary_polylines,
        bridging_area,
        bridging_angle,
    }
}

fn polyline(points: &[(i64, i64)]) -> Polyline {
    Polyline::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
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

fn flow() -> Flow {
    Flow {
        width: 0.000_01,
        height: 0.000_01,
        spacing: 0.000_01,
        nozzle_diameter: 0.4,
        bridge: true,
        mm3_per_mm: 1.0,
    }
}

#[test]
fn task22o63_empty_clips_run_complete_pipeline_and_preserve_angle() {
    let output = postprocess_candidate_bridge(
        CollisionResolvedCandidateBridge {
            boundary_polylines: Vec::new(),
            bridging_area: vec![rectangle(0, 0, 100, 100)],
            bridging_angle: 0.37,
        },
        vec![rectangle(-100, -100, 200, 200)],
        &[],
        &[],
        &[],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(output.bridging_area.is_empty());
    assert_eq!(output.bridging_angle, 0.37);
    assert_eq!(
        snapshot(&output.expansion_area),
        vec![vec![(200, 200), (-100, 200), (-100, -100), (200, -100)]]
    );
}
