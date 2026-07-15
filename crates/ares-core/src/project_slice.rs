use crate::{
    GenerationMetadata, SliceError, load_project,
    project::effective_config::resolve_bounded_project_config,
};

pub async fn slice_project(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let project = load_project(project)?;
    let _ = resolve_bounded_project_config(&project)?;
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
