use crate::{
    SliceError,
    project_slice::perimeters::{
        classic::infill_boundary::{self, GeometryStep},
        prepare_post_classic_gap_extrusion, prepare_post_classic_infill_boundary,
    },
};

use super::super::super::super::support::{KsrArchive, ksr_project};

const GEOMETRY_ERROR: &str =
    "Classic infill-boundary geometry is outside the supported Clipper range";
const CONFIG: &str = "Metadata/project_settings.config";

#[test]
fn task22o15_geometry_event_order_keeps_mandatory_empty_and_nonempty_top_calls() {
    infill_boundary::reset_geometry_hooks();
    prepare_post_classic_infill_boundary(ksr_project()).unwrap();
    let events = infill_boundary::geometry_events();

    assert!(events.windows(4).any(|window| {
        window
            == [
                GeometryStep::OrdinaryOffset,
                GeometryStep::TopOffset,
                GeometryStep::TopIntersection,
                GeometryStep::SurfaceAppend,
            ]
    }));
    assert!(events.windows(5).any(|window| {
        window
            == [
                GeometryStep::TopOffset,
                GeometryStep::TopIntersection,
                GeometryStep::TopOverlapOffset,
                GeometryStep::TopUnion,
                GeometryStep::SurfaceAppend,
            ]
    }));
    assert!(events.windows(2).any(|window| {
        window
            == [
                GeometryStep::SurfaceAppend,
                GeometryStep::ExtraPerimeterGuard,
            ]
    }));
    assert_eq!(
        events
            .iter()
            .filter(|&&step| step == GeometryStep::AggregateUnion)
            .count(),
        events
            .iter()
            .filter(|&&step| step == GeometryStep::OrdinaryOffset)
            .count()
    );
}

#[test]
fn task22o15_empty_remaining_still_runs_aggregate_and_mandatory_top_calls() {
    let mut source = prepare_post_classic_gap_extrusion(ksr_project()).unwrap();
    for surface in source
        .objects
        .iter_mut()
        .flat_map(|object| object.records.iter_mut().flatten())
        .flat_map(|record| &mut record.surfaces)
    {
        surface.remaining.clear();
    }
    infill_boundary::reset_geometry_hooks();
    infill_boundary::finish(source).unwrap();
    let events = infill_boundary::geometry_events();
    assert!(!events.contains(&GeometryStep::Simplify));
    assert!(events.contains(&GeometryStep::AggregateUnion));
    assert!(
        events
            .windows(2)
            .any(|window| { window == [GeometryStep::TopOffset, GeometryStep::TopIntersection] })
    );
}

macro_rules! operation_failure_test {
    ($name:ident, $step:expr, $project:expr) => {
        #[test]
        fn $name() {
            let source = prepare_post_classic_gap_extrusion($project).unwrap();
            infill_boundary::reset_geometry_hooks();
            infill_boundary::fail_geometry_at($step);
            match infill_boundary::finish(source) {
                Err(SliceError::InvalidInput(message)) => assert_eq!(message, GEOMETRY_ERROR),
                Err(error) => panic!("unexpected O15 error: {error:?}"),
                Ok(_) => panic!("injected O15 operation unexpectedly succeeded"),
            }
        }
    };
}

operation_failure_test!(
    task22o15_simplification_failure_is_stable,
    GeometryStep::Simplify,
    ksr_project()
);
operation_failure_test!(
    task22o15_aggregate_union_failure_is_stable,
    GeometryStep::AggregateUnion,
    ksr_project()
);
operation_failure_test!(
    task22o15_ordinary_offset_failure_is_stable,
    GeometryStep::OrdinaryOffset,
    ksr_project()
);
operation_failure_test!(
    task22o15_top_offset_failure_is_stable_even_for_empty_top_fills,
    GeometryStep::TopOffset,
    ksr_project()
);
operation_failure_test!(
    task22o15_top_intersection_failure_is_stable,
    GeometryStep::TopIntersection,
    ksr_project()
);
operation_failure_test!(
    task22o15_top_overlap_offset_failure_is_stable,
    GeometryStep::TopOverlapOffset,
    ksr_project()
);
operation_failure_test!(
    task22o15_top_union_failure_is_stable,
    GeometryStep::TopUnion,
    ksr_project()
);
operation_failure_test!(
    task22o15_no_overlap_two_failure_is_stable,
    GeometryStep::NoOverlapTwo,
    ksr_project()
);
operation_failure_test!(
    task22o15_final_top_union_failure_is_stable,
    GeometryStep::FinalTopUnion,
    ksr_project()
);

#[test]
fn task22o15_no_overlap_one_failure_is_stable() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        CONFIG,
        "\"infill_wall_overlap\": \"15%\"",
        "\"infill_wall_overlap\": \"100%\"",
    );
    let source = prepare_post_classic_gap_extrusion(archive.bytes()).unwrap();
    infill_boundary::reset_geometry_hooks();
    infill_boundary::fail_geometry_at(GeometryStep::NoOverlapOne);
    match infill_boundary::finish(source) {
        Err(SliceError::InvalidInput(message)) => assert_eq!(message, GEOMETRY_ERROR),
        Err(error) => panic!("unexpected O15 error: {error:?}"),
        Ok(_) => panic!("injected no-overlap-one unexpectedly succeeded"),
    }
}
