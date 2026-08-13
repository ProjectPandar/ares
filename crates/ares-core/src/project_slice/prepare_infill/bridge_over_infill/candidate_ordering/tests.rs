mod bounds;
mod order;
mod ownership;

use crate::geometry::{Point, Polygon};

use super::super::types::{CandidateSource, CandidateSurface};

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

fn candidate(id: usize, polygons: Vec<Polygon>) -> CandidateSurface {
    CandidateSurface {
        source: CandidateSource {
            layer_index: 7,
            region_index: id % 3,
            surface_index: id,
        },
        new_polygons: polygons,
        bridge_angle: id as f64 + 0.125,
    }
}

fn ids(candidates: &[CandidateSurface]) -> Vec<usize> {
    candidates
        .iter()
        .map(|candidate| candidate.source.surface_index)
        .collect()
}
