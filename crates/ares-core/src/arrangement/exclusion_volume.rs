//! `bed_exclusion_arrange_polygon` (`src/libslic3r/ExclusionVolumeGeometry.cpp:70-83` at
//! v2.5.0-beta `345efec64`).

use super::{BoundingBox, Point, Polygon};
use crate::geometry::convex_hull;

/// The arrange blocker of a bed-exclusion polygon: a convex one exact, a concave one its bounding
/// box.
pub fn bed_exclusion_arrange_polygon(polygon: &Polygon) -> Polygon {
    if polygon.points().len() < 3 {
        return polygon.clone();
    }
    let hull = Polygon::new(convex_hull(polygon.points()));
    let (polygon_area, hull_area) = (polygon.area().abs(), hull.area().abs());
    let tolerance = hull_area.max(1.0) * 1e-12;
    if (hull_area - polygon_area).abs() <= tolerance {
        return polygon.clone();
    }
    // `get_extents(polygon).polygon()`.
    let bounds = BoundingBox::from_expolygon(&super::ExPolygon::new(polygon.clone(), Vec::new()))
        .expect("polygon has points");
    let (min, max) = (bounds.min(), bounds.max());
    Polygon::new(vec![
        min,
        Point::new(max.x(), min.y()),
        max,
        Point::new(min.x(), max.y()),
    ])
}
