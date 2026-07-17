use crate::{ProjectVolumeType, SliceError, Transform3d, load_project, slice_project};

use super::super::state::prepare_project_slice;
use super::{
    raw_support::{intersections, preflight_order_scenarios},
    support::{
        KsrArchive, identity_resolved, ksr_project, metadata, object, plan, project_volume,
        project_volume_at_x, slot_limit, unsupported,
    },
};

const ROOT_COMPONENT: &str = r#"<component p:path="/3D/Objects/ksr_fdmtest_v4.drc_2.model" objectid="1" p:UUID="00020000-b206-40ff-9872-83e8017abed1" transform="1 0 0 0 1 0 0 0 1 0 0 0"/>"#;
const REPEATED_COMPONENT: &str =
    r#"<component p:path="/3D/Objects/ksr_fdmtest_v4.drc_2.model" objectid="1"/>"#;
const FIRST_VERTEX: &str = r#"<vertex x="17.6525421" y="-26.3965759" z="-45.5"/>"#;
const NONFINITE_VERTEX: &str = r#"<vertex x="NaN" y="-26.3965759" z="-45.5"/>"#;
const MIN_LAYER_HEIGHTS: &str = concat!(
    "\t\"min_layer_height\": [\r\n",
    "\t\t\"0.08\",\r\n",
    "\t\t\"0.08\"\r\n",
    "\t]",
);
const EMPTY_MIN_LAYER_HEIGHTS: &str = "\t\"min_layer_height\": []";
const FILAMENT_SHRINK: &str = concat!(
    "\t\"filament_shrink\": [\r\n",
    "\t\t\"100%\",\r\n",
    "\t\t\"100%\"\r\n",
    "\t]",
);
const NONIDENTITY_FILAMENT_SHRINK: &str = concat!(
    "\t\"filament_shrink\": [\r\n",
    "\t\t\"99%\",\r\n",
    "\t\t\"100%\"\r\n",
    "\t]",
);
const FILAMENT_SHRINK_Z: &str = concat!(
    "\t\"filament_shrinkage_compensation_z\": [\r\n",
    "\t\t\"100%\",\r\n",
    "\t\t\"100%\"\r\n",
    "\t]",
);
const NONIDENTITY_FILAMENT_SHRINK_Z: &str = concat!(
    "\t\"filament_shrinkage_compensation_z\": [\r\n",
    "\t\t\"101%\",\r\n",
    "\t\t\"100%\"\r\n",
    "\t]",
);
const LAYER_HEIGHT_RANGE: &str = r#"<objects><object id="1"><range min_z="0" max_z="1"><option opt_key="layer_height">0.18</option></range></object></objects>"#;
const EXTRUDER_RANGE: &str = r#"<objects><object id="1"><range min_z="0" max_z="1"><option opt_key="extruder">1</option></range></object></objects>"#;

#[tokio::test]
async fn task22b_lifecycle_preserves_load_config_writer_task22a_and_raw_error_precedence() {
    let malformed = b"not a project archive";
    assert_eq!(
        slice_project(malformed, metadata()).await.unwrap_err(),
        load_project(malformed).unwrap_err()
    );

    let mut cycle = KsrArchive::new();
    make_shrink_nonidentity(&mut cycle);
    cycle.replace_unique(
        "3D/3dmodel.model",
        ROOT_COMPONENT,
        r#"<component objectid="2"/>"#,
    );
    assert_eq!(
        slice_project(cycle.bytes(), metadata()).await.unwrap_err(),
        SliceError::InvalidInput(
            "invalid project model graph: component graph contains a cycle".to_owned()
        )
    );

    let mut chain = KsrArchive::new();
    chain.replace_unique(
        "3D/Objects/ksr_fdmtest_v4.drc_2.model",
        FIRST_VERTEX,
        NONFINITE_VERTEX,
    );
    chain.replace_unique(
        "Metadata/project_settings.config",
        MIN_LAYER_HEIGHTS,
        EMPTY_MIN_LAYER_HEIGHTS,
    );
    chain.invalidate_flush_matrix();
    set_scalar(&mut chain, "raft_layers", "0", "1");
    assert_eq!(
        slice_error(&chain).await,
        SliceError::InvalidInput("project mesh vertices must be finite".to_owned())
    );

    chain.replace_unique(
        "3D/Objects/ksr_fdmtest_v4.drc_2.model",
        NONFINITE_VERTEX,
        FIRST_VERTEX,
    );
    assert_eq!(
        slice_error(&chain).await,
        SliceError::InvalidInput("invalid Orca option min_layer_height".to_owned())
    );
    chain.replace_unique(
        "Metadata/project_settings.config",
        EMPTY_MIN_LAYER_HEIGHTS,
        MIN_LAYER_HEIGHTS,
    );
    assert_eq!(slice_error(&chain).await, flush_matrix_error());
    chain.repair_flush_matrix();
    assert_eq!(
        slice_error(&chain).await,
        SliceError::UnsupportedProjectFeature("raft_layers".to_owned())
    );
    set_scalar(&mut chain, "raft_layers", "1", "0");
    chain.insert_text("Metadata/layer_config_ranges.xml", LAYER_HEIGHT_RANGE);
    assert_eq!(
        slice_error(&chain).await,
        SliceError::UnsupportedProjectFeature("layer_height".to_owned())
    );
    chain.replace_unique(
        "Metadata/layer_config_ranges.xml",
        LAYER_HEIGHT_RANGE,
        EXTRUDER_RANGE,
    );
    assert_eq!(
        slice_error(&chain).await,
        SliceError::UnsupportedProjectFeature("layer_config_ranges".to_owned())
    );

    let mut expanded = KsrArchive::new();
    make_shrink_nonidentity(&mut expanded);
    let repeated_components = REPEATED_COMPONENT.repeat(55);
    expanded.replace_unique("3D/3dmodel.model", ROOT_COMPONENT, &repeated_components);
    assert_eq!(
        slice_project(expanded.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::InvalidInput(
            "project expanded model item count exceeds supported limit of 1000000".to_owned()
        )
    );

    for (objects, resolved, plans, expected) in preflight_order_scenarios() {
        assert_eq!(
            intersections(&objects, &resolved, plans).unwrap_err(),
            expected
        );
    }
    assert_later_request_preflights_beat_earlier_coordinate();

    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );

    let mut non_bambu = KsrArchive::new();
    set_scalar(
        &mut non_bambu,
        "printer_model",
        "Bambu Lab X2D",
        "Generic FFF",
    );
    let non_bambu_state = prepare_project_slice(non_bambu.clone().bytes()).unwrap();
    assert!(non_bambu_state.config_block.is_none());
    assert_eq!(non_bambu_state.intersected_objects.len(), 1);
    assert_eq!(non_bambu_state.intersected_objects[0].volumes().len(), 1);
    assert_eq!(
        non_bambu_state.intersected_objects[0].volumes()[0]
            .layers()
            .len(),
        460
    );
    assert_eq!(
        slice_error(&non_bambu).await,
        SliceError::ProjectSlicingIncomplete
    );
}

#[tokio::test]
async fn task22b_identity_shrink_options_precede_task22a_and_raw_state() {
    let mut archive = KsrArchive::new();
    archive.invalidate_flush_matrix();
    set_scalar(&mut archive, "raft_layers", "0", "1");
    archive.insert_text("Metadata/layer_config_ranges.xml", EXTRUDER_RANGE);
    archive.replace_unique(
        "Metadata/project_settings.config",
        FILAMENT_SHRINK,
        NONIDENTITY_FILAMENT_SHRINK,
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        FILAMENT_SHRINK_Z,
        NONIDENTITY_FILAMENT_SHRINK_Z,
    );
    assert_eq!(
        slice_error(&archive).await,
        SliceError::UnsupportedProjectFeature("filament_shrink".to_owned())
    );

    archive.replace_unique(
        "Metadata/project_settings.config",
        NONIDENTITY_FILAMENT_SHRINK,
        FILAMENT_SHRINK,
    );
    assert_eq!(
        slice_error(&archive).await,
        SliceError::UnsupportedProjectFeature("filament_shrinkage_compensation_z".to_owned())
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        NONIDENTITY_FILAMENT_SHRINK_Z,
        FILAMENT_SHRINK_Z,
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
        SliceError::UnsupportedProjectFeature("layer_config_ranges".to_owned())
    );
}

fn set_scalar(archive: &mut KsrArchive, key: &str, from: &str, to: &str) {
    archive.replace_unique(
        "Metadata/project_settings.config",
        &format!("\t\"{key}\": \"{from}\","),
        &format!("\t\"{key}\": \"{to}\","),
    );
}

fn make_shrink_nonidentity(archive: &mut KsrArchive) {
    archive.replace_unique(
        "Metadata/project_settings.config",
        FILAMENT_SHRINK,
        NONIDENTITY_FILAMENT_SHRINK,
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

fn assert_later_request_preflights_beat_earlier_coordinate() {
    let invalid_coordinate = object(
        "invalid-coordinate.model",
        1,
        vec![project_volume_at_x(
            "invalid-coordinate.model",
            1,
            ProjectVolumeType::ModelPart,
            f64::from(f32::MAX),
        )],
        &[Transform3d::IDENTITY],
    );
    assert_eq!(
        intersections(
            std::slice::from_ref(&invalid_coordinate),
            &[identity_resolved(0)],
            vec![plan(0, 0, 1)]
        )
        .unwrap_err(),
        coordinate_error()
    );

    let shared_later = object(
        "shared-later.model",
        2,
        vec![project_volume(
            "shared-later.model",
            2,
            ProjectVolumeType::ModelPart,
            true,
            true,
        )],
        &[Transform3d::IDENTITY],
    );
    assert_eq!(
        intersections(
            &[invalid_coordinate.clone(), shared_later],
            &[identity_resolved(0), identity_resolved(1)],
            vec![plan(0, 0, 1), plan(1, 0, 1)]
        )
        .unwrap_err(),
        unsupported("shared_mesh_centering")
    );

    let dense_later = object(
        "dense-later.model",
        3,
        (0..10)
            .map(|index| {
                project_volume(
                    "dense-later.model",
                    10 + index,
                    ProjectVolumeType::ModelPart,
                    true,
                    false,
                )
            })
            .collect(),
        &[Transform3d::IDENTITY],
    );
    assert_eq!(
        intersections(
            &[invalid_coordinate, dense_later],
            &[identity_resolved(0), identity_resolved(1)],
            vec![plan(0, 0, 1), plan(1, 0, 100_000)]
        )
        .unwrap_err(),
        slot_limit()
    );
}

fn coordinate_error() -> SliceError {
    SliceError::InvalidInput(
        "project mesh slicing coordinate is nonfinite or outside the scaled coordinate range"
            .to_owned(),
    )
}
