use crate::{SliceError, slice_project};

use super::super::super::support::{KsrArchive, metadata};
use crate::project_slice::prepare_infill::surface_type_detection::{
    invocations, reset_invocations,
};

#[tokio::test]
async fn task22o17_public_lifecycle_invokes_detection_once_and_remains_incomplete() {
    reset_invocations();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(invocations(), 1);
}

#[tokio::test]
async fn task22o17_counterbore_fails_before_detection() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"counterbore_hole_bridging\": \"none\"",
        "\"counterbore_hole_bridging\": \"partiallybridge\"",
    );
    reset_invocations();
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned())
    );
    assert_eq!(invocations(), 0);
}

#[tokio::test]
async fn task22o17_spiral_mode_fails_before_detection() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"spiral_mode\": \"0\"",
        "\"spiral_mode\": \"1\"",
    );
    reset_invocations();
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("spiral_mode".to_owned())
    );
    assert_eq!(invocations(), 0);
}
