use crate::{
    GenerationMetadata, SliceError, load_project,
    options::{is_bambu_project, write_config_block},
    project::effective_config::resolve_bounded_project_config,
};

pub async fn slice_project(
    project: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let project = load_project(project)?;
    let resolved = resolve_bounded_project_config(&project)?;
    let mut config_block = Vec::new();
    if is_bambu_project(&resolved.views.full) {
        write_config_block(&resolved.views, 0, &mut config_block)?;
    }
    let documents = project.documents();
    let _ = (
        &documents.model_settings,
        &documents.slice_info,
        &documents.filament_sequences,
        &documents.plate_documents,
        metadata,
        config_block,
    );
    Err(SliceError::ProjectSlicingIncomplete)
}
