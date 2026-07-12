use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::{
    StagedExtentBox, StagedMultiLayerVolumeCacheLayer, StagedUpdateVolumeBboxesVolume,
    StagedVolumeExtents, staged_update_volume_bboxes_multi_layer_cached_reuse,
};

fn bbox(marker: f32) -> StagedExtentBox {
    StagedExtentBox::new([marker, 0.0, 0.0], [marker, 1.0, 1.0])
}

fn extent(volume_id: u64, marker: f32) -> StagedVolumeExtents {
    StagedVolumeExtents::new(volume_id, bbox(marker))
}

fn layer(extents: Vec<StagedVolumeExtents>) -> StagedMultiLayerVolumeCacheLayer {
    StagedMultiLayerVolumeCacheLayer::new(extents)
}

fn volume(volume_id: u64, volume_type: StagedModelVolumeType) -> StagedUpdateVolumeBboxesVolume {
    StagedUpdateVolumeBboxesVolume::new(volume_id, volume_type)
}

#[test]
fn update_volume_bboxes_multi_layer_cached_reuse_reuses_cached_extents_across_layers() {
    let old_extents = vec![vec![extent(10, 1.0)], vec![extent(10, 2.0)]];
    let mut layers = vec![layer(Vec::new()), layer(Vec::new())];

    staged_update_volume_bboxes_multi_layer_cached_reuse(
        &[10],
        &old_extents,
        &mut layers,
        &[volume(10, StagedModelVolumeType::ModelPart)],
    );

    assert_eq!(layers[0].volumes(), &[extent(10, 1.0)]);
    assert_eq!(layers[1].volumes(), &[extent(10, 2.0)]);
}

#[test]
fn update_volume_bboxes_multi_layer_cached_reuse_skips_missing_old_extent_per_layer() {
    let old_extents = vec![vec![extent(10, 1.0)], vec![extent(20, 2.0)]];
    let mut layers = vec![layer(Vec::new()), layer(Vec::new())];

    staged_update_volume_bboxes_multi_layer_cached_reuse(
        &[10],
        &old_extents,
        &mut layers,
        &[volume(10, StagedModelVolumeType::ModelPart)],
    );

    assert_eq!(layers[0].volumes(), &[extent(10, 1.0)]);
    assert!(layers[1].volumes().is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_cached_reuse_defers_uncached_volumes() {
    let old_extents = vec![vec![extent(10, 1.0)]];
    let mut layers = vec![layer(Vec::new())];

    staged_update_volume_bboxes_multi_layer_cached_reuse(
        &[],
        &old_extents,
        &mut layers,
        &[volume(10, StagedModelVolumeType::NegativeVolume)],
    );

    assert!(layers[0].volumes().is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_cached_reuse_filters_non_solid_or_modifier_volumes() {
    let old_extents = vec![vec![extent(10, 1.0), extent(20, 2.0), extent(30, 3.0)]];
    let mut layers = vec![layer(Vec::new())];
    let model_volumes = [
        volume(10, StagedModelVolumeType::SupportBlocker),
        volume(20, StagedModelVolumeType::SupportEnforcer),
        volume(30, StagedModelVolumeType::Invalid),
    ];

    staged_update_volume_bboxes_multi_layer_cached_reuse(
        &[10, 20, 30],
        &old_extents,
        &mut layers,
        &model_volumes,
    );

    assert!(layers[0].volumes().is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_cached_reuse_preserves_model_order_and_duplicate_visits() {
    let old_extents = vec![vec![extent(10, 1.0), extent(20, 2.0)]];
    let mut layers = vec![layer(vec![extent(99, 9.0)])];
    let model_volumes = [
        volume(20, StagedModelVolumeType::ModelPart),
        volume(10, StagedModelVolumeType::ParameterModifier),
        volume(20, StagedModelVolumeType::NegativeVolume),
    ];

    staged_update_volume_bboxes_multi_layer_cached_reuse(
        &[10, 20],
        &old_extents,
        &mut layers,
        &model_volumes,
    );

    assert_eq!(
        layers[0].volumes(),
        &[
            extent(99, 9.0),
            extent(20, 2.0),
            extent(10, 1.0),
            extent(20, 2.0)
        ]
    );
}

#[test]
fn update_volume_bboxes_multi_layer_cached_reuse_accepts_empty_layers_and_old_extents() {
    let mut layers = Vec::new();

    staged_update_volume_bboxes_multi_layer_cached_reuse(
        &[10],
        &[],
        &mut layers,
        &[volume(10, StagedModelVolumeType::ModelPart)],
    );

    assert!(layers.is_empty());
}
