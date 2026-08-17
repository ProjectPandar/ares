use crate::{
    SliceError,
    project_slice::{
        prepare_infill::{bridge_over_infill::transaction, combine_infill},
        tests::support::{KsrArchive, metadata},
    },
    slice_project,
};

#[tokio::test]
async fn task22o72_public_active_combination_error_precedes_the_incomplete_sink() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"infill_combination\": \"0\"",
        "\"infill_combination\": \"1\"",
    );
    combine_infill::reset_hooks();
    transaction::reset_hooks();

    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("infill_combination".to_owned())
    );
    assert_eq!(transaction::invocations(), 1);
    assert_eq!(transaction::disposals(), 1);
    assert_eq!(combine_infill::invocations(), 1);
    assert_eq!(combine_infill::disposals(), 0);

    combine_infill::reset_hooks();
    transaction::reset_hooks();
}

#[tokio::test]
async fn task22o72_public_density_above_source_f32_threshold_is_active() {
    let threshold = f64::from(0.00011_f32);
    let next = f64::from_bits(threshold.to_bits() + 1).to_string();
    combine_infill::reset_hooks();
    transaction::reset_hooks();

    assert_eq!(
        slice_project(combination_density_archive(&next).bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("infill_combination".to_owned())
    );
    assert_eq!(transaction::invocations(), 1);
    assert_eq!(transaction::disposals(), 1);
    assert_eq!(combine_infill::invocations(), 1);
    assert_eq!(combine_infill::disposals(), 0);

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
