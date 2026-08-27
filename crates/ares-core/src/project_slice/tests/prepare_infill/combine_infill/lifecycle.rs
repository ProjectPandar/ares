use crate::{
    project_slice::{
        prepare_infill::{bridge_over_infill::transaction, combine_infill},
        tests::support::{KsrArchive, metadata},
    },
    slice_project,
};

#[tokio::test]
async fn active_combination_reaches_complete_public_slice() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"infill_combination\": \"0\"",
        "\"infill_combination\": \"1\"",
    );
    combine_infill::reset_hooks();
    transaction::reset_hooks();

    let output = slice_project(archive.bytes(), metadata()).await.unwrap();
    assert!(!output.is_empty());
    assert_eq!(transaction::invocations(), 1);
    assert_eq!(transaction::disposals(), 1);
    assert_eq!(combine_infill::invocations(), 1);
    assert_eq!(combine_infill::disposals(), 1);

    combine_infill::reset_hooks();
    transaction::reset_hooks();
}

#[tokio::test]
async fn density_above_source_f32_threshold_combines_successfully() {
    let threshold = f64::from(0.00011_f32);
    let next = f64::from_bits(threshold.to_bits() + 1).to_string();
    combine_infill::reset_hooks();
    transaction::reset_hooks();

    let output = slice_project(combination_density_archive(&next).bytes(), metadata())
        .await
        .unwrap();
    assert!(!output.is_empty());
    assert_eq!(transaction::invocations(), 1);
    assert_eq!(transaction::disposals(), 1);
    assert_eq!(combine_infill::invocations(), 1);
    assert_eq!(combine_infill::disposals(), 1);

    combine_infill::reset_hooks();
    transaction::reset_hooks();
}

fn combination_density_archive(density: &str) -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"infill_combination\": \"0\"",
        "\"infill_combination\": \"1\"",
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_density\": \"15%\"",
        &format!("\"sparse_infill_density\": \"{density}%\""),
    );
    archive
}
