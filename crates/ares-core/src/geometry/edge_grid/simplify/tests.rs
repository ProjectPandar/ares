use super::*;

const RESOLUTION: Coord = 1000;

fn grid_for(contours: &[Vec<Point>]) -> EdgeGrid {
    // Upstream SupportGridPattern passes a tight, grid-aligned bbox
    // (`SupportMaterial.cpp:676-680`); mirror that here.
    EdgeGrid::new_from_contours(
        contours.iter().map(Vec::as_slice),
        Point::new(0, 0),
        Point::new(10000, 10000),
        RESOLUTION,
    )
    .unwrap()
}

fn square(min: Coord, max: Coord) -> Vec<Point> {
    // CCW outer contour.
    vec![
        Point::new(min, min),
        Point::new(max, min),
        Point::new(max, max),
        Point::new(min, max),
    ]
}

/// A grid-aligned square round-trips to a single 4-corner polygon
/// (`EdgeGrid.cpp:1283` marching squares + corner shrink).
#[test]
fn grid_aligned_square_simplifies_to_four_corners() {
    let grid = grid_for(&[square(0, 10000)]);
    let field = grid.calculate_sdf();
    let polygons = grid.contours_simplified(&field, 10, true);

    assert_eq!(polygons.len(), 1, "one contour expected");
    let points = polygons[0].points();
    assert_eq!(points.len(), 4, "grid-aligned corners only");
    // The contour stays a rectangle around the source square: each
    // corner has axis-perpendicular neighbors.
    for index in 0..4 {
        let previous = points[(index + 3) % 4];
        let following = points[(index + 1) % 4];
        let point = points[index];
        let (dx_prev, dy_prev) = (point.x() - previous.x(), point.y() - previous.y());
        let (dx_next, dy_next) = (following.x() - point.x(), following.y() - point.y());
        // Consecutive edges of the simplified rectangle are perpendicular.
        assert_ne!(dx_prev * dy_next, dy_prev * dx_next);
    }
    // Inside the source square (with the small shrink margin).
    for point in points {
        assert!(point.x() > -RESOLUTION);
        assert!(point.x() < 10000 + RESOLUTION);
        assert!(point.y() > -RESOLUTION);
        assert!(point.y() < 10000 + RESOLUTION);
    }
}

/// The signed distance field is negative inside and positive outside
/// the source contour (`EdgeGrid.cpp:672` signum + Danielsson
/// propagation).
#[test]
fn sdf_is_negative_inside_and_positive_outside() {
    let grid = grid_for(&[square(0, 10000)]);
    let field = grid.calculate_sdf();
    let (rows, cols) = grid.dimensions();
    let corner_cols = cols + 1;

    let center = field.values[(rows / 2) * corner_cols + cols / 2];
    assert!(
        center < 0.0,
        "center must be inside (negative), got {center}"
    );

    let outside = field.values[0];
    assert!(outside > 0.0, "corner of the domain must be outside");
}

/// `fill_holes` closes single-cell holes (`EdgeGrid.cpp:1298-1310`).
#[test]
fn fill_holes_closes_single_cell_holes() {
    // CW hole (reversed square), two grid cells wide.
    let mut hole = vec![
        Point::new(3000, 3000),
        Point::new(5000, 3000),
        Point::new(5000, 5000),
        Point::new(3000, 5000),
    ];
    hole.reverse();
    let grid = grid_for(&[square(0, 10000), hole]);
    let field = grid.calculate_sdf();

    let open = grid.contours_simplified(&field, 10, false);
    assert_eq!(open.len(), 2, "outer contour plus hole");

    let filled = grid.contours_simplified(&field, 10, true);
    assert_eq!(filled.len(), 1, "hole filled");
}

#[test]
fn grid_domain_is_tight() {
    let grid = grid_for(&[square(0, 10000)]);
    let (rows, cols) = grid.dimensions();
    assert_eq!((rows, cols), (11, 11));
}
