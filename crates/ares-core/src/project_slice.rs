use crate::{GenerationMetadata, SliceError, load_project};

pub async fn slice_project(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let project = load_project(project)?;
    let documents = project.documents();
    let _ = (
        &documents.model_settings,
        &documents.slice_info,
        &documents.filament_sequences,
        &documents.plate_documents,
        metadata,
    );
    Err(SliceError::ProjectSlicingIncomplete)
}
