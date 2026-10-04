use super::{AutoLift, Layer, Object};
use crate::geometry::{BoundingBox, CoordinateScale, Point, Polygon};
use crate::project_slice::gcode_emit::motion::{EmitState, MotionOptions};

#[test]
fn auto_lift_queries_every_instance_without_duplicating_layer_geometry() {
    // Print.cpp:594–614 translates the travel into every PrintInstance frame.
    let min = Point::new(-500_000, -500_000);
    let max = Point::new(500_000, 500_000);
    let lift = AutoLift {
        maximum_hop: 10.0,
        objects: vec![Object {
            shifts: vec![Point::new(0, 0), Point::new(30_000_000, 0)],
            layers: vec![Layer {
                print_z: 0.2,
                bounds: Some(BoundingBox::new(min, max)),
                polygons: vec![Polygon::new(vec![
                    min,
                    Point::new(max.x(), min.y()),
                    max,
                    Point::new(min.x(), max.y()),
                ])],
            }],
        }],
    };
    let state = EmitState {
        active_layer_print_z: 0.2,
        options: MotionOptions {
            travel_slope_radians: std::f64::consts::FRAC_PI_4,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    for (x, expected) in [(0, true), (20_000_000, false), (30_000_000, true)] {
        assert_eq!(
            lift.crosses_overhang(
                &state,
                [Point::new(x - 1_000_000, 0), Point::new(x + 1_000_000, 0)],
                CoordinateScale::Normal
            ),
            expected
        );
    }
}
