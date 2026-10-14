use crate::project_slice::{
    prepare_infill::{
        surface_type_detection,
        vertical_shell_projection::{self, GeometryStep},
        vertical_shells,
    },
    tests::support::{KsrArchive, metadata},
};
use crate::{SliceError, slice_project};

use super::fixture;

#[tokio::test]
async fn task22o20_o17_and_o19_geometry_failures_precede_projection() {
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    vertical_shell_projection::reset_invocations();
    assert!(matches!(
        slice_project(KsrArchive::new().bytes(), metadata(), crate::SlicerContext::new(0)).await,
        Err(SliceError::InvalidInput(message))
            if message == "surface-type detection geometry is outside the supported Clipper range"
    ));
    assert_eq!(vertical_shell_projection::invocations(), 0);
    surface_type_detection::reset_geometry_hooks();

    for step in [
        vertical_shells::GeometryStep::Top,
        vertical_shells::GeometryStep::Bottom,
    ] {
        vertical_shells::reset_geometry_hooks();
        vertical_shells::fail_geometry_at(step);
        vertical_shell_projection::reset_invocations();
        assert!(matches!(
            slice_project(KsrArchive::new().bytes(), metadata(), crate::SlicerContext::new(0)).await,
            Err(SliceError::InvalidInput(message))
                if message == "vertical-shell cache geometry is outside the supported Clipper range"
        ));
        assert_eq!(vertical_shell_projection::invocations(), 0);
        vertical_shells::reset_geometry_hooks();
    }
}

#[test]
fn task22o20_active_multi_object_later_slot_failure_is_transactional_and_ordered() {
    let mut first = fixture::prepare_o19(KsrArchive::new().bytes());
    let mut second = fixture::prepare_o19(KsrArchive::new().bytes());
    first.objects.push(second.objects.pop().unwrap());
    first.caches.push(second.caches.pop().unwrap());
    first
        .predecessor
        .objects
        .push(second.predecessor.objects.pop().unwrap());
    let (probe, dropped) = first.predecessor.drop_probe_observer();

    vertical_shell_projection::reset_geometry_hooks();
    vertical_shell_projection::fail_geometry_at_occurrence(GeometryStep::TopVisit, 1_835);
    let error = match vertical_shell_projection::prepare(first) {
        Err(error) => error,
        Ok(_) => panic!("later-object failure must expose no partial successor"),
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "vertical-shell projection geometry is outside the supported Clipper range".to_owned()
        )
    );
    let events = vertical_shell_projection::geometry_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| **event == GeometryStep::TopVisit)
            .count(),
        1_835
    );
    assert_eq!(events.last(), Some(&GeometryStep::TopVisit));
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    vertical_shell_projection::reset_geometry_hooks();
}
