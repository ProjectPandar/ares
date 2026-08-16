use super::inside_internal_surfaces;
use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{gcode_emit::motion::arc, region_slices::RegionSurface},
};

fn internal_square() -> RegionSurface {
    RegionSurface::internal(ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10_000_000, 0),
            Point::new(10_000_000, 10_000_000),
            Point::new(0, 10_000_000),
        ]),
        Vec::new(),
    ))
}

#[test]
fn travel_fully_inside_internal_surface_avoids_retraction() {
    assert!(inside_internal_surfaces(
        &[internal_square()],
        arc::Point { x: 1.0, y: 1.0 },
        arc::Point { x: 9.0, y: 9.0 },
        CoordinateScale::Normal,
        (0.0, 0.0),
    ));
}

#[test]
fn travel_leaving_internal_surface_requires_retraction() {
    assert!(!inside_internal_surfaces(
        &[internal_square()],
        arc::Point { x: 1.0, y: 1.0 },
        arc::Point { x: 11.0, y: 9.0 },
        CoordinateScale::Normal,
        (0.0, 0.0),
    ));
}
