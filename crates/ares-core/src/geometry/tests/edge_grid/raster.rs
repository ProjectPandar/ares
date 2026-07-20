use super::super::super::{
    Point,
    edge_grid::raster::{RasterGrid, visit_line},
};

fn raster(p1: (i64, i64), p2: (i64, i64)) -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    visit_line(
        RasterGrid {
            bounds_min: Point::new(0, 0),
            resolution: 10,
            rows: 5,
            cols: 5,
        },
        Point::new(p1.0, p1.1),
        Point::new(p2.0, p2.1),
        |row, col| {
            cells.push((row, col));
            true
        },
    )
    .unwrap();
    cells
}

#[test]
fn task22m_edge_grid_raster_handles_same_cell_and_axis_aligned_edges() {
    assert_eq!(raster((1, 1), (9, 9)), [(0, 0)]);
    assert_eq!(raster((5, 20), (35, 20)), [(2, 0), (2, 1), (2, 2), (2, 3)]);
    assert_eq!(raster((35, 20), (5, 20)), [(2, 3), (2, 2), (2, 1), (2, 0)]);
    assert_eq!(raster((20, 5), (20, 35)), [(0, 2), (1, 2), (2, 2), (3, 2)]);
    assert_eq!(raster((20, 35), (20, 5)), [(3, 2), (2, 2), (1, 2), (0, 2)]);
}

#[test]
fn task22m_edge_grid_raster_freezes_corner_crossings_in_all_quadrants() {
    assert_eq!(raster((5, 5), (35, 35)), [(0, 0), (1, 1), (2, 2), (3, 3)]);
    assert_eq!(raster((35, 35), (5, 5)), [(3, 3), (2, 2), (1, 1), (0, 0)]);
    assert_eq!(
        raster((5, 35), (35, 5)),
        [(3, 0), (3, 1), (2, 1), (2, 2), (1, 2), (1, 3), (0, 3),]
    );
    assert_eq!(
        raster((35, 5), (5, 35)),
        [(0, 3), (1, 3), (1, 2), (2, 2), (2, 1), (3, 1), (3, 0),]
    );
}

#[test]
fn task22m_edge_grid_raster_freezes_non_corner_crossings_in_all_quadrants() {
    assert_eq!(
        raster((3, 6), (37, 24)),
        [(0, 0), (0, 1), (1, 1), (1, 2), (2, 2), (2, 3)]
    );
    assert_eq!(
        raster((3, 34), (37, 16)),
        [(3, 0), (3, 1), (2, 1), (2, 2), (1, 2), (1, 3)]
    );
    assert_eq!(
        raster((37, 6), (3, 24)),
        [(0, 3), (0, 2), (1, 2), (1, 1), (2, 1), (2, 0)]
    );
    assert_eq!(
        raster((37, 34), (3, 16)),
        [(3, 3), (3, 2), (2, 2), (2, 1), (1, 1), (1, 0)]
    );
}

#[test]
fn task22m_edge_grid_raster_assigns_boundary_edges_to_the_source_cells() {
    assert_eq!(
        raster((0, 20), (40, 20)),
        [(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)]
    );
    assert_eq!(
        raster((40, 20), (0, 20)),
        [(2, 4), (2, 3), (2, 2), (2, 1), (2, 0)]
    );
    assert_eq!(
        raster((20, 0), (20, 40)),
        [(0, 2), (1, 2), (2, 2), (3, 2), (4, 2)]
    );
    assert_eq!(
        raster((20, 40), (20, 0)),
        [(4, 2), (3, 2), (2, 2), (1, 2), (0, 2)]
    );
}
