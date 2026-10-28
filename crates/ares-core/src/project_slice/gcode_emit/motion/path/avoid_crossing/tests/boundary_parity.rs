//! Native-captured boundaries/routes and source-cited travel emission, Orca 2.4.2.
use super::*;

fn square() -> Vec<ExPolygon> {
    vec![ExPolygon::new(
        Polygon::new(vec![
            Point::new(5_000_000, 5_000_000),
            Point::new(-5_000_000, 5_000_000),
            Point::new(-5_000_000, -5_000_000),
            Point::new(5_000_000, -5_000_000),
        ]),
        Vec::new(),
    )]
}

fn crossing_geometry(slices: &[ExPolygon]) -> AvoidCrossingGeometry<'_> {
    AvoidCrossingGeometry {
        external_perimeter_width: 0.22,
        layer_slices: slices,
        perimeter_spacing: 0.189_955_74,
        top_surfaces: &[],
        chunk_slices: &[],
        chunk_perimeter_spacing: 0.0,
    }
}

#[test]
fn get_boundary_keeps_scaled_spacing_fraction_and_plain_union_contour() {
    let slices = square();
    let super::super::boundary::BuildResult::Ready(boundary) =
        super::super::boundary::Boundary::build(
            &crossing_geometry(&slices),
            CoordinateScale::Normal,
            [
                Point::new(-3_559_422, 4_728_540),
                Point::new(4_890_001, 4_890_001),
            ],
        )
        .unwrap()
    else {
        panic!("a square builds a boundary");
    };
    assert_eq!(boundary.scaled_spacing, 189_955.0);
    assert_eq!(
        boundary.contours,
        [vec![
            Point::new(4_715_068, 4_715_068),
            Point::new(-4_715_068, 4_715_068),
            Point::new(-4_715_068, -4_715_068),
            Point::new(4_715_068, -4_715_068),
        ]]
    );
}

#[test]
fn avoid_perimeters_inner_captured_outer_wall_approach() {
    assert_route(
        (-3_559_422, 4_728_540),
        (4_890_001, 4_890_001),
        &[(-3_559_522, 4_714_968), (4_714_998, 4_714_998)],
    );
}

#[test]
fn avoid_perimeters_inner_captured_retry_retains_backward_route() {
    assert_route(
        (4_890_001, 4_860_001),
        (4_814_021, -2_328_698),
        &[(4_714_998, 4_714_998), (4_714_968, -2_328_798)],
    );
}

#[test]
fn travel_to_same_point_outside_safe_zone_retains_native_corner() {
    // AvoidCrossingPerimeters.cpp:595,211–216 extends even an empty hit list.
    // The native same-XY capture retains this protected common-boundary point.
    assert_route(
        (4_890_000, 4_890_000),
        (4_890_000, 4_890_000),
        &[(4_715_068, 4_715_068)],
    );
}

#[test]
fn travel_to_same_point_outside_closest_line_radius_keeps_singleton_interior_empty() {
    // The native route at (6,6) is a singleton: no closest lines are in range.
    // Stripping its endpoints must not underflow (to_polyline, cpp:348–355).
    assert_route((6_000_000, 6_000_000), (6_000_000, 6_000_000), &[]);
}

#[test]
fn start_travel_slope_z_at_same_xy_outside_safe_zone_emits_corner_then_destination() {
    use crate::project_slice::gcode_emit::motion::{
        EmitState, MotionOptions, features::PathProperties, path::start_travel, scarf::Slope,
    };

    let slices = square();
    let geometry = LayerGeometry {
        avoid_crossing: crossing_geometry(&slices),
        ..geometry()
    };
    let mut state = EmitState {
        x: 4.89,
        y: 4.89,
        last_scaled_position: (4_890_000, 4_890_000),
        last_position_defined: true,
        positioned: true,
        layer_z: 1.0,
        travel_feedrate: 6000.0,
        options: MotionOptions {
            reduce_crossing_wall: true,
            retraction_minimum_travel: 1.0,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();
    start_travel::emit(
        &mut output,
        &mut state,
        start_travel::Request {
            force_no_extrusion: false,
            first_scaled: (4_890_000, 4_890_000),
            first_x: 4.89,
            first_y: 4.89,
            properties: PathProperties {
                contour: None,
                mm3_per_mm: 0.04,
                width: 0.22,
                height: 0.25,
                feature: "Outer wall",
                is_perimeter: true,
                end_clip: 0.0,
                fitting: &[],
                slope: Some(Slope {
                    z_begin: 0.0,
                    z_end: 1.0,
                    e_begin: 0.0,
                    e_end: 1.0,
                    speed: 50.0,
                    flow_ratio: 1.0,
                }),
                inside_loop: false,
            },
            geometry,
        },
    )
    .unwrap();
    // GCode.cpp:7485–7502 emits the detour at nominal Z, then the scarf Z.
    assert_eq!(
        output,
        b"G1 X4.715 Y4.715 F6000\nG1 X4.89 Y4.89 Z.75 F6000\n"
    );
    assert_eq!(state.scarf_z, Some(0.75));
    assert_eq!(state.last_scaled_position, (4_890_000, 4_890_000));
    assert!(state.last_position_defined);
}

fn assert_route(start: (i64, i64), end: (i64, i64), expected: &[(i64, i64)]) {
    let slices = square();
    let geometry = LayerGeometry {
        avoid_crossing: crossing_geometry(&slices),
        ..geometry()
    };
    let point = |(x, y)| MotionPoint {
        x: geometry.scale.unscale(x),
        y: geometry.scale.unscale(y),
    };
    let mut boundary = super::super::build_boundary(&geometry).unwrap();
    let route = super::super::route(
        Request {
            start: point(start),
            end: point(end),
            geometry,
            offset: (0.0, 0.0),
            origin: (0.0, 0.0),
            use_external: false,
            max_detour: FloatOrPercent::Float(0.0),
        },
        &mut boundary,
    )
    .unwrap();
    assert_eq!(
        route,
        expected.iter().copied().map(point).collect::<Vec<_>>()
    );
}
