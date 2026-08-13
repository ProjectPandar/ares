mod clustering;
mod geometry;

use crate::geometry::{Point, Polygon};

use super::CandidateClusterLayer;
use crate::project_slice::prepare_infill::bridge_over_infill::types::{
    CandidateSource, CandidateSurface,
};

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn square(min_x: i64, min_y: i64, size: i64) -> Polygon {
    polygon(&[
        (min_x, min_y),
        (min_x + size, min_y),
        (min_x + size, min_y + size),
        (min_x, min_y + size),
    ])
}

fn candidate(layer: usize, region: usize, polygons: Vec<Polygon>) -> CandidateSurface {
    CandidateSurface {
        source: CandidateSource {
            layer_index: layer,
            region_index: region,
            surface_index: 0,
        },
        new_polygons: polygons,
        bridge_angle: -1.0,
    }
}

fn view<'a>(
    layer_index: usize,
    print_z: f64,
    bridge_height: f32,
    candidates: &'a [CandidateSurface],
) -> CandidateClusterLayer<'a> {
    CandidateClusterLayer {
        layer_index,
        print_z,
        bridge_height,
        candidates,
    }
}

fn points(polygon: &Polygon) -> Vec<(i64, i64)> {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

type CandidateSnapshot = (CandidateSource, u64, Vec<Vec<(i64, i64)>>);

fn snapshot(candidates: &[CandidateSurface]) -> Vec<CandidateSnapshot> {
    candidates
        .iter()
        .map(|candidate| {
            (
                candidate.source,
                candidate.bridge_angle.to_bits(),
                candidate.new_polygons.iter().map(points).collect(),
            )
        })
        .collect()
}
