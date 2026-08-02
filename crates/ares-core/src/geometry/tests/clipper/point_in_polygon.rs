use crate::geometry::{Point, clipper::point_in_polygon};

fn p(x: i64, y: i64) -> Point {
    Point::new(x, y)
}

#[test]
fn task22o4_point_in_polygon_preserves_clipper_tristate_edges() {
    let square = [p(0, 0), p(10, 0), p(10, 10), p(0, 10)];
    assert_eq!(point_in_polygon(p(5, 5), &square), 1);
    assert_eq!(point_in_polygon(p(11, 5), &square), 0);
    for point in [p(0, 0), p(5, 0), p(10, 5), p(5, 10), p(0, 5)] {
        assert_eq!(point_in_polygon(point, &square), -1);
    }
}

#[test]
fn task22o4_point_in_polygon_handles_winding_concavity_and_degenerate_paths() {
    let concave = [p(0, 0), p(8, 0), p(8, 8), p(4, 4), p(0, 8)];
    assert_eq!(point_in_polygon(p(2, 2), &concave), 1);
    assert_eq!(point_in_polygon(p(4, 6), &concave), 0);
    assert_eq!(point_in_polygon(p(6, 6), &concave), -1);
    let reversed = concave.into_iter().rev().collect::<Vec<_>>();
    assert_eq!(point_in_polygon(p(2, 2), &reversed), 1);
    assert_eq!(point_in_polygon(p(1, 1), &[p(0, 0), p(2, 2)]), 0);
}

#[test]
fn task22o4_point_in_polygon_uses_source_cross_product_at_large_coordinates() {
    let base = 4_000_000_000_000_000_000_i64;
    let triangle = [p(base, base), p(base + 1000, base + 2), p(base, base + 4)];
    assert_eq!(point_in_polygon(p(base + 499, base + 1), &triangle), 1);
    assert_eq!(point_in_polygon(p(base + 500, base + 1), &triangle), -1);
    assert_eq!(point_in_polygon(p(base + 501, base + 1), &triangle), 0);

    let wide = 1_i64 << 60;
    let floating_boundary = [p(-wide, -wide), p(wide + 1, wide), p(wide + 1, -wide)];
    assert_eq!(point_in_polygon(p(0, 0), &floating_boundary), -1);
}
