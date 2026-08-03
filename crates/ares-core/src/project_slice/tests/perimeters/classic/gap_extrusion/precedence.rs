use crate::{
    SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::perimeters::classic::{
        gap_extrusion::{self, PreparedPostClassicGapExtrusion},
        medial_gap::PreparedPostClassicMedialGap,
    },
};

use super::aligned::{combine, source_with_filter};

const OPTION_ERROR: &str = "invalid Orca option filter_out_gap_fill";

#[derive(Clone, Copy)]
enum Candidate {
    Flow,
    OpenOffset,
    Difference,
}

#[test]
fn task22o14_invalid_option_before_each_candidate_prevents_all_staging() {
    for candidate in [
        Candidate::Flow,
        Candidate::OpenOffset,
        Candidate::Difference,
    ] {
        assert_precedence(candidate, true);
    }
}

#[test]
fn task22o14_invalid_option_after_each_candidate_prevents_all_staging() {
    for candidate in [
        Candidate::Flow,
        Candidate::OpenOffset,
        Candidate::Difference,
    ] {
        assert_precedence(candidate, false);
    }
}

fn assert_precedence(candidate: Candidate, invalid_first: bool) {
    let invalid = source_with_filter("-1");
    let mut failing = source_with_filter("0");
    inject_failure(&mut failing, candidate);
    let combined = if invalid_first {
        combine(invalid, failing)
    } else {
        combine(failing, invalid)
    };

    gap_extrusion::reset_stage_surface_invocations();
    assert_option_error(gap_extrusion::finish(combined));
    assert_eq!(gap_extrusion::stage_surface_invocations(), 0);
}

fn inject_failure(source: &mut PreparedPostClassicMedialGap, candidate: Candidate) {
    let (object, record, surface) = retained_surface(source);
    match candidate {
        Candidate::Flow => {
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
        }
        Candidate::OpenOffset => {
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
        }
        Candidate::Difference => {
            source.predecessor.objects[object]
                .predecessor
                .predecessor
                .records[record]
                .as_mut()
                .unwrap()
                .surfaces[surface]
                .last = vec![out_of_range_expolygon()];
        }
    }
}

fn out_of_range_expolygon() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(i64::MAX - 20, 0),
            Point::new(i64::MAX - 10, 0),
            Point::new(i64::MAX - 10, 10),
            Point::new(i64::MAX - 20, 10),
        ]),
        Vec::new(),
    )
}

fn retained_surface(source: &PreparedPostClassicMedialGap) -> (usize, usize, usize) {
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

fn assert_option_error(result: Result<PreparedPostClassicGapExtrusion, SliceError>) {
    match result {
        Err(SliceError::InvalidInput(message)) => assert_eq!(message, OPTION_ERROR),
        Err(error) => panic!("unexpected O14 error: {error:?}"),
        Ok(_) => panic!("invalid filter unexpectedly produced an O14 successor"),
    }
}
