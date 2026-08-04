use crate::project_slice::{
    prepare_infill::{
        surface_type_detection,
        vertical_shell_projection::{self, GeometryStep as ProjectionStep},
        vertical_shell_trimming::{self, GeometryStep},
        vertical_shells,
    },
    tests::support::{KsrArchive, metadata},
};
use crate::{SliceError, slice_project};

#[tokio::test]
async fn task22o21_public_lifecycle_runs_once_and_remains_incomplete() {
    vertical_shell_trimming::reset_invocations();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(vertical_shell_trimming::invocations(), 1);
}

#[tokio::test]
async fn task22o21_earlier_capability_errors_have_precedence() {
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
        vertical_shell_trimming::reset_invocations();
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            expected
        );
        assert_eq!(vertical_shell_trimming::invocations(), 0);
    }
    let (multi_region, _) = crate::project_slice::tests::region_fixture::modifier_projects();
    vertical_shell_trimming::reset_invocations();
    assert_eq!(
        slice_project(multi_region, metadata()).await.unwrap_err(),
        SliceError::UnsupportedProjectFeature("multi_region_layer_slices".to_owned())
    );
    assert_eq!(vertical_shell_trimming::invocations(), 0);
}

#[tokio::test]
async fn task22o21_o17_o19_and_o20_failures_precede_trimming() {
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    vertical_shell_trimming::reset_invocations();
    assert!(matches!(
        slice_project(KsrArchive::new().bytes(), metadata()).await,
        Err(SliceError::InvalidInput(message))
            if message == "surface-type detection geometry is outside the supported Clipper range"
    ));
    assert_eq!(vertical_shell_trimming::invocations(), 0);
    surface_type_detection::reset_geometry_hooks();

    for step in [
        vertical_shells::GeometryStep::Top,
        vertical_shells::GeometryStep::Bottom,
    ] {
        vertical_shells::reset_geometry_hooks();
        vertical_shells::fail_geometry_at(step);
        vertical_shell_trimming::reset_invocations();
        assert!(
            slice_project(KsrArchive::new().bytes(), metadata())
                .await
                .is_err()
        );
        assert_eq!(vertical_shell_trimming::invocations(), 0);
        vertical_shells::reset_geometry_hooks();
    }

    for step in [
        ProjectionStep::TopVisit,
        ProjectionStep::BottomVisit,
        ProjectionStep::HoleIntersection,
        ProjectionStep::ShellUnion,
    ] {
        vertical_shell_projection::reset_geometry_hooks();
        vertical_shell_projection::fail_geometry_at(step);
        vertical_shell_trimming::reset_invocations();
        assert!(
            slice_project(KsrArchive::new().bytes(), metadata())
                .await
                .is_err()
        );
        assert_eq!(vertical_shell_trimming::invocations(), 0);
        vertical_shell_projection::reset_geometry_hooks();
    }
}

#[tokio::test]
async fn task22o21_each_o20_anchor_failure_precedes_trimming() {
    for (step, top) in [
        (ProjectionStep::TopAnchorOffset, true),
        (ProjectionStep::TopAnchorIntersection, true),
        (ProjectionStep::BottomAnchorOffset, false),
        (ProjectionStep::BottomAnchorIntersection, false),
    ] {
        let mut archive = KsrArchive::new();
        if top {
            archive.replace_unique(
                "Metadata/project_settings.config",
                "\"top_shell_layers\": \"5\"",
                "\"top_shell_layers\": \"1\"",
            );
            archive.replace_unique(
                "Metadata/project_settings.config",
                "\"top_shell_thickness\": \"1\"",
                "\"top_shell_thickness\": \"0\"",
            );
        } else {
            archive.replace_unique(
                "Metadata/project_settings.config",
                "\"bottom_shell_layers\": \"3\"",
                "\"bottom_shell_layers\": \"1\"",
            );
        }
        vertical_shell_projection::reset_geometry_hooks();
        vertical_shell_projection::fail_geometry_at(step);
        vertical_shell_trimming::reset_invocations();
        assert!(slice_project(archive.bytes(), metadata()).await.is_err());
        assert_eq!(vertical_shell_trimming::invocations(), 0);
        vertical_shell_projection::reset_geometry_hooks();
    }
}

#[test]
fn task22o21_later_object_failure_is_whole_project_transactional() {
    let mut first = super::fixture::prepare_o20(KsrArchive::new().bytes());
    let mut second = super::fixture::prepare_o20(KsrArchive::new().bytes());
    first.objects.push(second.objects.pop().unwrap());
    first.caches.push(second.caches.pop().unwrap());
    first.projections.push(second.projections.pop().unwrap());
    first
        .predecessor
        .objects
        .push(second.predecessor.objects.pop().unwrap());
    let (probe, dropped) = first.predecessor.drop_probe_observer();
    vertical_shell_trimming::reset_geometry_hooks();
    vertical_shell_trimming::fail_geometry_at_occurrence(GeometryStep::SafetyOffset, 461);
    let error = match vertical_shell_trimming::prepare(first) {
        Err(error) => error,
        Ok(_) => panic!("later-object failure must expose no O21 successor"),
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "vertical-shell internal trimming geometry is outside the supported Clipper range"
                .to_owned()
        )
    );
    let events = vertical_shell_trimming::geometry_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| **event == GeometryStep::SafetyOffset)
            .count(),
        461
    );
    assert_eq!(events.last(), Some(&GeometryStep::SafetyOffset));
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    vertical_shell_trimming::reset_geometry_hooks();
}
