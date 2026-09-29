use super::{DistanceField, RADIUS_PER_CELL_SIZE, UnsupportedCell, UnsupportedPointsGrid};
use crate::geometry::{BoundingBox, Point};

const RADIUS: i64 = 1000;

fn field(locs: &[Point]) -> DistanceField {
    let cell_size = RADIUS / RADIUS_PER_CELL_SIZE;
    let bbox = BoundingBox::new(Point::new(0, 0), Point::new(20_000, 20_000));
    let unsupported_points: Vec<UnsupportedCell> = locs
        .iter()
        .map(|&loc| UnsupportedCell {
            loc,
            dist_to_boundary: 0,
        })
        .collect();
    let unsupported_points_grid = UnsupportedPointsGrid::initialize(&unsupported_points, |p| {
        Point::new(p.x() / cell_size, p.y() / cell_size)
    });
    DistanceField {
        cell_size,
        supporting_radius: RADIUS,
        supporting_radius2: RADIUS * RADIUS,
        unsupported_points_erased: vec![false; unsupported_points.len()],
        unsupported_points,
        unsupported_points_bbox: bbox,
        unsupported_points_grid,
    }
}

/// `DistanceField.cpp:130-136`: a grid location whose projection falls
/// outside `[0, l2]` skips only the rectangle test and still reaches the
/// erase check, which is gated on the cell's own `loc` lying within the
/// supporting radius of the new leaf.
#[test]
fn update_erases_in_radius_cell_whose_grid_corner_projects_past_the_branch() {
    // Grid corner (3984, 4980) is 1016 from the leaf (outside the circle)
    // and projects beyond `l2` along to_node→leaf; the cell's loc is 990
    // from the leaf.
    let mut field = field(&[Point::new(4010, 5000)]);
    field.update(Point::new(10_000, 5000), Point::new(5000, 5000));
    assert_eq!(field.try_get_next_point(0), None);
}

/// Upstream has no zero-length early return: `extent` is NaN, the
/// rectangle test's `d < -1 || d > 1` is false, and every cell within the
/// radius of the leaf is erased.
#[test]
fn update_with_zero_length_branch_erases_cells_within_radius() {
    let mut field = field(&[Point::new(5500, 5000), Point::new(8000, 5000)]);
    field.update(Point::new(5000, 5000), Point::new(5000, 5000));
    assert_eq!(
        field.try_get_next_point(0),
        Some((1, Point::new(8000, 5000)))
    );
}
