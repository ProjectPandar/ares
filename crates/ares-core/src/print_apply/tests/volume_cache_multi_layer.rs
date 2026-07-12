use super::super::volume_cache_state::{
    StagedExtentBox, StagedMultiLayerVolumeCacheLayer, StagedVolumeExtents,
    staged_update_volume_bboxes_multi_layer_old_extents,
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

#[test]
fn update_volume_bboxes_multi_layer_old_extents_empty_cache_clears_all_layers() {
    let mut layers = vec![layer(vec![extent(10, 1.0)]), layer(vec![extent(20, 2.0)])];

    let old_extents = staged_update_volume_bboxes_multi_layer_old_extents(&[], &mut layers);

    assert!(old_extents.is_empty());
    assert!(layers[0].volumes().is_empty());
    assert!(layers[1].volumes().is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_old_extents_non_empty_cache_captures_and_clears_layers() {
    let mut layers = vec![layer(vec![extent(10, 1.0)]), layer(vec![extent(20, 2.0)])];

    let old_extents = staged_update_volume_bboxes_multi_layer_old_extents(&[10], &mut layers);

    assert_eq!(
        old_extents,
        vec![vec![extent(10, 1.0)], vec![extent(20, 2.0)]]
    );
    assert!(layers[0].volumes().is_empty());
    assert!(layers[1].volumes().is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_old_extents_accepts_empty_layer_ranges() {
    let mut layers = Vec::new();

    let old_extents = staged_update_volume_bboxes_multi_layer_old_extents(&[10], &mut layers);

    assert!(old_extents.is_empty());
    assert!(layers.is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_old_extents_preserves_empty_per_layer_lists() {
    let mut layers = vec![
        layer(Vec::new()),
        layer(vec![extent(20, 2.0)]),
        layer(Vec::new()),
    ];

    let old_extents = staged_update_volume_bboxes_multi_layer_old_extents(&[20], &mut layers);

    assert_eq!(old_extents, vec![vec![], vec![extent(20, 2.0)], vec![]]);
    assert!(layers.iter().all(|layer| layer.volumes().is_empty()));
}

#[test]
fn update_volume_bboxes_multi_layer_old_extents_preserves_layer_and_volume_order() {
    let mut layers = vec![
        layer(vec![extent(30, 3.0), extent(10, 1.0)]),
        layer(vec![extent(40, 4.0), extent(20, 2.0)]),
    ];

    let old_extents = staged_update_volume_bboxes_multi_layer_old_extents(&[10, 20], &mut layers);

    assert_eq!(
        old_extents,
        vec![
            vec![extent(30, 3.0), extent(10, 1.0)],
            vec![extent(40, 4.0), extent(20, 2.0)],
        ]
    );
    assert!(layers.iter().all(|layer| layer.volumes().is_empty()));
}
