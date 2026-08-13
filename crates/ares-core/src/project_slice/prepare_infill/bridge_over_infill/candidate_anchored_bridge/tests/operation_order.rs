use std::cell::Cell;

use crate::geometry::{ClipperError, CoordinateScale};

use super::{
    construct_candidate_anchored_bridge_using, flow, input, line, polygon, polyline,
    snapshot_polygons, snapshot_polylines,
};

#[test]
fn task22o61_empty_lightning_appends_anchors_then_calls_construct_once() {
    let area = vec![polygon(&[(0, 0), (10, 0), (10, 10)])];
    let boundaries = vec![polyline(&[(70, 0), (10, 0)])];
    let anchors = vec![polyline(&[(8, 9), (3, 4), (8, 9)]), polyline(&[(99, 100)])];
    let area_before = snapshot_polygons(&area);
    let anchors_before = snapshot_polylines(&anchors);
    let bridge_flow = flow();
    let angle = f64::from_bits(0x3fe2_3456_789a_bcde);
    let construct_calls = Cell::new(0);
    let expected_area = vec![polygon(&[(1, 2), (3, 4), (5, 6)])];

    let result = construct_candidate_anchored_bridge_using(
        input(
            &area,
            boundaries.clone(),
            &anchors,
            &[],
            bridge_flow,
            angle,
            CoordinateScale::LargeBed,
        ),
        |_, _| panic!("empty Lightning must skip closed intersection"),
        |_, _| panic!("empty Lightning must skip expansion"),
        |_, _| panic!("empty Lightning must skip open intersection"),
        |received_area, lines, received_flow, received_angle, scale| {
            construct_calls.set(construct_calls.get() + 1);
            assert!(std::ptr::eq(received_area, area.as_slice()));
            assert_eq!(
                lines,
                &[line(70, 0, 10, 0), line(8, 9, 3, 4), line(3, 4, 8, 9)]
            );
            assert_eq!(
                [
                    received_flow.width.to_bits(),
                    received_flow.height.to_bits(),
                    received_flow.spacing.to_bits(),
                    received_flow.nozzle_diameter.to_bits(),
                    u32::from(received_flow.bridge),
                ],
                [
                    bridge_flow.width.to_bits(),
                    bridge_flow.height.to_bits(),
                    bridge_flow.spacing.to_bits(),
                    bridge_flow.nozzle_diameter.to_bits(),
                    u32::from(bridge_flow.bridge),
                ]
            );
            assert_eq!(
                received_flow.mm3_per_mm.to_bits(),
                bridge_flow.mm3_per_mm.to_bits()
            );
            assert_eq!(received_angle.to_bits(), angle.to_bits());
            assert_eq!(scale, CoordinateScale::LargeBed);
            Ok(expected_area.clone())
        },
    )
    .unwrap();

    let mut expected_boundaries = boundaries;
    expected_boundaries.extend(anchors.clone());
    assert_eq!(result.boundary_polylines, expected_boundaries);
    assert_eq!(result.bridging_area, expected_area);
    assert_eq!(construct_calls.get(), 1);
    assert_eq!(snapshot_polygons(&area), area_before);
    assert_eq!(snapshot_polylines(&anchors), anchors_before);
}

#[test]
fn task22o61_lightning_overlap_expands_original_area_and_replaces_boundaries() {
    for (scale, expected_delta) in [
        (CoordinateScale::Normal, 10_000_000_f32),
        (CoordinateScale::LargeBed, 1_000_000_f32),
    ] {
        let area = vec![polygon(&[(0, 0), (20, 0), (20, 20)])];
        let boundaries = vec![polyline(&[(70, 0), (10, 0)])];
        let anchors = vec![polyline(&[(8, 9), (3, 4)])];
        let lightning = vec![polygon(&[(5, 5), (15, 5), (15, 15)])];
        let overlap = vec![polygon(&[(101, 102), (103, 104), (105, 106)])];
        let expanded = vec![polygon(&[(201, 202), (203, 204), (205, 206)])];
        let replacement = vec![
            polyline(&[(50, 60), (20, 30), (50, 60)]),
            polyline(&[(9, 10)]),
        ];
        let bridged = vec![
            polygon(&[(301, 302), (303, 304), (305, 306)]),
            polygon(&[(11, 12), (13, 14), (15, 16)]),
        ];
        let step = Cell::new(0);

        let output = construct_candidate_anchored_bridge_using(
            input(
                &area,
                boundaries.clone(),
                &anchors,
                &lightning,
                flow(),
                0.37,
                scale,
            ),
            |subject, clip| {
                assert_eq!(step.replace(1), 0);
                assert!(std::ptr::eq(subject, area.as_slice()));
                assert!(std::ptr::eq(clip, lightning.as_slice()));
                Ok(overlap.clone())
            },
            |subject, delta| {
                assert_eq!(step.replace(2), 1);
                assert!(std::ptr::eq(subject, area.as_slice()));
                assert_eq!(delta.to_bits(), expected_delta.to_bits());
                Ok(expanded.clone())
            },
            |subject, clip| {
                assert_eq!(step.replace(3), 2);
                let mut appended = boundaries.clone();
                appended.extend(anchors.clone());
                assert_eq!(subject, appended);
                assert_eq!(clip, expanded);
                Ok(replacement.clone())
            },
            |received_area, lines, _, angle, received_scale| {
                assert_eq!(step.replace(4), 3);
                assert!(std::ptr::eq(received_area, area.as_slice()));
                assert_eq!(lines, &[line(50, 60, 20, 30), line(20, 30, 50, 60)]);
                assert_eq!(angle.to_bits(), 0.37_f64.to_bits());
                assert_eq!(received_scale, scale);
                Ok(bridged.clone())
            },
        )
        .unwrap();

        assert_eq!(step.get(), 4);
        assert_eq!(output.boundary_polylines, replacement);
        assert_eq!(output.bridging_area, bridged);
    }
}

#[test]
fn task22o61_empty_overlap_skips_expand_and_open_clip_but_keeps_appended_lines() {
    let area = vec![polygon(&[(0, 0), (10, 0), (10, 10)])];
    let boundaries = vec![polyline(&[(1, 2), (3, 4)])];
    let anchors = vec![polyline(&[(5, 6), (7, 8)])];
    let lightning = vec![polygon(&[(20, 20), (30, 20), (30, 30)])];

    let output = construct_candidate_anchored_bridge_using(
        input(
            &area,
            boundaries.clone(),
            &anchors,
            &lightning,
            flow(),
            0.0,
            CoordinateScale::Normal,
        ),
        |_, _| Ok(Vec::new()),
        |_, _| panic!("empty overlap must skip expansion"),
        |_, _| panic!("empty overlap must skip open intersection"),
        |_, lines, _, _, _| {
            assert_eq!(lines, &[line(1, 2, 3, 4), line(5, 6, 7, 8)]);
            Ok(vec![polygon(&[(9, 9), (10, 9), (10, 10)])])
        },
    )
    .unwrap();

    let mut expected = boundaries;
    expected.extend(anchors);
    assert_eq!(output.boundary_polylines, expected);
}

#[test]
fn task22o61_each_injected_error_stops_at_its_source_ordinal() {
    let area = vec![polygon(&[(0, 0), (10, 0), (10, 10)])];
    let boundaries = vec![polyline(&[(1, 2), (3, 4)])];
    let lightning = vec![polygon(&[(0, 0), (5, 0), (5, 5)])];
    let error = ClipperError::CoordinateOutOfRange;

    let closed = construct_candidate_anchored_bridge_using(
        input(
            &area,
            boundaries.clone(),
            &[],
            &lightning,
            flow(),
            0.0,
            CoordinateScale::Normal,
        ),
        |_, _| Err(error),
        |_, _| panic!("must stop after closed error"),
        |_, _| panic!("must stop after closed error"),
        |_, _, _, _, _| panic!("must stop after closed error"),
    );
    assert!(matches!(closed, Err(ClipperError::CoordinateOutOfRange)));

    let expanded = construct_candidate_anchored_bridge_using(
        input(
            &area,
            boundaries.clone(),
            &[],
            &lightning,
            flow(),
            0.0,
            CoordinateScale::Normal,
        ),
        |_, _| Ok(area.clone()),
        |_, _| Err(error),
        |_, _| panic!("must stop after expansion error"),
        |_, _, _, _, _| panic!("must stop after expansion error"),
    );
    assert!(matches!(expanded, Err(ClipperError::CoordinateOutOfRange)));

    let opened = construct_candidate_anchored_bridge_using(
        input(
            &area,
            boundaries.clone(),
            &[],
            &lightning,
            flow(),
            0.0,
            CoordinateScale::Normal,
        ),
        |_, _| Ok(area.clone()),
        |_, _| Ok(area.clone()),
        |_, _| Err(error),
        |_, _, _, _, _| panic!("must stop after open error"),
    );
    assert!(matches!(opened, Err(ClipperError::CoordinateOutOfRange)));

    let constructed = construct_candidate_anchored_bridge_using(
        input(
            &area,
            boundaries,
            &[],
            &lightning,
            flow(),
            0.0,
            CoordinateScale::Normal,
        ),
        |_, _| Ok(area.clone()),
        |_, _| Ok(area.clone()),
        |subject, _| Ok(subject.to_vec()),
        |_, _, _, _, _| Err(error),
    );
    assert!(matches!(
        constructed,
        Err(ClipperError::CoordinateOutOfRange)
    ));
}

#[test]
fn task22o61_injected_empty_area_and_outer_boundaries_reach_construct_in_order() {
    let output = construct_candidate_anchored_bridge_using(
        input(
            &[],
            Vec::new(),
            &[],
            &[],
            flow(),
            0.0,
            CoordinateScale::Normal,
        ),
        |_, _| panic!("empty Lightning skips closed intersection"),
        |_, _| panic!("empty Lightning skips expansion"),
        |_, _| panic!("empty Lightning skips open intersection"),
        |area, lines, _, _, _| {
            assert!(area.is_empty());
            assert!(lines.is_empty());
            Ok(Vec::new())
        },
    )
    .unwrap();
    assert!(output.boundary_polylines.is_empty());
    assert!(output.bridging_area.is_empty());
}
