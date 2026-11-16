#[path = "direct_catalog.rs"]
mod direct_catalog;

pub async fn slice_project_plate_validated(
    input: impl AsRef<[u8]>,
    metadata: ares_core::GenerationMetadata,
    plate: Option<u32>,
    overrides: &ares_core::ConfigOverrides,
    context: ares_core::SlicerContext<'_>,
) -> Result<Vec<u8>, ares_core::SliceError> {
    let catalog = direct_catalog::catalog();
    ares_core::slice_project_plate_validated_with_catalog(
        input,
        ares_core::DirectProjectSliceRequest {
            plate,
            overrides,
            system_catalog: Some(&catalog),
        },
        metadata,
        context,
    )
    .await
}
