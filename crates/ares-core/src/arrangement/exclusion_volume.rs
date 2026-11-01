//! `bed_exclusion_arrange_polygon` (`src/libslic3r/ExclusionVolumeGeometry.cpp:70-83` at
//! v2.5.0-beta `345efec64`).

use super::Polygon;

/// The arrange blocker of a bed-exclusion polygon: a convex one exact, a concave one its bounding
/// box.
pub fn bed_exclusion_arrange_polygon(_polygon: &Polygon) -> Polygon {
    todo!("bed_exclusion_arrange_polygon")
}
