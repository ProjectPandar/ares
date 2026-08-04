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
async fn task22o20_public_lifecycle_runs_once_and_remains_incomplete() {
    vertical_shell_projection::reset_invocations();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(vertical_shell_projection::invocations(), 1);
}

#[tokio::test]
async fn task22o20_earlier_errors_have_precedence_and_zero_invocations() {
    for (from, to, expected) in [
        (
            "\"spiral_mode\": \"0\"",
            "\"spiral_mode\": \"1\"",
            SliceError::UnsupportedProjectFeature("spiral_mode".to_owned()),
        ),
        (
            "\"counterbore_hole_bridging\": \"none\"",
            "\"counterbore_hole_bridging\": \"partiallybridge\"",
            SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned()),
        ),
        (
            "\"interface_shells\": \"0\"",
            "\"interface_shells\": \"1\"",
            SliceError::UnsupportedProjectFeature("interface_shells".to_owned()),
        ),
        (
            "\"enable_extra_bridge_layer\": \"disabled\"",
            "\"enable_extra_bridge_layer\": \"apply_to_all\"",
            SliceError::UnsupportedProjectFeature("enable_extra_bridge_layer".to_owned()),
        ),
    ] {
        let mut archive = KsrArchive::new();
        archive.replace_unique("Metadata/project_settings.config", from, to);
        vertical_shell_projection::reset_invocations();
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            expected
        );
        assert_eq!(vertical_shell_projection::invocations(), 0);
    }
    let (multi_region, _) = crate::project_slice::tests::region_fixture::modifier_projects();
    vertical_shell_projection::reset_invocations();
    assert_eq!(
        slice_project(multi_region, metadata()).await.unwrap_err(),
        SliceError::UnsupportedProjectFeature("multi_region_layer_slices".to_owned())
    );
    assert_eq!(vertical_shell_projection::invocations(), 0);
}

#[tokio::test]
async fn task22o20_o17_and_o19_geometry_failures_precede_projection() {
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    vertical_shell_projection::reset_invocations();
    assert!(matches!(
        slice_project(KsrArchive::new().bytes(), metadata()).await,
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
            slice_project(KsrArchive::new().bytes(), metadata()).await,
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
