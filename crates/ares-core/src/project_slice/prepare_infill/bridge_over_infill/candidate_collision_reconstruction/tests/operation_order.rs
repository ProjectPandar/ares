use std::cell::{Cell, RefCell};

use super::*;

#[test]
fn task22o62_empty_history_expands_exact_initial_area_and_preserves_allocations() {
    for (scale, expected_delta) in [
        (CoordinateScale::Normal, 503_316_512.0_f32),
        (CoordinateScale::LargeBed, 50_331_652.0_f32),
    ] {
        let original = vec![rectangle(0, 0, 2_000_000, 1_600_000)];
        let boundaries = vec![polyline(&[(-5, 1), (5, 1)])];
        let bridge = vec![rectangle(10, 20, 30, 40)];
        let boundary_ptr = boundaries.as_ptr();
        let bridge_ptr = bridge.as_ptr();
        let bridge_points_ptr = bridge[0].points().as_ptr();

        let mut arithmetic_flow = flow();
        arithmetic_flow.spacing = 167.772_17_f32;
        let output = reconstruct_candidate_bridge_collision_using(
            &original,
            initial(boundaries, bridge),
            arithmetic_flow,
            0.37,
            &[],
            scale,
            |input, delta| {
                assert_eq!(input.as_ptr(), bridge_ptr);
                assert_eq!(input[0].points().as_ptr(), bridge_points_ptr);
                assert_eq!(delta.to_bits(), expected_delta.to_bits());
                Ok(vec![rectangle(-1, -1, 1, 1)])
            },
            |_, _| -> Result<Vec<Polygon>, ClipperError> {
                panic!("empty completed history must not intersect")
            },
            |_, _, _, _, _| -> Result<Vec<Polygon>, ClipperError> {
                panic!("no collision must not reconstruct")
            },
        )
        .unwrap();

        assert_eq!(output.boundary_polylines.as_ptr(), boundary_ptr);
        assert_eq!(output.bridging_area.as_ptr(), bridge_ptr);
        assert_eq!(output.bridging_area[0].points().as_ptr(), bridge_points_ptr);
        assert_eq!(output.bridging_angle, 0.37);
    }
}

#[test]
fn task22o62_first_collision_owns_angle_breaks_and_forwards_exact_source_operands() {
    let original = vec![rectangle(0, 0, 2_000_000, 1_600_000)];
    let boundaries = vec![polyline(&[(0, 0), (3, 4), (8, 9)])];
    let boundary_ptr = boundaries.as_ptr();
    let completed = vec![
        surface(9, vec![rectangle(50, 0, 60, 10)], 0.25),
        surface(2, vec![rectangle(10, 0, 20, 10)], 1.75),
        surface(5, vec![rectangle(30, 0, 40, 10)], 2.75),
    ];
    let expanded = vec![rectangle(-100, -100, 100, 100)];
    let intersection_geometry = rectangle(777, 777, 888, 888);
    let replacement = vec![rectangle(90, -8, 107, 6), rectangle(-90, -8, -73, 6)];
    let calls = Cell::new(0);
    let exact_flow = flow();

    let output = reconstruct_candidate_bridge_collision_using(
        &original,
        initial(boundaries, vec![rectangle(1, 1, 2, 2)]),
        exact_flow,
        0.5,
        &completed,
        CoordinateScale::LargeBed,
        |input, delta| {
            assert_eq!(
                snapshot_polygons(input),
                vec![vec![(1, 1), (2, 1), (2, 2), (1, 2)]]
            );
            assert_eq!(delta.to_bits(), 134_997.0_f32.to_bits());
            Ok(expanded.clone())
        },
        |subject, clip| {
            let call = calls.get();
            calls.set(call + 1);
            assert_eq!(subject.as_ptr(), completed[call].new_polygons.as_ptr());
            assert_eq!(snapshot_polygons(clip), snapshot_polygons(&expanded));
            match call {
                0 => Ok(Vec::new()),
                1 => Ok(vec![intersection_geometry.clone()]),
                _ => panic!("the first collision must break before later surfaces"),
            }
        },
        |area, lines, forwarded_flow, angle, scale| {
            assert_eq!(area.as_ptr(), original.as_ptr());
            assert_eq!(lines.len(), 2);
            assert_eq!((lines[0].a.x(), lines[0].a.y()), (0, 0));
            assert_eq!((lines[0].b.x(), lines[0].b.y()), (3, 4));
            assert_eq!((lines[1].a.x(), lines[1].a.y()), (3, 4));
            assert_eq!((lines[1].b.x(), lines[1].b.y()), (8, 9));
            assert_eq!(forwarded_flow.width, exact_flow.width);
            assert_eq!(forwarded_flow.height, exact_flow.height);
            assert_eq!(forwarded_flow.spacing, exact_flow.spacing);
            assert_eq!(forwarded_flow.nozzle_diameter, exact_flow.nozzle_diameter);
            assert_eq!(forwarded_flow.bridge, exact_flow.bridge);
            assert_eq!(forwarded_flow.mm3_per_mm, exact_flow.mm3_per_mm);
            assert_eq!(angle, 1.75);
            assert_eq!(scale, CoordinateScale::LargeBed);
            Ok(replacement.clone())
        },
    )
    .unwrap();

    assert_eq!(calls.get(), 2);
    assert_eq!(output.boundary_polylines.as_ptr(), boundary_ptr);
    assert_eq!(output.bridging_area, replacement);
    assert_ne!(output.bridging_area, vec![intersection_geometry]);
    assert_eq!(output.bridging_angle, 1.75);
}

#[test]
fn task22o62_injected_competing_errors_preserve_expand_intersection_construct_order() {
    let original = vec![rectangle(0, 0, 100, 100)];
    let completed = vec![surface(0, vec![rectangle(0, 0, 10, 10)], 1.0)];
    let run = |expand_error, intersection_error, construct_error| {
        let trace = RefCell::new(Vec::new());
        let result = reconstruct_candidate_bridge_collision_using(
            &original,
            initial(
                vec![polyline(&[(0, 0), (100, 0)])],
                vec![rectangle(1, 1, 9, 9)],
            ),
            flow(),
            0.5,
            &completed,
            CoordinateScale::Normal,
            |_, _| {
                trace.borrow_mut().push("expand");
                if expand_error {
                    Err(ClipperError::CoordinateOutOfRange)
                } else {
                    Ok(vec![rectangle(0, 0, 20, 20)])
                }
            },
            |_, _| {
                trace.borrow_mut().push("intersection");
                if intersection_error {
                    Err(ClipperError::OpenPathMustBeSubject)
                } else {
                    Ok(vec![rectangle(2, 2, 3, 3)])
                }
            },
            |_, _, _, _, _| {
                trace.borrow_mut().push("construct");
                if construct_error {
                    Err(ClipperError::OpenPathsRequirePolyTree)
                } else {
                    Ok(vec![rectangle(3, 3, 4, 4)])
                }
            },
        );
        (result.unwrap_err(), trace.into_inner())
    };

    assert_eq!(
        run(true, true, true),
        (ClipperError::CoordinateOutOfRange, vec!["expand"])
    );
    assert_eq!(
        run(false, true, true),
        (
            ClipperError::OpenPathMustBeSubject,
            vec!["expand", "intersection"]
        )
    );
    assert_eq!(
        run(false, false, true),
        (
            ClipperError::OpenPathsRequirePolyTree,
            vec!["expand", "intersection", "construct"]
        )
    );
}
