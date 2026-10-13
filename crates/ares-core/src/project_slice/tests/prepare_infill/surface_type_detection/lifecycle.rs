use crate::{SliceError, slice_project};

use super::super::super::support::{KsrArchive, metadata};
use crate::project_slice::prepare_infill::surface_type_detection::{
    invocations, reset_invocations,
};

#[tokio::test]
async fn task22o17_counterbore_fails_before_detection() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"counterbore_hole_bridging\": \"none\"",
        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"wall_generator\": \"classic\"",
        "\"wall_generator\": \"arachne\"",
    );
    reset_invocations();
    assert_eq!(
        slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned())
    );
    assert_eq!(invocations(), 0);
}
