use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::{
    StagedUpdateVolumeBboxesVolume, staged_update_volume_bboxes_volume_order_cache_ids,
};

fn update_volume(
    volume_id: u64,
    volume_type: StagedModelVolumeType,
) -> StagedUpdateVolumeBboxesVolume {
    StagedUpdateVolumeBboxesVolume::new(volume_id, volume_type)
}

#[test]
fn update_volume_bboxes_volume_order_sorts_unsorted_input_ids() {
    let mut cached_ids = vec![99];
    let model_volumes = [
        update_volume(30, StagedModelVolumeType::ModelPart),
        update_volume(10, StagedModelVolumeType::ParameterModifier),
        update_volume(20, StagedModelVolumeType::NegativeVolume),
    ];

    let sorted_ids =
        staged_update_volume_bboxes_volume_order_cache_ids(&mut cached_ids, &model_volumes);

    assert_eq!(sorted_ids, vec![10, 20, 30]);
    assert_eq!(cached_ids, vec![10, 20, 30]);
}

#[test]
fn update_volume_bboxes_volume_order_filters_non_solid_or_modifier_volumes() {
    let mut cached_ids = vec![1, 2, 3];
    let model_volumes = [
        update_volume(10, StagedModelVolumeType::SupportBlocker),
        update_volume(20, StagedModelVolumeType::ModelPart),
        update_volume(30, StagedModelVolumeType::SupportEnforcer),
        update_volume(40, StagedModelVolumeType::Invalid),
    ];

    let sorted_ids =
        staged_update_volume_bboxes_volume_order_cache_ids(&mut cached_ids, &model_volumes);

    assert_eq!(sorted_ids, vec![20]);
    assert_eq!(cached_ids, vec![20]);
}

#[test]
fn update_volume_bboxes_volume_order_empty_input_clears_stale_cached_ids() {
    let mut cached_ids = vec![10, 20];

    let sorted_ids = staged_update_volume_bboxes_volume_order_cache_ids(&mut cached_ids, &[]);

    assert!(sorted_ids.is_empty());
    assert!(cached_ids.is_empty());
}

#[test]
fn update_volume_bboxes_volume_order_preserves_already_sorted_eligible_ids() {
    let mut cached_ids = Vec::new();
    let model_volumes = [
        update_volume(10, StagedModelVolumeType::ModelPart),
        update_volume(20, StagedModelVolumeType::NegativeVolume),
        update_volume(30, StagedModelVolumeType::ParameterModifier),
    ];

    let sorted_ids =
        staged_update_volume_bboxes_volume_order_cache_ids(&mut cached_ids, &model_volumes);

    assert_eq!(sorted_ids, vec![10, 20, 30]);
    assert_eq!(cached_ids, vec![10, 20, 30]);
}

#[test]
fn update_volume_bboxes_volume_order_preserves_duplicate_ids_after_sorting() {
    let mut cached_ids = Vec::new();
    let model_volumes = [
        update_volume(20, StagedModelVolumeType::ModelPart),
        update_volume(10, StagedModelVolumeType::NegativeVolume),
        update_volume(20, StagedModelVolumeType::ParameterModifier),
    ];

    let sorted_ids =
        staged_update_volume_bboxes_volume_order_cache_ids(&mut cached_ids, &model_volumes);

    assert_eq!(sorted_ids, vec![10, 20, 20]);
    assert_eq!(cached_ids, vec![10, 20, 20]);
}

#[test]
fn update_volume_bboxes_volume_order_replaces_stale_cached_ids() {
    let mut cached_ids = vec![1, 2, 3, 4];
    let model_volumes = [
        update_volume(40, StagedModelVolumeType::SupportBlocker),
        update_volume(30, StagedModelVolumeType::ModelPart),
        update_volume(10, StagedModelVolumeType::ParameterModifier),
    ];

    let sorted_ids =
        staged_update_volume_bboxes_volume_order_cache_ids(&mut cached_ids, &model_volumes);

    assert_eq!(sorted_ids, vec![10, 30]);
    assert_eq!(cached_ids, vec![10, 30]);
}
