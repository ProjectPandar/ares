use crate::{
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::Point,
};

use super::stitch_toolpaths;

#[test]
fn task22o196_stitches_even_rectangle_edges_into_one_closed_line() {
    let corners = [
        Point::new(0, 0),
        Point::new(1_000, 0),
        Point::new(1_000, 1_000),
        Point::new(0, 1_000),
        Point::new(0, 0),
    ];
    let lines = corners
        .windows(2)
        .map(|edge| {
            let mut line = ExtrusionLine::new(0, false);
            line.push(ExtrusionJunction::new(edge[0], 400, 0));
            line.push(ExtrusionJunction::new(edge[1], 400, 0));
            line
        })
        .collect();
    let mut toolpaths = vec![lines];

    stitch_toolpaths(&mut toolpaths, 100, 10);

    assert_eq!(toolpaths[0].len(), 1);
    assert!(toolpaths[0][0].is_closed);
    assert_eq!(toolpaths[0][0].junctions.len(), 5);
    assert_eq!(
        toolpaths[0][0].junctions.first().unwrap().point,
        toolpaths[0][0].junctions.last().unwrap().point
    );
}
