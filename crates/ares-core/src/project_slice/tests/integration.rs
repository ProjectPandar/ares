use crate::{
    Point3d, ProjectVolumeType, SliceError, Transform3d, geometry::CoordinateScale, load_project,
    slice_project,
};

use super::{
    super::state::{ProjectSliceState, prepare_project_slice},
    raw_support::{
        bfs_restart_request, intersections, mesh_volume, ordinal_gap_object, planned_layers,
    },
    support::{KsrArchive, identity_resolved, metadata, object, project_volume, transform},
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
async fn task22a_lifecycle_reaches_planning_error_precedence() {
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
}

#[tokio::test]
async fn task22a_non_bambu_writes_config_block_and_runs_planning() {
    let mut archive = invalid_bambu_chain();
    set_scalar(
        &mut archive,
        "printer_model",
        "Bambu Lab X2D",
        "Generic FFF",
    );
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
}

#[test]
fn task22a_private_state_owns_single_project_config_block_and_plans() {
    let ProjectSliceState {
        project,
        resolved,
        config_block,
        scale,
        intersected_objects,
    } = prepare_project_slice(KsrArchive::new().bytes(), None).unwrap();

    assert_eq!(project.objects().len(), 1);
    assert_eq!(resolved.objects.len(), 1);
    assert_eq!(resolved.print_object_count, 1);
    assert_eq!(scale, CoordinateScale::Normal);
    assert_eq!(intersected_objects.len(), 1);
    let plan = &intersected_objects[0].plan;
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

#[test]
fn task22b_project_adapter_uses_slice_z_not_print_z_and_keeps_object_volume_identity() {
    use ProjectVolumeType::{ModelPart, NegativeVolume, ParameterModifier};

    let (mut restart_objects, _, _) = bfs_restart_request();
    let source_objects = vec![ordinal_gap_object(), restart_objects.remove(0)];
    let objects = intersections(
        &source_objects,
        &[identity_resolved(0), identity_resolved(1)],
        vec![
            planned_layers(0, 0, &[(100.0, 0.5), (101.0, 3.5)]),
            planned_layers(1, 0, &[(100.0, 0.5), (101.0, 3.5)]),
        ],
    )
    .unwrap();

    assert_eq!(objects.len(), 2);
    assert_eq!(
        (objects[0].source_object_index, objects[0].transform_index),
        (0, 0)
    );
    assert_eq!(
        (objects[1].source_object_index, objects[1].transform_index),
        (1, 0)
    );
    assert_eq!(
        objects[0]
            .volumes()
            .iter()
            .map(|volume| (volume.ordinal(), volume.volume_type()))
            .collect::<Vec<_>>(),
        [(2, ModelPart), (3, ParameterModifier), (5, NegativeVolume)]
    );
    assert_eq!(
        objects[1]
            .volumes()
            .iter()
            .map(|volume| (volume.ordinal(), volume.volume_type()))
            .collect::<Vec<_>>(),
        [(1, ModelPart), (2, ModelPart), (3, ModelPart)]
    );
    for volume in objects.iter().flat_map(|object| object.volumes()) {
        assert_eq!(
            volume.layers().iter().map(Vec::len).collect::<Vec<_>>(),
            [1, 0]
        );
    }
    let model_line = objects[0].volumes()[0].layers()[0][0];
    assert_eq!(model_line.a().point().x(), 0);
    assert_eq!(model_line.b().point().x(), 0);

    let scale = CoordinateScale::Normal;
    let lower = scale.factor() * (i64::MIN as f64);
    let upper = -lower;
    let lower_scaled = -2.0_f32.powi(63);
    let below_scaled = f32::from_bits(lower_scaled.to_bits() + 1);
    let below = f64::from(below_scaled) * scale.factor();
    let below_quotient = below / scale.factor();
    assert_eq!(lower_scaled.to_bits(), 0xdf00_0000);
    assert_eq!(below_scaled.to_bits(), 0xdf00_0001);
    assert_eq!((below_quotient as f32).to_bits(), below_scaled.to_bits());
    assert_ne!((below_quotient as f32).to_bits(), lower_scaled.to_bits());
    assert!(below_quotient < i64::MIN as f64);
    adapter_coordinate_case(0.0, translation_x(lower), Some(i64::MIN)).unwrap();
    assert_eq!(
        adapter_coordinate_case(0.0, translation_x(below), None).unwrap_err(),
        coordinate_error()
    );
    assert_eq!(
        adapter_coordinate_case(0.0, translation_xy(0.0, upper), None).unwrap_err(),
        coordinate_error()
    );
    assert_eq!(
        adapter_coordinate_case(
            f64::from(f32::MAX),
            transform("2 0 0 0 1 0 0 0 1 0 0 0"),
            None,
        )
        .unwrap_err(),
        coordinate_error()
    );
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

fn adapter_coordinate_case(
    x: f64,
    volume_transform: Transform3d,
    expected_x: Option<i64>,
) -> Result<(), SliceError> {
    let source = object(
        "adapter-coordinate.model",
        30,
        vec![
            project_volume(
                "adapter-coordinate.model",
                30,
                ProjectVolumeType::ModelPart,
                true,
                false,
            ),
            mesh_volume(
                31,
                ProjectVolumeType::ParameterModifier,
                vec![
                    Point3d::new(x, 0.0, 0.0),
                    Point3d::new(x, 1.0, 0.0),
                    Point3d::new(x, 0.0, 1.0),
                ],
                vec![[0, 1, 2]],
                volume_transform,
            ),
        ],
        &[Transform3d::IDENTITY],
    );
    let objects = intersections(
        std::slice::from_ref(&source),
        &[identity_resolved(0)],
        vec![planned_layers(0, 0, &[(100.0, 0.5)])],
    )?;
    if let Some(expected_x) = expected_x {
        let line = objects[0].volumes()[1].layers()[0][0];
        assert_eq!(line.a().point().x(), expected_x);
        assert_eq!(line.b().point().x(), expected_x);
    }
    Ok(())
}

fn translation_x(x: f64) -> Transform3d {
    translation_xy(x, 0.0)
}

fn translation_xy(x: f64, y: f64) -> Transform3d {
    transform(&format!("1 0 0 0 1 0 0 0 1 {x} {y} 0"))
}

fn coordinate_error() -> SliceError {
    SliceError::InvalidInput(
        "project mesh slicing coordinate is nonfinite or outside the scaled coordinate range"
            .to_owned(),
    )
}
