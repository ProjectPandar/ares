use super::super::MotionOptions;
use super::{EmitState, inside_internal_surfaces, wipe_moves};
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

#[test]
fn wipe_distribution_uses_configured_distance_when_clipping_rounds_long() {
    let state = EmitState {
        scale_factor: 0.000_001,
        options: MotionOptions {
            wipe: true,
            wipe_distance: 1.0,
            ..MotionOptions::default()
        },
        wipe_start: Some(arc::Point { x: 0.0, y: 0.0 }),
        wipe_path: vec![
            arc::Point { x: 0.0, y: 0.0 },
            arc::Point { x: 0.6, y: 0.2 },
            arc::Point { x: -1.0, y: 0.4 },
        ],
        ..EmitState::default()
    };

    let path = wipe_moves(&state);
    let clipped_distance = path.segments.iter().map(|(_, length)| length).sum::<f64>();

    assert!(clipped_distance > 1_000_000.0);
    assert_eq!(path.retraction_distance, 1.0);
    assert_eq!(path.distribution_distance, 1_000_000.0);
}
