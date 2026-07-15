use crate::{GenerationMetadata, SliceError, load_project, slice_project};

use super::support::ProjectParts;

const FLUSH_MATRIX: &str = concat!(
    "\t\"flush_volumes_matrix\": [\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"0\"\r\n",
    "\t]",
);
const INVALID_FLUSH_MATRIX: &str = "\t\"flush_volumes_matrix\": [\r\n\t\t\"0\"\r\n\t]";

#[tokio::test]
async fn project_config_export_preserves_archive_error_precedence() {
    let bytes = b"not a 3MF archive";
    let load_error = load_project(bytes).unwrap_err();

    assert_ne!(load_error, SliceError::ProjectSlicingIncomplete);
    assert_eq!(
        slice_project(bytes, metadata()).await.unwrap_err(),
        load_error
    );
}

#[tokio::test]
async fn project_config_export_preserves_materialization_error_precedence() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "Metadata/project_settings.config",
        r#"{"layer_height":"0.2"}"#,
        r#"{"layer_height":"0.2","filament_map":[]}"#,
    );

    assert_eq!(
        slice_project(parts.bytes(), metadata()).await.unwrap_err(),
        SliceError::InvalidInput("invalid Orca option filament_map".to_owned())
    );
}

#[tokio::test]
async fn project_config_export_returns_exact_bambu_flush_matrix_error() {
    let parts = invalid_flush_matrix_fixture();

    assert_eq!(
        slice_project(parts.bytes(), metadata()).await.unwrap_err(),
        SliceError::InvalidInput(
            "Flush volumes matrix do not match to the correct size!".to_owned()
        )
    );
}

#[tokio::test]
async fn project_config_export_runs_valid_bambu_at_source_plate_zero_before_incomplete() {
    assert_eq!(
        slice_project(ProjectParts::fixture().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

#[tokio::test]
async fn project_config_export_skips_non_bambu_even_with_invalid_writer_data() {
    let mut parts = invalid_flush_matrix_fixture();
    parts.replace(
        "Metadata/project_settings.config",
        r#""printer_model": "Bambu Lab X2D""#,
        r#""printer_model": "Generic FFF""#,
    );

    assert_eq!(
        slice_project(parts.bytes(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

fn invalid_flush_matrix_fixture() -> ProjectParts {
    let mut parts = ProjectParts::fixture();
    parts.replace(
        "Metadata/project_settings.config",
        FLUSH_MATRIX,
        INVALID_FLUSH_MATRIX,
    );
    parts
}

fn metadata() -> GenerationMetadata {
    GenerationMetadata::deterministic(2026, 7, 15, 1, 2, 3)
}
