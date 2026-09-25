//! Path-endpoint coordinate helpers split from `chain.rs` to keep each source
//! file under the LOC cap (`ShortestPath.cpp` endpoint extraction port).
use crate::project_slice::perimeters::classic::materialize::ExtrusionPath;

pub(super) fn coord_to_f64(point: [crate::geometry::Coord; 2]) -> [f64; 2] {
    [point[0] as f64, point[1] as f64]
}

pub(super) fn first_coord(path: &ExtrusionPath) -> [crate::geometry::Coord; 2] {
    let point = path.polyline.points.first().expect("O7 paths are valid");
    [point.x, point.y]
}

pub(super) fn last_coord(path: &ExtrusionPath) -> [crate::geometry::Coord; 2] {
    let point = path.polyline.points.last().expect("O7 paths are valid");
    [point.x, point.y]
}

pub(super) fn first_xy(path: &ExtrusionPath) -> [f64; 2] {
    coord_to_f64(first_coord(path))
}

pub(super) fn last_xy(path: &ExtrusionPath) -> [f64; 2] {
    coord_to_f64(last_coord(path))
}

pub(super) fn squared_coord_delta(
    first: [crate::geometry::Coord; 2],
    second: [crate::geometry::Coord; 2],
) -> f64 {
    let dx = (first[0] - second[0]) as f64;
    let dy = (first[1] - second[1]) as f64;
    dx * dx + dy * dy
}
