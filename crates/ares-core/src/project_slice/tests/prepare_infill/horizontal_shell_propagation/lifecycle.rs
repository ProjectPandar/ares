use crate::{
    SliceError,
    project_slice::{
        prepare_infill::{
            horizontal_shell_promotion,
            horizontal_shell_propagation::{self, GeometryStep},
        },
        slice_project,
        tests::support::{KsrArchive, metadata},
    },
};

#[tokio::test]
async fn task22o26_public_lifecycle_runs_once_and_disposes_successor() {
    horizontal_shell_propagation::reset_hooks();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(horizontal_shell_propagation::invocations(), 1);
    assert_eq!(horizontal_shell_propagation::disposals(), 1);
    assert_eq!(horizontal_shell_propagation::commits(), 0);
}

#[tokio::test]
async fn task22o26_o25_error_precedes_invocation() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extra_solid_infills\": \"\"",
        "\"extra_solid_infills\": \"2147483648\"",
    );
    horizontal_shell_propagation::reset_hooks();
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::InvalidInput("invalid extra_solid_infills pattern".to_owned())
    );
    assert_eq!(horizontal_shell_propagation::invocations(), 0);
    assert_eq!(horizontal_shell_propagation::disposals(), 0);
}

#[tokio::test]
async fn task22o26_geometry_error_precedes_terminal_incomplete_and_rolls_back_o25() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_all\",",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_moderate\",",
    );
    horizontal_shell_promotion::reset_hooks();
    horizontal_shell_propagation::reset_hooks();
    horizontal_shell_propagation::fail_geometry_at(GeometryStep::SafetyIntersection);
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::InvalidInput(
            "horizontal-shell propagation geometry is outside the supported Clipper range"
                .to_owned()
        )
    );
    assert_eq!(horizontal_shell_propagation::invocations(), 1);
    assert_eq!(horizontal_shell_propagation::commits(), 0);
    assert_eq!(horizontal_shell_propagation::disposals(), 1);
    assert_eq!(horizontal_shell_promotion::disposals(), 1);
}
