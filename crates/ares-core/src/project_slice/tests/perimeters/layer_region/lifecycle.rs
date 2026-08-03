use crate::{SliceError, project_slice::perimeters::layer_region, slice_project};

use super::super::super::support::{KsrArchive, ksr_project, metadata};

const CONFIG: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn task22o16_public_lifecycle_invokes_layer_region_once_and_stays_incomplete() {
    layer_region::reset_finish_invocations();
    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(layer_region::finish_invocations(), 1);
}

#[tokio::test]
async fn task22o16_counterbore_preflight_precedes_layer_region_lifecycle() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        CONFIG,
        "\"counterbore_hole_bridging\": \"none\"",
        "\"counterbore_hole_bridging\": \"partiallybridge\"",
    );
    layer_region::reset_finish_invocations();
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned())
    );
    assert_eq!(layer_region::finish_invocations(), 0);
}
