use super::super::super::{
    ClipperError, ClosestPointResult, EdgeGrid, ExPolygon, GridEdge, Point, Polygon,
};

fn square() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(20, 0),
            Point::new(20, 20),
            Point::new(0, 20),
        ]),
        Vec::new(),
    )
}

fn square_grid() -> EdgeGrid {
    EdgeGrid::new(&square(), Point::new(0, 0), Point::new(20, 20), 10).unwrap()
}

const fn edge(contour_index: usize, segment_index: usize) -> GridEdge {
    GridEdge {
        contour_index,
        segment_index,
    }
}

#[test]
fn task22o44_edge_grid_closest_query_preserves_signed_distance_and_normalized_t() {
    let grid = square_grid();

    assert_eq!(
        grid.closest_point_signed_distance(Point::new(7, 2), 5)
            .unwrap(),
        Some(ClosestPointResult {
            contour_index: 0,
            segment_index: 0,
            distance: -2.0,
            t: 0.35,
        })
    );
    assert_eq!(
        grid.closest_point_signed_distance(Point::new(7, -2), 5)
            .unwrap(),
        Some(ClosestPointResult {
            contour_index: 0,
            segment_index: 0,
            distance: 2.0,
            t: 0.35,
        })
    );
}

#[test]
fn task22o44_edge_grid_closest_query_rejects_the_exact_search_radius() {
    assert_eq!(
        square_grid()
            .closest_point_signed_distance(Point::new(7, 5), 5)
            .unwrap(),
        None
    );
}

#[test]
fn task22o44_edge_grid_closest_query_keeps_segment_end_and_first_win_ownership() {
    let grid = square_grid();
    assert_eq!(
        grid.closest_point_signed_distance(Point::new(20, 0), 1)
            .unwrap(),
        Some(ClosestPointResult {
            contour_index: 0,
            segment_index: 0,
            distance: 0.0,
            t: 1.0,
        })
    );
    assert_eq!(
        grid.closest_point_signed_distance(Point::new(-2, 0), 3)
            .unwrap(),
        Some(ClosestPointResult {
            contour_index: 0,
            segment_index: 3,
            distance: 2.0,
            t: 1.0,
        })
    );
}

#[test]
fn task22o44_edge_grid_closest_query_assigns_convex_and_reflex_vertex_wedges() {
    let convex = square_grid()
        .closest_point_signed_distance(Point::new(22, -2), 3)
        .unwrap()
        .unwrap();
    assert_eq!(convex.contour_index, 0);
    assert_eq!(convex.segment_index, 1);
    assert_eq!(convex.distance, 8.0f64.sqrt());
    assert_eq!(convex.t, 0.0);

    let reflex_polygon = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(20, 0),
            Point::new(20, 20),
            Point::new(10, 10),
            Point::new(0, 20),
        ]),
        Vec::new(),
    );
    let reflex_grid =
        EdgeGrid::new(&reflex_polygon, Point::new(0, 0), Point::new(20, 20), 10).unwrap();
    let reflex = reflex_grid
        .closest_point_signed_distance(Point::new(11, 8), 3)
        .unwrap()
        .unwrap();
    assert_eq!(reflex.contour_index, 0);
    assert_eq!(reflex.segment_index, 3);
    assert_eq!(reflex.distance, -5.0f64.sqrt());
    assert_eq!(reflex.t, 0.0);
}

#[test]
fn task22o44_edge_grid_closest_query_preserves_hole_sign_and_cell_first_win() {
    let hole = Polygon::new(vec![
        Point::new(5, 5),
        Point::new(5, 15),
        Point::new(15, 15),
        Point::new(15, 5),
    ]);
    let expolygon = ExPolygon::new(square().into_parts().0, vec![hole]);
    let grid = EdgeGrid::new(&expolygon, Point::new(0, 0), Point::new(20, 20), 10).unwrap();

    assert_eq!(
        grid.closest_point_signed_distance(Point::new(7, 10), 4)
            .unwrap(),
        Some(ClosestPointResult {
            contour_index: 1,
            segment_index: 0,
            distance: 2.0,
            t: 0.5,
        })
    );
    assert_eq!(
        grid.closest_point_signed_distance(Point::new(10, 10), 11)
            .unwrap(),
        Some(ClosestPointResult {
            contour_index: 1,
            segment_index: 0,
            distance: 5.0,
            t: 0.5,
        })
    );

    let outer_grid = square_grid();
    assert_eq!(
        outer_grid
            .closest_point_signed_distance(Point::new(10, 10), 11)
            .unwrap(),
        Some(ClosestPointResult {
            contour_index: 0,
            segment_index: 0,
            distance: -10.0,
            t: 0.5,
        })
    );
}

#[test]
fn task22o44_edge_grid_closest_query_reports_checked_integer_overflow() {
    let low = i64::MIN + 16;
    let high = i64::MAX - 16;
    let expolygon = ExPolygon::new(
        Polygon::new(vec![
            Point::new(low, 0),
            Point::new(high, 0),
            Point::new(low, 1),
        ]),
        Vec::new(),
    );
    let grid = EdgeGrid::new(
        &expolygon,
        Point::new(low, 0),
        Point::new(high, 1),
        i64::MAX,
    )
    .unwrap();

    assert_eq!(
        grid.closest_point_signed_distance(Point::new(0, 0), i64::MAX),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o44_edge_grid_closest_query_keeps_inclusive_max_cell_and_scans_after_zero() {
    let low = i64::MIN + 16;
    let high = i64::MAX - 16;
    let initial_min = Point::new(-i64::MAX + 16, low);
    let initial_max = Point::new(1, high);
    let full_height = vec![Point::new(0, low), Point::new(0, high), Point::new(1, low)];
    let grid =
        EdgeGrid::new_from_contours([full_height.as_slice()], initial_min, initial_max, i64::MAX)
            .unwrap();

    assert_eq!(
        grid.closest_point_signed_distance(Point::new(-5, 5), 5),
        Err(ClipperError::CoordinateOutOfRange)
    );

    let zero_hit = vec![Point::new(-6, 5), Point::new(-4, 5), Point::new(-5, 6)];
    let grid = EdgeGrid::new_from_contours(
        [zero_hit.as_slice(), full_height.as_slice()],
        initial_min,
        initial_max,
        i64::MAX,
    )
    .unwrap();
    assert_eq!(
        grid.closest_point_signed_distance(Point::new(-5, 5), 5),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o44_edge_grid_line_query_reuses_raster_order_edges_and_early_stop() {
    let outer = vec![
        Point::new(0, 0),
        Point::new(8, 0),
        Point::new(8, 8),
        Point::new(0, 8),
    ];
    let hole = vec![
        Point::new(2, 2),
        Point::new(2, 6),
        Point::new(6, 6),
        Point::new(6, 2),
    ];
    let grid = EdgeGrid::new_from_contours(
        [outer.as_slice(), (&[] as &[Point]), hole.as_slice()],
        Point::new(0, 0),
        Point::new(8, 8),
        10,
    )
    .unwrap();
    assert_eq!(grid.contour(0), outer);
    assert_eq!(grid.contour(1), hole);

    let mut visits = Vec::new();
    grid.visit_cells_intersecting_line(
        Point::new(-5, -5),
        Point::new(15, 15),
        |row, col, edges| {
            visits.push((row, col, edges.to_vec()));
            true
        },
    )
    .unwrap();
    assert_eq!(
        visits,
        [
            (1, 1, vec![edge(0, 0), edge(0, 3), edge(1, 0), edge(1, 3)],),
            (2, 2, vec![edge(0, 1), edge(0, 2), edge(1, 1), edge(1, 2)],),
            (3, 3, vec![]),
        ]
    );

    let mut stopped = Vec::new();
    grid.visit_cells_intersecting_line(Point::new(-5, -5), Point::new(15, 15), |row, col, _| {
        stopped.push((row, col));
        stopped.len() < 2
    })
    .unwrap();
    assert_eq!(stopped, [(1, 1), (2, 2)]);
}
