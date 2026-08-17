use crate::{
    SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::perimeters::{classic::gap_extrusion, prepare_post_classic_medial_gap},
};

use super::super::super::super::support::ksr_project;
#[test]
fn task22o14_invalid_derived_flow_is_transactional() {
    let mut source = prepare_post_classic_medial_gap(ksr_project()).unwrap();
    let (probe, dropped) = source.predecessor.drop_probe_observer();
    let (object, record, _) = retained_surface(&source);
    let prelude = &mut source.predecessor.objects[object]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.records[record]
        .as_mut()
        .unwrap()
        .solid_infill_flow
        .height = f32::NAN;
    assert!(probe.upgrade().is_some());
    assert!(!dropped.load(std::sync::atomic::Ordering::SeqCst));
    assert_stage_error(
        gap_extrusion::finish(source),
        "Classic variable-width gap flow is invalid",
    );
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o14_open_offset_range_error_is_transactional() {
    let mut source = prepare_post_classic_medial_gap(ksr_project()).unwrap();
    let (object, record, surface) = retained_surface(&source);
    let polyline = &mut source.objects[object].records[record]
        .as_mut()
        .unwrap()
        .surfaces[surface]
        .medial
        .as_mut()
        .unwrap()
        .polylines[0];
    polyline.points = vec![
        Point::new(i64::MAX - 1_000, 0),
        Point::new(i64::MAX - 500, 0),
    ];
    polyline.width = vec![400_000.0, 400_000.0];
    assert_stage_error(
        gap_extrusion::finish(source),
        "Classic gap-extrusion geometry is outside the supported Clipper range",
    );
}

#[test]
fn task22o14_difference_range_error_is_transactional() {
    let mut source = prepare_post_classic_medial_gap(ksr_project()).unwrap();
    let (object, record, surface) = retained_surface(&source);
    source.predecessor.objects[object]
        .predecessor
        .predecessor
        .records[record]
        .as_mut()
        .unwrap()
        .surfaces[surface]
        .last = vec![ExPolygon::new(
        Polygon::new(vec![
            Point::new(i64::MAX - 20, 0),
            Point::new(i64::MAX - 10, 0),
            Point::new(i64::MAX - 10, 10),
            Point::new(i64::MAX - 20, 10),
        ]),
        Vec::new(),
    )];
    assert_stage_error(
        gap_extrusion::finish(source),
        "Classic gap-extrusion geometry is outside the supported Clipper range",
    );
}

fn retained_surface(
    source: &crate::project_slice::perimeters::classic::medial_gap::PreparedPostClassicMedialGap,
) -> (usize, usize, usize) {
    source
        .objects
        .iter()
        .enumerate()
        .find_map(|(object_index, object)| {
            object
                .records
                .iter()
                .enumerate()
                .find_map(|(record_index, record)| {
                    record.as_ref()?.surfaces.iter().enumerate().find_map(
                        |(surface_index, surface)| {
                            surface
                                .medial
                                .as_ref()
                                .is_some_and(|domain| !domain.polylines.is_empty())
                                .then_some((object_index, record_index, surface_index))
                        },
                    )
                })
        })
        .expect("KSR must contain a retained medial polyline")
}

fn assert_stage_error(
    result: Result<
        crate::project_slice::perimeters::classic::PreparedPostClassicGapExtrusion,
        SliceError,
    >,
    expected: &str,
) {
    match result {
        Err(SliceError::InvalidInput(message)) => assert_eq!(message, expected),
        Err(error) => panic!("unexpected O14 error: {error:?}"),
        Ok(_) => panic!("O14 failure case unexpectedly succeeded"),
    }
}
