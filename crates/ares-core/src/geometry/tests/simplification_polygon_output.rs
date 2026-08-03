use super::super::{ExPolygon, FillRule, Point, Polygon, simplify_expolygon_polygons, union_ex};

#[test]
fn task22o15_simplify_p_returns_flat_contour_then_hole_paths() {
    let input = ExPolygon::new(
        polygon(&[(0, 0), (40, 0), (40, 40), (0, 40)]),
        vec![
            polygon(&[(5, 5), (5, 15), (15, 15), (15, 5)]),
            polygon(&[(20, 20), (20, 30), (30, 30), (30, 20)]),
        ],
    );
    let output = simplify_expolygon_polygons(&input, 0.1).unwrap();
    assert_eq!(output.len(), 3);
    assert!(output[0].area().abs() > output[1].area().abs());
    assert_eq!(output[1].area().abs(), output[2].area().abs());
}

#[test]
fn task22o15_simplify_p_defers_polytree_grouping_until_aggregate_union() {
    let first = ExPolygon::new(polygon(&[(0, 0), (20, 0), (20, 20), (0, 20)]), vec![]);
    let second = ExPolygon::new(polygon(&[(10, 0), (30, 0), (30, 20), (10, 20)]), vec![]);
    let mut paths = simplify_expolygon_polygons(&first, 0.1).unwrap();
    paths.extend(simplify_expolygon_polygons(&second, 0.1).unwrap());
    assert_eq!(paths.len(), 2);
    assert_eq!(union_ex(&paths, FillRule::NonZero).unwrap().len(), 1);
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
