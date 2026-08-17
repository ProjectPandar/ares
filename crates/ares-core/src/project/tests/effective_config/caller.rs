use crate::{GenerationMetadata, SliceError, load_project, slice_project};

use super::support::ProjectParts;

#[tokio::test]
async fn slice_project_reports_materialized_cardinality_before_incomplete() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "Metadata/project_settings.config",
        r#"{"layer_height":"0.2"}"#,
        r#"{"layer_height":"0.2","filament_map":[]}"#,
    );
    let bytes = parts.bytes();
    load_project(&bytes).unwrap();

    assert_eq!(
        slice_project(bytes, metadata()).await.unwrap_err(),
        SliceError::InvalidInput("invalid Orca option filament_map".to_owned())
    );
}

#[tokio::test]
async fn slice_project_keeps_malformed_archive_before_resolution() {
    let bytes = b"not a 3MF archive";
    let load_error = load_project(bytes).unwrap_err();

    assert_ne!(load_error, SliceError::ProjectSlicingIncomplete);
    assert_eq!(
        slice_project(bytes, metadata()).await.unwrap_err(),
        load_error
    );
}

fn metadata() -> GenerationMetadata {
    GenerationMetadata::deterministic(2026, 7, 15, 1, 2, 3)
}
