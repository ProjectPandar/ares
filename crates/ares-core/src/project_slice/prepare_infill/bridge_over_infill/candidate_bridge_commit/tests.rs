mod append;
mod replace;

use super::*;
use crate::geometry::{Point, Polygon, Polyline};

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

fn source(layer_index: usize, region_index: usize, surface_index: usize) -> CandidateSource {
    CandidateSource {
        layer_index,
        region_index,
        surface_index,
    }
}

fn postprocessed(
    bridging_area: Vec<Polygon>,
    bridging_angle: f64,
    expansion_area: Vec<Polygon>,
) -> PostprocessedCandidateBridge {
    PostprocessedCandidateBridge {
        boundary_polylines: vec![Polyline::new(vec![Point::new(0, 0), Point::new(1, 1)])],
        bridging_area,
        bridging_angle,
        expansion_area,
    }
}

fn candidate(source: CandidateSource, x: i64, angle: f64) -> CandidateSurface {
    CandidateSurface {
        source,
        new_polygons: vec![rectangle(x, 0, x + 10, 10)],
        bridge_angle: angle,
    }
}

type CandidateSnapshot = (CandidateSource, u64, Vec<Vec<(i64, i64)>>);

fn snapshot(candidates: &[CandidateSurface]) -> Vec<CandidateSnapshot> {
    candidates
        .iter()
        .map(|candidate| {
            (
                candidate.source,
                candidate.bridge_angle.to_bits(),
                candidate
                    .new_polygons
                    .iter()
                    .map(|polygon| {
                        polygon
                            .points()
                            .iter()
                            .map(|point| (point.x(), point.y()))
                            .collect()
                    })
                    .collect(),
            )
        })
        .collect()
}
