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
fn task22d_spatial_strict_radius_is_exact_at_extreme_coordinates() {
    let query = point(i64::MAX - 4, i64::MIN + 4);
    assert_eq!(
        distance_squared_inside(query, point(i64::MAX - 1, i64::MIN), 5),
        None
    );
    assert_eq!(
        distance_squared_inside(query, point(i64::MAX - 1, i64::MIN + 1), 5),
        Some(18)
    );

    for radius in [2_000_000, 199_999] {
        let query = point(i64::MAX - radius, i64::MIN + radius);
        assert_eq!(
            distance_squared_inside(query, point(i64::MAX, i64::MIN + radius), radius),
            None
        );
        assert_eq!(
            distance_squared_inside(query, point(i64::MAX - 1, i64::MIN + radius), radius,),
            Some((radius as u128 - 1).pow(2))
        );
    }
}

#[test]
fn task22d_spatial_three_by_three_grid_covers_negative_cell_boundaries() {
    let mut grid = RadiusGrid::new(10);
    let first = key(0, EndpointSide::Start);
    grid.insert(first, point(0, 0));
    assert_eq!(grid.find(point(-1, -1), |_| true).unwrap().key, first);
    assert_eq!(
        grid.find(point(-1, -1), |_| true).unwrap().distance_squared,
        2
    );

    grid.remove(first);
    let second = key(1, EndpointSide::Start);
    grid.insert(second, point(-11, -11));
    assert_eq!(grid.find(point(-10, -10), |_| true).unwrap().key, second);
}

#[test]
fn task22d_spatial_three_by_three_grid_queries_all_nine_cells() {
    let coordinates = |offset| match offset {
        -1 => (0, -1),
        0 => (5, 5),
        1 => (9, 10),
        _ => unreachable!(),
    };

    for y_offset in -1..=1 {
        for x_offset in -1..=1 {
            let (query_x, candidate_x) = coordinates(x_offset);
            let (query_y, candidate_y) = coordinates(y_offset);
            let mut grid = RadiusGrid::new(10);
            let candidate = key(0, EndpointSide::Start);
            grid.insert(candidate, point(candidate_x, candidate_y));

            let nearest = grid.find(point(query_x, query_y), |_| true).unwrap();
            assert_eq!(
                nearest.key, candidate,
                "cell offset ({x_offset}, {y_offset})"
            );
            assert_eq!(
                nearest.distance_squared,
                u128::from(x_offset != 0) + u128::from(y_offset != 0),
                "cell offset ({x_offset}, {y_offset})"
            );
        }
    }
}

#[test]
fn task22d_spatial_equal_distance_uses_index_then_start_before_end() {
    let mut grid = RadiusGrid::new(5);
    let index_two_start = key(2, EndpointSide::Start);
    let index_one_end = key(1, EndpointSide::End);
    let index_one_start = key(1, EndpointSide::Start);
    grid.insert(index_two_start, point(-3, 0));
    grid.insert(index_one_end, point(3, 0));
    grid.insert(index_one_start, point(0, -3));

    assert_eq!(
        grid.find(point(0, 0), |_| true).unwrap().key,
        index_one_start
    );
    assert!(grid.remove(index_one_start));
    assert_eq!(grid.find(point(0, 0), |_| true).unwrap().key, index_one_end);
    assert!(grid.remove(index_one_end));
    assert_eq!(
        grid.find(point(0, 0), |_| true).unwrap().key,
        index_two_start
    );
}

#[test]
fn task22d_spatial_removal_reinsertion_and_activity_are_request_local() {
    let mut grid = RadiusGrid::new(20);
    let changing_end = key(0, EndpointSide::End);
    let active_start = key(1, EndpointSide::Start);
    grid.insert(changing_end, point(10, 0));
    assert_eq!(grid.len(), 1);
    assert!(grid.remove(changing_end));
    assert!(grid.find(point(0, 0), |_| true).is_none());

    grid.insert(changing_end, point(-1, 0));
    grid.insert(active_start, point(1, 0));
    assert_eq!(grid.len(), 2);
    assert_eq!(
        grid.find(point(0, 0), |candidate| candidate != 0)
            .unwrap()
            .key,
        active_start
    );
    assert_eq!(grid.len(), 2);
}

#[test]
fn task22d_spatial_dense_cell_queries_are_repeatable_and_nonmutating() {
    let mut grid = RadiusGrid::new(100);
    for original_index in 0..1_024 {
        grid.insert(key(original_index, EndpointSide::Start), point(1, 1));
        grid.insert(key(original_index, EndpointSide::End), point(1, 1));
    }
    let expected = key(0, EndpointSide::Start);
    let count = grid.len();

    for _ in 0..8 {
        assert_eq!(grid.find(point(0, 0), |_| true).unwrap().key, expected);
        assert_eq!(grid.len(), count);
    }
}
