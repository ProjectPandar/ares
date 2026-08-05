mod morphology;
mod radii;
mod transaction;

use crate::geometry::{Point, Polygon};

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}
