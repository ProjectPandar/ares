use crate::{GenerationMetadata, SliceError, load_project, slice_project};

use super::support::ProjectParts;

// OrcaSlicer.cpp:6029-6031: an empty filament_map is grown to the filament
// count with 1s instead of being rejected, so slicing proceeds past
// materialization and fails later at layer detection.
#[tokio::test]
async fn slice_project_grows_empty_filament_map_before_failing_on_layers() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "Metadata/project_settings.config",
        r#"{"layer_height":"0.2"}"#,
        r#"{"layer_height":"0.2","filament_map":[]}"#,
    );
    let bytes = parts.bytes();
    load_project(&bytes).unwrap();

    assert_eq!(
        slice_project(bytes, metadata(), crate::SlicerContext::new(0))
            .await
            .unwrap_err(),
        SliceError::InvalidInput("no layers were detected".to_owned())
    );
}

#[tokio::test]
async fn slice_project_keeps_malformed_archive_before_resolution() {
    let bytes = b"not a 3MF archive";
    let load_error = load_project(bytes).unwrap_err();

    assert_ne!(load_error, SliceError::ProjectSlicingIncomplete);
    assert_eq!(
        slice_project(bytes, metadata(), crate::SlicerContext::new(0))
            .await
            .unwrap_err(),
        load_error
    );
}

fn metadata() -> GenerationMetadata {
    GenerationMetadata::deterministic(2026, 7, 15, 1, 2, 3)
}
