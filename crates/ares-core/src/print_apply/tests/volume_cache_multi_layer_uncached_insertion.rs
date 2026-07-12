use super::super::mesh_state::{StagedBoundingBox3f, StagedRangeBoundingBox3f};
use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::{
    StagedExtentBox, StagedMultiLayerVolumeCacheLayer, StagedUpdateVolumeBboxesVolume,
    StagedVolumeExtents, staged_update_volume_bboxes_multi_layer_uncached_insertion,
};

fn bbox(marker: f32) -> StagedExtentBox {
    StagedExtentBox::new([marker, 0.0, 0.0], [marker, 1.0, 1.0])
}

fn mesh_bbox(marker: f32) -> StagedRangeBoundingBox3f {
    StagedRangeBoundingBox3f::new_populated(StagedBoundingBox3f::from_min_max(
        [marker, 0.0, 0.0],
        [marker, 1.0, 1.0],
    ))
}

fn empty_mesh_bbox() -> StagedRangeBoundingBox3f {
    StagedRangeBoundingBox3f::new_empty()
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
fn update_volume_bboxes_multi_layer_uncached_insertion_appends_populated_bboxes_across_layers() {
    let bboxes = vec![vec![mesh_bbox(1.0), mesh_bbox(2.0)]];
    let mut layers = vec![layer(Vec::new()), layer(Vec::new())];

    staged_update_volume_bboxes_multi_layer_uncached_insertion(
        &[],
        &bboxes,
        &mut layers,
        &[volume(10, StagedModelVolumeType::ModelPart)],
    );

    assert_eq!(layers[0].volumes(), &[extent(10, 1.0)]);
    assert_eq!(layers[1].volumes(), &[extent(10, 2.0)]);
}

#[test]
fn update_volume_bboxes_multi_layer_uncached_insertion_skips_unpopulated_layer_bboxes() {
    let bboxes = vec![vec![mesh_bbox(1.0), empty_mesh_bbox()]];
    let mut layers = vec![layer(Vec::new()), layer(Vec::new())];

    staged_update_volume_bboxes_multi_layer_uncached_insertion(
        &[],
        &bboxes,
        &mut layers,
        &[volume(10, StagedModelVolumeType::NegativeVolume)],
    );

    assert_eq!(layers[0].volumes(), &[extent(10, 1.0)]);
    assert!(layers[1].volumes().is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_uncached_insertion_ignores_cached_ids() {
    let bboxes = vec![vec![mesh_bbox(1.0)]];
    let mut layers = vec![layer(Vec::new())];

    staged_update_volume_bboxes_multi_layer_uncached_insertion(
        &[10],
        &bboxes,
        &mut layers,
        &[volume(10, StagedModelVolumeType::ParameterModifier)],
    );

    assert!(layers[0].volumes().is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_uncached_insertion_filters_non_solid_or_modifier_volumes() {
    let bboxes = vec![
        vec![mesh_bbox(1.0)],
        vec![mesh_bbox(2.0)],
        vec![mesh_bbox(3.0)],
    ];
    let mut layers = vec![layer(Vec::new())];
    let model_volumes = [
        volume(10, StagedModelVolumeType::SupportBlocker),
        volume(20, StagedModelVolumeType::SupportEnforcer),
        volume(30, StagedModelVolumeType::Invalid),
    ];

    staged_update_volume_bboxes_multi_layer_uncached_insertion(
        &[],
        &bboxes,
        &mut layers,
        &model_volumes,
    );

    assert!(layers[0].volumes().is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_uncached_insertion_preserves_prefix_and_model_order() {
    let bboxes = vec![vec![mesh_bbox(2.0)], vec![mesh_bbox(1.0)]];
    let mut layers = vec![layer(vec![extent(99, 9.0)])];
    let model_volumes = [
        volume(20, StagedModelVolumeType::ModelPart),
        volume(10, StagedModelVolumeType::ModelPart),
    ];

    staged_update_volume_bboxes_multi_layer_uncached_insertion(
        &[],
        &bboxes,
        &mut layers,
        &model_volumes,
    );

    assert_eq!(
        layers[0].volumes(),
        &[extent(99, 9.0), extent(20, 2.0), extent(10, 1.0)]
    );
}

#[test]
fn update_volume_bboxes_multi_layer_uncached_insertion_processes_duplicate_uncached_ids() {
    let bboxes = vec![vec![mesh_bbox(1.0)], vec![mesh_bbox(2.0)]];
    let mut layers = vec![layer(Vec::new())];
    let model_volumes = [
        volume(10, StagedModelVolumeType::ModelPart),
        volume(10, StagedModelVolumeType::NegativeVolume),
    ];

    staged_update_volume_bboxes_multi_layer_uncached_insertion(
        &[],
        &bboxes,
        &mut layers,
        &model_volumes,
    );

    assert_eq!(layers[0].volumes(), &[extent(10, 1.0), extent(10, 2.0)]);
}
