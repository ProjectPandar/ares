mod precedence;

use crate::{
    SliceError,
    project_slice::{
        prepare_infill::{vertical_shell_filtering, vertical_shell_regularization},
        tests::support::{KsrArchive, metadata},
    },
    slice_project,
};

#[tokio::test]
async fn task22o23_public_lifecycle_runs_once_after_o22_and_remains_incomplete() {
    vertical_shell_regularization::reset_invocations();
    vertical_shell_filtering::reset_invocations();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(vertical_shell_regularization::invocations(), 1);
    assert_eq!(vertical_shell_filtering::invocations(), 1);
}

#[tokio::test]
async fn task22o23_earlier_capability_error_has_precedence() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"spiral_mode\": \"0\"",
        "\"spiral_mode\": \"1\"",
    );
    vertical_shell_filtering::reset_invocations();
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("spiral_mode".to_owned())
    );
    assert_eq!(vertical_shell_filtering::invocations(), 0);
}

#[tokio::test]
async fn task22o23_o22_failure_precedes_filtering() {
    vertical_shell_regularization::reset_geometry_hooks();
    vertical_shell_regularization::fail_geometry_at(
        vertical_shell_regularization::GeometryStep::Union,
    );
    vertical_shell_filtering::reset_invocations();
    assert!(matches!(
        slice_project(KsrArchive::new().bytes(), metadata()).await,
        Err(SliceError::InvalidInput(message))
            if message == "vertical-shell regularization geometry is outside the supported Clipper range"
    ));
    assert_eq!(vertical_shell_filtering::invocations(), 0);
    vertical_shell_regularization::reset_geometry_hooks();
}
