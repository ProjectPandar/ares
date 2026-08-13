use std::cell::{Cell, RefCell};

use super::*;

#[test]
fn task22o66_flattens_every_kind_and_hole_then_runs_exact_operations() {
    let fill_surfaces = [
        surface(
            RegionSurfaceKind::Top,
            expolygon(rectangle(0, 0, 100, 100), vec![rectangle(20, 20, 40, 40)]),
        ),
        surface(
            RegionSurfaceKind::InternalVoid,
            expolygon(rectangle(200, 0, 300, 100), Vec::new()),
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            expolygon(rectangle(-200, 0, -100, 100), Vec::new()),
        ),
    ];
    let before = surface_snapshot(&fill_surfaces);
    let ensuring = [rectangle(-10, -10, 310, 110)];
    let ensuring_before = snapshot(&ensuring);
    let expected_flat = vec![
        rectangle(0, 0, 100, 100),
        rectangle(20, 20, 40, 40),
        rectangle(200, 0, 300, 100),
        rectangle(-200, 0, -100, 100),
    ];
    let unioned = vec![
        expolygon(rectangle(1, 1, 99, 99), vec![rectangle(21, 21, 39, 39)]),
        expolygon(rectangle(201, 1, 299, 99), Vec::new()),
    ];
    let expected_union_flat = vec![
        rectangle(1, 1, 99, 99),
        rectangle(21, 21, 39, 39),
        rectangle(201, 1, 299, 99),
    ];
    let shrunk = vec![rectangle(10, 10, 90, 90)];
    let ring = vec![rectangle(1, 1, 10, 99), rectangle(201, 1, 210, 99)];
    let clipped = vec![expolygon(rectangle(1, 1, 10, 99), Vec::new())];

    for (scale, spacing, delta) in [
        (CoordinateScale::Normal, 167.772_17, -167_772_176.0_f32),
        (CoordinateScale::LargeBed, 167.772_17, -16_777_216.0_f32),
        (CoordinateScale::Normal, 0.45, -449_999.0_f32),
    ] {
        let step = Cell::new(0);
        let output = prepare_region_bridge_ensuring_areas_using(
            &fill_surfaces,
            &ensuring,
            flow(spacing),
            scale,
            RegionBridgeEnsuringOperations {
                union: |input: &[Polygon]| {
                    assert_eq!(step.replace(1), 0);
                    assert_eq!(snapshot(input), snapshot(&expected_flat));
                    Ok(unioned.clone())
                },
                shrink: |input: &[Polygon], actual_delta: f32| {
                    assert_eq!(step.replace(2), 1);
                    assert_eq!(snapshot(input), snapshot(&expected_union_flat));
                    assert_eq!(actual_delta.to_bits(), delta.to_bits());
                    Ok(shrunk.clone())
                },
                difference: |subject: &[Polygon], clip: &[Polygon]| {
                    assert_eq!(step.replace(3), 2);
                    assert_eq!(snapshot(subject), snapshot(&expected_union_flat));
                    assert_eq!(snapshot(clip), snapshot(&shrunk));
                    Ok(ring.clone())
                },
                intersection: |subject: &[Polygon], clip: &[Polygon]| {
                    assert_eq!(step.replace(4), 3);
                    assert_eq!(subject.as_ptr(), ensuring.as_ptr());
                    assert_eq!(snapshot(clip), snapshot(&ring));
                    Ok(clipped.clone())
                },
            },
        )
        .unwrap();

        assert_eq!(step.get(), 4);
        assert_eq!(snapshot(&output.near_perimeters), snapshot(&ring));
        assert_eq!(output.additional_ensuring, clipped);
        assert_eq!(surface_snapshot(&fill_surfaces), before);
        assert_eq!(snapshot(&ensuring), ensuring_before);
    }
}

#[test]
fn task22o66_empty_inputs_still_run_all_operations() {
    let visits = RefCell::new(Vec::new());
    let output = prepare_region_bridge_ensuring_areas_using(
        &[],
        &[],
        flow(0.000_01),
        CoordinateScale::Normal,
        RegionBridgeEnsuringOperations {
            union: |input: &[Polygon]| {
                visits.borrow_mut().push("union");
                assert!(input.is_empty());
                Ok(Vec::new())
            },
            shrink: |input: &[Polygon], delta: f32| {
                visits.borrow_mut().push("shrink");
                assert!(input.is_empty());
                assert_eq!(delta.to_bits(), (-9.0_f32).to_bits());
                Ok(Vec::new())
            },
            difference: |subject: &[Polygon], clip: &[Polygon]| {
                visits.borrow_mut().push("difference");
                assert!(subject.is_empty());
                assert!(clip.is_empty());
                Ok(Vec::new())
            },
            intersection: |subject: &[Polygon], clip: &[Polygon]| {
                visits.borrow_mut().push("intersection");
                assert!(subject.is_empty());
                assert!(clip.is_empty());
                Ok(Vec::new())
            },
        },
    )
    .unwrap();

    assert_eq!(
        visits.into_inner(),
        vec!["union", "shrink", "difference", "intersection"]
    );
    assert_eq!(
        output,
        RegionBridgeEnsuringAreas {
            near_perimeters: Vec::new(),
            additional_ensuring: Vec::new(),
        }
    );
}

#[test]
fn task22o66_flatten_expolygons_preserves_component_contour_hole_order() {
    let flattened = flatten_expolygons(vec![
        expolygon(
            rectangle(0, 0, 100, 100),
            vec![rectangle(10, 10, 20, 20), rectangle(30, 30, 40, 40)],
        ),
        expolygon(rectangle(200, 0, 300, 100), Vec::new()),
    ]);
    assert_eq!(
        snapshot(&flattened),
        snapshot(&[
            rectangle(0, 0, 100, 100),
            rectangle(10, 10, 20, 20),
            rectangle(30, 30, 40, 40),
            rectangle(200, 0, 300, 100),
        ])
    );
}

#[test]
fn task22o66_output_keeps_difference_and_intersection_engine_order() {
    let ring = vec![rectangle(200, 0, 210, 10), rectangle(-20, 0, -10, 10)];
    let clipped = vec![
        expolygon(rectangle(200, 0, 210, 10), Vec::new()),
        expolygon(rectangle(-20, 0, -10, 10), Vec::new()),
    ];
    let output = prepare_region_bridge_ensuring_areas_using(
        &[],
        &[],
        flow(0.000_01),
        CoordinateScale::Normal,
        RegionBridgeEnsuringOperations {
            union: |_: &[Polygon]| Ok(Vec::new()),
            shrink: |_: &[Polygon], _: f32| Ok(Vec::new()),
            difference: |_: &[Polygon], _: &[Polygon]| Ok(ring.clone()),
            intersection: |_: &[Polygon], _: &[Polygon]| Ok(clipped.clone()),
        },
    )
    .unwrap();

    assert_eq!(snapshot(&output.near_perimeters), snapshot(&ring));
    assert_eq!(output.additional_ensuring, clipped);
}

#[test]
fn task22o66_forwards_exact_miter_three_to_the_shrink_kernel() {
    assert_eq!(shrink_configuration_for_test(), (JoinType::Miter, 3.0));
    let surfaces = [surface(
        RegionSurfaceKind::Internal,
        expolygon(rectangle(0, 0, 100, 100), Vec::new()),
    )];
    let expected_union = vec![expolygon(rectangle(0, 0, 100, 100), Vec::new())];
    let called = Cell::new(false);

    let output = prepare_region_bridge_ensuring_areas_using(
        &surfaces,
        &[],
        flow(0.000_03),
        CoordinateScale::Normal,
        RegionBridgeEnsuringOperations {
            union: |_: &[Polygon]| Ok(expected_union.clone()),
            shrink: |subject: &[Polygon], delta: f32| {
                called.set(true);
                assert_eq!(snapshot(subject), snapshot(&[rectangle(0, 0, 100, 100)]));
                assert_eq!(delta.to_bits(), (-29.0_f32).to_bits());
                Ok(Vec::new())
            },
            difference: |_: &[Polygon], _: &[Polygon]| Ok(Vec::new()),
            intersection: |_: &[Polygon], _: &[Polygon]| Ok(Vec::new()),
        },
    )
    .unwrap();

    assert!(called.get());
    assert!(output.near_perimeters.is_empty());
}

#[test]
fn task22o66_first_error_stops_the_operation_transaction() {
    for fail_at in 0..4 {
        let step = Cell::new(0);
        let result = prepare_region_bridge_ensuring_areas_using(
            &[],
            &[],
            flow(0.000_01),
            CoordinateScale::Normal,
            RegionBridgeEnsuringOperations {
                union: |_: &[Polygon]| {
                    let current = step.get();
                    step.set(current + 1);
                    if current == fail_at {
                        Err(ClipperError::CoordinateOutOfRange)
                    } else {
                        Ok(Vec::new())
                    }
                },
                shrink: |_: &[Polygon], _: f32| {
                    let current = step.get();
                    step.set(current + 1);
                    if current == fail_at {
                        Err(ClipperError::CoordinateOutOfRange)
                    } else {
                        Ok(Vec::new())
                    }
                },
                difference: |_: &[Polygon], _: &[Polygon]| {
                    let current = step.get();
                    step.set(current + 1);
                    if current == fail_at {
                        Err(ClipperError::CoordinateOutOfRange)
                    } else {
                        Ok(Vec::new())
                    }
                },
                intersection: |_: &[Polygon], _: &[Polygon]| {
                    let current = step.get();
                    step.set(current + 1);
                    if current == fail_at {
                        Err(ClipperError::CoordinateOutOfRange)
                    } else {
                        Ok(Vec::new())
                    }
                },
            },
        );

        assert_eq!(result.unwrap_err(), ClipperError::CoordinateOutOfRange);
        assert_eq!(step.get(), fail_at + 1);
    }
}
