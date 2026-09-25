use crate::geometry::Point;

use super::{EndpointKey, EndpointSide, RadiusGrid, distance_squared_inside};

fn point(x: i64, y: i64) -> Point {
    Point::new(x, y)
}

fn key(original_index: usize, side: EndpointSide) -> EndpointKey {
    EndpointKey {
        original_index,
        side,
    }
}

#[test]
fn task22d_spatial_radius_check_is_strict() {
    // `find` returns a candidate only when `dist < search_radius^2`.
    let mut grid = RadiusGrid::new(10);
    let far = key(0, EndpointSide::Start);
    // distance exactly 10 -> rejected (strict less-than on the squared bound).
    grid.insert(far, point(10, 0));
    assert!(grid.find(point(0, 0), |_| true).is_none());
    // distance < 10 -> accepted.
    let near = key(1, EndpointSide::Start);
    grid.insert(near, point(3, 0));
    assert!(grid.find(point(0, 0), |_| true).is_some());
}

#[test]
fn task22d_spatial_find_returns_squared_distance_as_f64() {
    let mut grid = RadiusGrid::new(16);
    let candidate = key(0, EndpointSide::Start);
    grid.insert(candidate, point(3, 4));
    let nearest = grid.find(point(0, 0), |_| true).unwrap();
    assert_eq!(nearest.distance_squared, 25.0);
}

#[test]
fn task22d_spatial_distance_squared_inside_reports_f64() {
    assert_eq!(
        distance_squared_inside(point(0, 0), point(3, 4), 10),
        Some(25.0)
    );
    // Strict bound: dist^2 == radius^2 -> None.
    assert_eq!(distance_squared_inside(point(0, 0), point(10, 0), 10), None);
}

#[test]
fn task22d_spatial_remove_then_reinsert_is_request_local() {
    let mut grid = RadiusGrid::new(20);
    let changing_end = key(0, EndpointSide::End);
    let active_start = key(1, EndpointSide::Start);
    grid.insert(changing_end, point(1, 0));
    grid.insert(active_start, point(2, 0));
    assert_eq!(grid.len(), 2);
    assert!(grid.remove(changing_end));
    // The removed entry is skipped by find even if it is geometrically closer.
    assert_eq!(grid.find(point(0, 0), |_| true).unwrap().key, active_start);
    assert_eq!(grid.len(), 1);
}

#[test]
fn task22d_spatial_closest_candidate_wins() {
    let mut grid = RadiusGrid::new(64);
    let near = key(0, EndpointSide::Start);
    let far = key(1, EndpointSide::Start);
    grid.insert(far, point(20, 0));
    grid.insert(near, point(4, 0));
    assert_eq!(grid.find(point(0, 0), |_| true).unwrap().key, near);
}

#[test]
fn task22d_spatial_out_of_radius_is_rejected() {
    let mut grid = RadiusGrid::new(8);
    grid.insert(key(0, EndpointSide::Start), point(100, 100));
    assert!(grid.find(point(0, 0), |_| true).is_none());
}
