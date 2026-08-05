use crate::{
    SliceError,
    project_slice::{
        prepare_infill::{vertical_shell_assignment, vertical_shell_filtering},
        slice_project,
        tests::support::{KsrArchive, metadata},
    },
};

#[tokio::test]
async fn task22o24_public_lifecycle_runs_once_after_o23_and_remains_incomplete() {
    vertical_shell_filtering::reset_invocations();
    vertical_shell_assignment::reset_invocations();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(vertical_shell_filtering::invocations(), 1);
    assert_eq!(vertical_shell_assignment::invocations(), 1);
}

#[tokio::test]
async fn task22o24_predecessor_failure_has_zero_assignment_invocations() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"spiral_mode\": \"0\"",
        "\"spiral_mode\": \"1\"",
    );
    vertical_shell_assignment::reset_invocations();
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("spiral_mode".to_owned())
    );
    assert_eq!(vertical_shell_assignment::invocations(), 0);
}

#[tokio::test]
async fn task22o24_each_o23_geometry_failure_precedes_assignment() {
    for step in [
        vertical_shell_filtering::GeometryStep::NeighborIntersection,
        vertical_shell_filtering::GeometryStep::ClosingGrow,
        vertical_shell_filtering::GeometryStep::ClosingShrink,
        vertical_shell_filtering::GeometryStep::VisibilityDifference,
        vertical_shell_filtering::GeometryStep::CandidateExpansion,
        vertical_shell_filtering::GeometryStep::ProtectionDifference,
    ] {
        vertical_shell_filtering::reset_geometry_hooks();
        vertical_shell_filtering::fail_geometry_at(step);
        vertical_shell_assignment::reset_invocations();
        assert!(matches!(
            slice_project(KsrArchive::new().bytes(), metadata()).await,
            Err(SliceError::InvalidInput(message))
                if message == "vertical-shell tiny-island filtering geometry is outside the supported Clipper range"
        ));
        assert_eq!(vertical_shell_assignment::invocations(), 0);
    }
    vertical_shell_filtering::reset_geometry_hooks();
}
