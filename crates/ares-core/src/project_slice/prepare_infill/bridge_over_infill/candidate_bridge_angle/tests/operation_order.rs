use std::cell::Cell;

use crate::{OrcaFloat, ProcessInfillPattern, geometry::CoordinateScale};

use super::{input, line, polygon, polyline, region};
use crate::project_slice::prepare_infill::bridge_over_infill::candidate_bridge_angle::determine_candidate_bridge_angle_using;

#[test]
fn task22o60_anchor_dispatch_forwards_exact_inputs_lines_and_sparse_pattern() {
    let area = vec![polygon(&[(70, 10), (90, 10)])];
    let anchors = vec![
        polyline(&[(8, 9), (3, 4), (8, 9), (3, 4)]),
        polyline(&[(8, 9), (3, 4)]),
        polyline(&[(3, 4), (3, 4)]),
        polyline(&[(99, 100)]),
        polyline(&[(50, 60), (20, 30)]),
    ];
    let boundaries = vec![polyline(&[(1, 2), (7, 8)])];
    let mut region = region();
    region.sparse_infill_pattern = ProcessInfillPattern::HilbertCurve;
    let rotation = f64::from_bits(0xbfe2_3456_789a_bcde);
    let step = Cell::new(0);
    let detected = f64::from_bits(0x3fe2_3456_789a_bcde);
    let result = f64::from_bits(0x4002_3456_789a_bcde);

    let output = determine_candidate_bridge_angle_using(
        input(
            &area,
            &anchors,
            &boundaries,
            &region,
            rotation,
            CoordinateScale::LargeBed,
        ),
        |received_area, lines, pattern, scale| {
            assert_eq!(step.replace(1), 0);
            assert!(std::ptr::eq(received_area, area.as_slice()));
            assert_eq!(scale, CoordinateScale::LargeBed);
            assert_eq!(pattern, ProcessInfillPattern::HilbertCurve);
            assert_eq!(
                lines,
                &[
                    line(8, 9, 3, 4),
                    line(3, 4, 8, 9),
                    line(8, 9, 3, 4),
                    line(8, 9, 3, 4),
                    line(3, 4, 3, 4),
                    line(50, 60, 20, 30)
                ]
            );
            detected
        },
        |received, received_region, received_rotation| {
            assert_eq!(step.replace(2), 1);
            assert_eq!(received.to_bits(), detected.to_bits());
            assert!(std::ptr::eq(received_region, &region));
            assert_eq!(received_rotation.to_bits(), rotation.to_bits());
            result
        },
    );

    assert_eq!(step.get(), 2);
    assert_eq!(output.to_bits(), result.to_bits());
}

#[test]
fn task22o60_empty_anchors_use_ordered_boundaries_and_neutral_line_pattern() {
    let area = vec![polygon(&[(0, 0), (1, 0)])];
    let anchors = Vec::new();
    let boundaries = vec![
        polyline(&[(70, 0), (10, 0), (50, 0)]),
        polyline(&[(4, 5)]),
        polyline(&[(0, 0), (90, 0)]),
    ];
    let mut region = region();
    region.sparse_infill_pattern = ProcessInfillPattern::OctagramSpiral;
    let detector_calls = Cell::new(0);
    let override_calls = Cell::new(0);

    let output = determine_candidate_bridge_angle_using(
        input(
            &area,
            &anchors,
            &boundaries,
            &region,
            -0.0,
            CoordinateScale::Normal,
        ),
        |_, lines, pattern, _| {
            detector_calls.set(detector_calls.get() + 1);
            assert_eq!(pattern, ProcessInfillPattern::Line);
            assert_eq!(
                lines,
                &[line(70, 0, 10, 0), line(10, 0, 50, 0), line(0, 0, 90, 0)]
            );
            7.25
        },
        |detected, _, rotation| {
            override_calls.set(override_calls.get() + 1);
            assert_eq!(detected.to_bits(), 7.25_f64.to_bits());
            assert_eq!(rotation.to_bits(), (-0.0_f64).to_bits());
            detected
        },
    );

    assert_eq!(output.to_bits(), 7.25_f64.to_bits());
    assert_eq!(detector_calls.get(), 1);
    assert_eq!(override_calls.get(), 1);
}

#[test]
fn task22o60_nonempty_one_point_anchors_own_dispatch_before_flattening() {
    let area = vec![polygon(&[(0, 0), (1, 0)])];
    let anchors = vec![polyline(&[(1, 2)]), polyline(&[(3, 4)])];
    let boundaries = vec![polyline(&[(10, 20), (30, 40)])];
    let mut region = region();
    region.sparse_infill_pattern = ProcessInfillPattern::Grid;

    let output = determine_candidate_bridge_angle_using(
        input(
            &area,
            &anchors,
            &boundaries,
            &region,
            0.0,
            CoordinateScale::Normal,
        ),
        |_, lines, pattern, _| {
            assert!(lines.is_empty());
            assert_eq!(pattern, ProcessInfillPattern::Grid);
            0.125
        },
        |detected, _, _| detected,
    );

    assert_eq!(output.to_bits(), 0.125_f64.to_bits());
}

#[test]
fn task22o60_detector_output_is_not_changed_before_override() {
    let area = vec![polygon(&[(0, 0), (1, 0)])];
    let anchors = vec![polyline(&[(0, 0), (1, 0)])];
    let boundaries = Vec::new();
    let mut region = region();
    region.internal_bridge_angle = OrcaFloat(0.0);
    let payload = f64::from_bits(0x7ff8_0000_0000_0042);

    let output = determine_candidate_bridge_angle_using(
        input(
            &area,
            &anchors,
            &boundaries,
            &region,
            f64::from_bits(0x7ff8_0000_0000_0084),
            CoordinateScale::Normal,
        ),
        |_, _, _, _| payload,
        |detected, _, _| {
            assert_eq!(detected.to_bits(), payload.to_bits());
            detected
        },
    );

    assert_eq!(output.to_bits(), payload.to_bits());
}
