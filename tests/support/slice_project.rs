#[path = "direct_catalog.rs"]
mod direct_catalog;

pub async fn slice_project(
    input: impl AsRef<[u8]>,
    metadata: ares_core::GenerationMetadata,
    context: ares_core::SlicerContext<'_>,
) -> Result<Vec<u8>, ares_core::SliceError> {
    let catalog = direct_catalog::catalog();
    ares_core::slice_project_plate_with_catalog(
        input,
        metadata,
        None,
        &ares_core::ConfigOverrides::default(),
        Some(&catalog),
        context,
    )
    .await
}
