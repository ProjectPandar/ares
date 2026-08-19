use super::order_compensation_polytree;
use crate::geometry::{ExPolygon, Point, Polygon};

#[test]
fn compensation_polytree_orders_outer_siblings_and_holes_deterministically() {
    let holes = vec![square(20, 10, 1), square(10, 20, 1), square(0, 20, 1)];
    let mut expolygons = vec![
        ExPolygon::new(square(0, 0, 2), Vec::new()),
        ExPolygon::new(square(0, 0, 4), holes),
    ];

    order_compensation_polytree(&mut expolygons);

    assert_eq!(
        expolygons
            .iter()
            .map(|expolygon| expolygon.contour().area())
            .collect::<Vec<_>>(),
        vec![16.0, 4.0]
    );
    assert_eq!(
        expolygons[0]
            .holes()
            .iter()
            .map(|hole| hole.points()[0])
            .collect::<Vec<_>>(),
        vec![Point::new(0, 20), Point::new(10, 20), Point::new(20, 10)]
    );
}

fn square(x: i64, y: i64, side: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + side, y),
        Point::new(x + side, y + side),
        Point::new(x, y + side),
    ])
}
