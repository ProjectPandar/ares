use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::{
    StagedExtentBox, StagedSingleLayerVolumeBboxInput, StagedVolumeExtents,
    staged_update_volume_bboxes_single_layer,
};

fn bbox(marker: f32) -> StagedExtentBox {
    StagedExtentBox::new([marker, 0.0, 0.0], [marker, 1.0, 1.0])
}

fn extent(volume_id: u64, marker: f32) -> StagedVolumeExtents {
    StagedVolumeExtents::new(volume_id, bbox(marker))
}

fn input(
    volume_id: u64,
    volume_type: StagedModelVolumeType,
    marker: f32,
) -> StagedSingleLayerVolumeBboxInput {
    StagedSingleLayerVolumeBboxInput::new(volume_id, volume_type, bbox(marker))
}

#[test]
fn update_volume_bboxes_single_layer_reuses_cached_old_extent() {
    let cached_ids = [10];
    let old_extents = [extent(10, 1.0)];
    let model_volumes = [input(10, StagedModelVolumeType::ModelPart, 9.0)];

    let extents =
        staged_update_volume_bboxes_single_layer(&cached_ids, &old_extents, &model_volumes);

    assert_eq!(extents, vec![extent(10, 1.0)]);
}

#[test]
fn update_volume_bboxes_single_layer_inserts_supplied_new_extent_for_uncached_volume() {
    let cached_ids = [];
    let old_extents = [];
    let model_volumes = [input(20, StagedModelVolumeType::NegativeVolume, 2.0)];

    let extents =
        staged_update_volume_bboxes_single_layer(&cached_ids, &old_extents, &model_volumes);

    assert_eq!(extents, vec![extent(20, 2.0)]);
}

#[test]
fn update_volume_bboxes_single_layer_skips_cached_volume_when_old_extent_is_missing() {
    let cached_ids = [30];
    let old_extents = [];
    let model_volumes = [input(30, StagedModelVolumeType::ParameterModifier, 3.0)];

    let extents =
        staged_update_volume_bboxes_single_layer(&cached_ids, &old_extents, &model_volumes);

    assert!(extents.is_empty());
}

#[test]
fn update_volume_bboxes_single_layer_filters_non_solid_or_modifier_volumes() {
    let cached_ids = [];
    let old_extents = [];
    let model_volumes = [
        input(10, StagedModelVolumeType::SupportBlocker, 1.0),
        input(20, StagedModelVolumeType::SupportEnforcer, 2.0),
        input(30, StagedModelVolumeType::Invalid, 3.0),
        input(40, StagedModelVolumeType::ModelPart, 4.0),
    ];

    let extents =
        staged_update_volume_bboxes_single_layer(&cached_ids, &old_extents, &model_volumes);

    assert_eq!(extents, vec![extent(40, 4.0)]);
}

#[test]
fn update_volume_bboxes_single_layer_preserves_input_order() {
    let cached_ids = [10, 30];
    let old_extents = [extent(10, 1.0), extent(30, 3.0)];
    let model_volumes = [
        input(30, StagedModelVolumeType::ModelPart, 9.0),
        input(20, StagedModelVolumeType::ModelPart, 2.0),
        input(10, StagedModelVolumeType::ModelPart, 8.0),
    ];

    let extents =
        staged_update_volume_bboxes_single_layer(&cached_ids, &old_extents, &model_volumes);

    assert_eq!(
        extents,
        vec![extent(30, 3.0), extent(20, 2.0), extent(10, 1.0)]
    );
}

#[test]
fn update_volume_bboxes_single_layer_processes_duplicate_ids_independently() {
    let cached_ids = [10];
    let old_extents = [extent(10, 1.0)];
    let model_volumes = [
        input(10, StagedModelVolumeType::ModelPart, 9.0),
        input(20, StagedModelVolumeType::ModelPart, 2.0),
        input(20, StagedModelVolumeType::ParameterModifier, 3.0),
    ];

    let extents =
        staged_update_volume_bboxes_single_layer(&cached_ids, &old_extents, &model_volumes);

    assert_eq!(
        extents,
        vec![extent(10, 1.0), extent(20, 2.0), extent(20, 3.0)]
    );
}

#[test]
fn update_volume_bboxes_single_layer_empty_input_returns_empty_extents() {
    let cached_ids = [10];
    let old_extents = [extent(10, 1.0)];

    let extents = staged_update_volume_bboxes_single_layer(&cached_ids, &old_extents, &[]);

    assert!(extents.is_empty());
}
