use crate::{SliceError, load_project, slice_project};

use super::{
    super::state::{ProjectSliceState, prepare_project_slice},
    support::{KsrArchive, metadata},
};

const MIN_LAYER_HEIGHTS: &str = concat!(
    "\t\"min_layer_height\": [\r\n",
    "\t\t\"0.08\",\r\n",
    "\t\t\"0.08\"\r\n",
    "\t]",
);
const MAX_LAYER_HEIGHTS: &str = concat!(
    "\t\"max_layer_height\": [\r\n",
    "\t\t\"0.28\",\r\n",
    "\t\t\"0.28\"\r\n",
    "\t]",
);
const EMPTY_MIN_LAYER_HEIGHTS: &str = "\t\"min_layer_height\": []";
const EMPTY_MAX_LAYER_HEIGHTS: &str = "\t\"max_layer_height\": []";
const FILAMENT_DIAMETERS: &str = concat!(
    "\t\"filament_diameter\": [\r\n",
    "\t\t\"1.75\",\r\n",
    "\t\t\"1.75\"\r\n",
    "\t]",
);
const EMPTY_FILAMENT_DIAMETERS: &str = "\t\"filament_diameter\": []";

#[tokio::test]
async fn task22a_lifecycle_preserves_archive_effective_and_writer_precedence() {
    let malformed = b"not a 3MF archive";
    assert_eq!(
        slice_project(malformed, metadata()).await.unwrap_err(),
        load_project(malformed).unwrap_err()
    );

    let mut archive = invalid_bambu_chain();
    archive.replace(
        "Metadata/project_settings.config",
        MIN_LAYER_HEIGHTS,
        EMPTY_MIN_LAYER_HEIGHTS,
    );
    archive.replace(
        "Metadata/project_settings.config",
        MAX_LAYER_HEIGHTS,
        EMPTY_MAX_LAYER_HEIGHTS,
    );
    assert_eq!(
        slice_error(&archive).await,
        SliceError::InvalidInput("invalid Orca option min_layer_height".to_owned())
    );

    archive.replace(
        "Metadata/project_settings.config",
        EMPTY_MIN_LAYER_HEIGHTS,
        MIN_LAYER_HEIGHTS,
    );
    assert_eq!(
        slice_error(&archive).await,
        SliceError::InvalidInput("invalid Orca option max_layer_height".to_owned())
    );

    archive.replace(
        "Metadata/project_settings.config",
        EMPTY_MAX_LAYER_HEIGHTS,
        MAX_LAYER_HEIGHTS,
    );
    archive.replace(
        "Metadata/project_settings.config",
        FILAMENT_DIAMETERS,
        EMPTY_FILAMENT_DIAMETERS,
    );
    assert_eq!(
        slice_error(&archive).await,
        SliceError::InvalidInput("invalid Orca option filament_diameter".to_owned())
    );
    archive.replace(
        "Metadata/project_settings.config",
        EMPTY_FILAMENT_DIAMETERS,
        FILAMENT_DIAMETERS,
    );
    assert_eq!(slice_error(&archive).await, flush_matrix_error());
}

#[tokio::test]
async fn task22a_lifecycle_reaches_planning_error_then_incomplete() {
    let mut archive = invalid_bambu_chain();
    assert_eq!(slice_error(&archive).await, flush_matrix_error());

    archive.repair_flush_matrix();
    assert_eq!(
        slice_error(&archive).await,
        SliceError::UnsupportedProjectFeature("raft_layers".to_owned())
    );

    set_scalar(&mut archive, "raft_layers", "1", "0");
    assert_eq!(
        slice_error(&archive).await,
        SliceError::InvalidInput("invalid Orca option layer_height".to_owned())
    );

    set_scalar(&mut archive, "layer_height", "0", "0.2");
    assert_eq!(
        slice_error(&archive).await,
        SliceError::ProjectSlicingIncomplete
    );
}

#[tokio::test]
async fn task22a_non_bambu_skips_writer_but_runs_planning() {
    let mut archive = invalid_bambu_chain();
    set_scalar(
        &mut archive,
        "printer_model",
        "Bambu Lab X2D",
        "Generic FFF",
    );
    assert_eq!(
        slice_error(&archive).await,
        SliceError::UnsupportedProjectFeature("raft_layers".to_owned())
    );

    set_scalar(&mut archive, "raft_layers", "1", "0");
    assert_eq!(
        slice_error(&archive).await,
        SliceError::InvalidInput("invalid Orca option layer_height".to_owned())
    );

    set_scalar(&mut archive, "layer_height", "0", "0.2");
    let state = prepare_project_slice(archive.clone().bytes()).unwrap();
    assert!(state.config_block.is_none());
    assert_eq!(
        slice_error(&archive).await,
        SliceError::ProjectSlicingIncomplete
    );
}

#[test]
fn task22a_private_state_owns_single_project_config_block_and_plans() {
    let ProjectSliceState {
        project,
        resolved,
        config_block,
        planned_objects,
    } = prepare_project_slice(KsrArchive::new().bytes()).unwrap();

    assert_eq!(project.objects().len(), 1);
    assert_eq!(resolved.objects.len(), 1);
    assert_eq!(resolved.print_object_count, 1);
    assert_eq!(planned_objects.len(), 1);
    let plan = &planned_objects[0];
    assert_eq!(
        plan.source_object_index,
        resolved.objects[0].source_object_index
    );
    assert_eq!(plan.transform_index, 0);
    assert_eq!(resolved.objects[0].print_objects.len(), 1);
    assert!(project.objects().get(plan.source_object_index).is_some());

    let block = config_block.unwrap();
    assert!(block.starts_with(b"; CONFIG_BLOCK_START\n"));
    assert!(block.ends_with(b"; CONFIG_BLOCK_END\n\n"));
}

fn invalid_bambu_chain() -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.invalidate_flush_matrix();
    set_scalar(&mut archive, "raft_layers", "0", "1");
    set_scalar(&mut archive, "layer_height", "0.2", "0");
    archive
}

fn set_scalar(archive: &mut KsrArchive, key: &str, from: &str, to: &str) {
    archive.replace(
        "Metadata/project_settings.config",
        &format!("\t\"{key}\": \"{from}\","),
        &format!("\t\"{key}\": \"{to}\","),
    );
}

async fn slice_error(archive: &KsrArchive) -> SliceError {
    slice_project(archive.clone().bytes(), metadata())
        .await
        .unwrap_err()
}

fn flush_matrix_error() -> SliceError {
    SliceError::InvalidInput("Flush volumes matrix do not match to the correct size!".to_owned())
}
