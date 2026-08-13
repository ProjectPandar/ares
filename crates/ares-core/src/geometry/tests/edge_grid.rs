use super::super::{ClipperError, EdgeGrid, ExPolygon, GridEdge, Point, Polygon};

mod query;
mod raster;

fn square_with_hole() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(20, 0),
            Point::new(20, 20),
            Point::new(0, 20),
        ]),
        vec![
            Polygon::new(Vec::new()),
            Polygon::new(vec![
                Point::new(5, 5),
                Point::new(5, 15),
                Point::new(15, 15),
                Point::new(15, 5),
            ]),
        ],
    )
}

fn compact_square_with_hole() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(8, 0),
            Point::new(8, 8),
            Point::new(0, 8),
        ]),
        vec![Polygon::new(vec![
            Point::new(2, 2),
            Point::new(2, 6),
            Point::new(6, 6),
            Point::new(6, 2),
        ])],
    )
}

const fn edge(contour_index: usize, segment_index: usize) -> GridEdge {
    GridEdge {
        contour_index,
        segment_index,
    }
}

#[test]
fn task22m_edge_grid_freezes_bounds_dimensions_and_contour_ownership() {
    let grid = EdgeGrid::new(
        &square_with_hole(),
        Point::new(0, 0),
        Point::new(20, 20),
        10,
    )
    .unwrap();

    assert_eq!(grid.bounds(), (Point::new(-16, -16), Point::new(36, 36)));
    assert_eq!(grid.resolution(), 10);
    assert_eq!(grid.dimensions(), (6, 6));
    assert_eq!(
        grid.contour(0),
        &[
            Point::new(0, 0),
            Point::new(20, 0),
            Point::new(20, 20),
            Point::new(0, 20),
        ]
    );
    assert_eq!(
        grid.contour(1),
        &[
            Point::new(5, 5),
            Point::new(5, 15),
            Point::new(15, 15),
            Point::new(15, 5),
        ]
    );
    assert_eq!(
        grid.segment(GridEdge {
            contour_index: 0,
            segment_index: 3,
        }),
        (Point::new(0, 20), Point::new(0, 0))
    );
}

#[test]
fn task22m_edge_grid_merges_initial_bounds_before_fixed_expansion() {
    let grid = EdgeGrid::new(
        &square_with_hole(),
        Point::new(-40, -30),
        Point::new(25, 50),
        10,
    )
    .unwrap();

    assert_eq!(grid.bounds(), (Point::new(-56, -46), Point::new(41, 66)));
    assert_eq!(grid.dimensions(), (12, 10));
}

#[test]
fn task22m_edge_grid_rejects_invalid_resolution_and_coordinate_growth() {
    let expolygon = square_with_hole();
    assert_eq!(
        EdgeGrid::new(&expolygon, Point::new(0, 0), Point::new(20, 20), 0),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        EdgeGrid::new(&expolygon, Point::new(i64::MIN, 0), Point::new(20, 20), 10,),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        EdgeGrid::new(&expolygon, Point::new(0, 0), Point::new(i64::MAX, 20), 10,),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22m_edge_grid_box_query_is_row_major_and_keeps_candidate_order() {
    let grid = EdgeGrid::new(
        &square_with_hole(),
        Point::new(0, 0),
        Point::new(20, 20),
        10,
    )
    .unwrap();
    let mut visits = Vec::new();

    grid.visit_cells_intersecting_box(Point::new(14, 14), Point::new(23, 23), |row, col, edges| {
        visits.push((row, col, edges.to_vec()));
        true
    });

    assert_eq!(
        visits,
        [(
            3,
            3,
            vec![
                GridEdge {
                    contour_index: 0,
                    segment_index: 1,
                },
                GridEdge {
                    contour_index: 0,
                    segment_index: 2,
                },
                GridEdge {
                    contour_index: 1,
                    segment_index: 1,
                },
                GridEdge {
                    contour_index: 1,
                    segment_index: 2,
                },
            ],
        )]
    );
}

#[test]
fn task22m_edge_grid_box_query_clamps_outside_and_stops_early() {
    let grid = EdgeGrid::new(
        &square_with_hole(),
        Point::new(0, 0),
        Point::new(20, 20),
        10,
    )
    .unwrap();
    let mut outside = Vec::new();
    grid.visit_cells_intersecting_box(
        Point::new(-100, -100),
        Point::new(-90, -90),
        |row, col, _| {
            outside.push((row, col));
            true
        },
    );
    grid.visit_cells_intersecting_box(Point::new(90, 90), Point::new(100, 100), |row, col, _| {
        outside.push((row, col));
        true
    });
    assert!(outside.is_empty());

    let mut visits = Vec::new();
    grid.visit_cells_intersecting_box(Point::new(0, 0), Point::new(20, 20), |row, col, _| {
        visits.push((row, col));
        visits.len() < 4
    });
    assert_eq!(visits, [(1, 1), (1, 2), (1, 3), (2, 1)]);
}

#[test]
fn task22m_edge_grid_box_query_exposes_only_intersected_cells() {
    let grid = EdgeGrid::new(
        &square_with_hole(),
        Point::new(0, 0),
        Point::new(20, 20),
        10,
    )
    .unwrap();
    let mut visits = Vec::new();
    grid.visit_cells_intersecting_box(Point::new(-5, -5), Point::new(3, 3), |row, col, edges| {
        visits.push((row, col, edges.to_vec()));
        true
    });

    assert_eq!(
        visits,
        [(
            1,
            1,
            vec![
                GridEdge {
                    contour_index: 0,
                    segment_index: 0,
                },
                GridEdge {
                    contour_index: 0,
                    segment_index: 3,
                },
            ],
        )]
    );
}

#[test]
fn task22m_edge_grid_two_pass_fill_freezes_all_cells_and_flattened_order() {
    let grid = EdgeGrid::new(
        &compact_square_with_hole(),
        Point::new(0, 0),
        Point::new(8, 8),
        10,
    )
    .unwrap();
    assert_eq!(grid.bounds(), (Point::new(-16, -16), Point::new(24, 24)));
    assert_eq!(grid.dimensions(), (4, 4));

    let mut cells = Vec::new();
    grid.visit_cells_intersecting_box(grid.bounds().0, grid.bounds().1, |row, col, edges| {
        cells.push((row, col, edges.to_vec()));
        true
    });
    let expected = [
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![edge(0, 0), edge(0, 3), edge(1, 0), edge(1, 3)],
        vec![edge(0, 0), edge(0, 1), edge(1, 2), edge(1, 3)],
        vec![],
        vec![],
        vec![edge(0, 2), edge(0, 3), edge(1, 0), edge(1, 1)],
        vec![edge(0, 1), edge(0, 2), edge(1, 1), edge(1, 2)],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
    ];
    assert_eq!(
        cells,
        expected
            .iter()
            .enumerate()
            .map(|(index, edges)| (index / 4, index % 4, edges.clone()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        cells
            .iter()
            .flat_map(|(_, _, edges)| edges.iter().copied())
            .collect::<Vec<_>>(),
        [
            edge(0, 0),
            edge(0, 3),
            edge(1, 0),
            edge(1, 3),
            edge(0, 0),
            edge(0, 1),
            edge(1, 2),
            edge(1, 3),
            edge(0, 2),
            edge(0, 3),
            edge(1, 0),
            edge(1, 1),
            edge(0, 1),
            edge(0, 2),
            edge(1, 1),
            edge(1, 2),
        ]
    );
}

#[test]
fn task22m_edge_grid_box_query_freezes_max_minus_one_and_signed_truncation() {
    let expolygon = ExPolygon::new(
        Polygon::new(vec![
            Point::new(16, 16),
            Point::new(24, 16),
            Point::new(24, 24),
            Point::new(16, 24),
        ]),
        Vec::new(),
    );
    let grid = EdgeGrid::new(&expolygon, Point::new(16, 16), Point::new(24, 24), 10).unwrap();
    assert_eq!(grid.bounds(), (Point::new(0, 0), Point::new(40, 40)));

    let mut boundary = Vec::new();
    grid.visit_cells_intersecting_box(Point::new(0, 0), Point::new(20, 20), |row, col, _| {
        boundary.push((row, col));
        true
    });
    assert_eq!(boundary, [(0, 0), (0, 1), (1, 0), (1, 1)]);

    let mut near_below = Vec::new();
    grid.visit_cells_intersecting_box(Point::new(-1, -1), Point::new(0, 0), |row, col, _| {
        near_below.push((row, col));
        true
    });
    assert_eq!(near_below, [(0, 0)]);

    let mut all_cells = Vec::new();
    grid.visit_cells_intersecting_box(Point::new(0, 0), Point::new(40, 40), |row, col, _| {
        all_cells.push((row, col));
        all_cells.len() < 16
    });
    assert_eq!(
        all_cells,
        (0..4)
            .flat_map(|row| (0..4).map(move |col| (row, col)))
            .collect::<Vec<_>>()
    );
}
