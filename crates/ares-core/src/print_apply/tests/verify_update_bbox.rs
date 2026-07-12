use super::super::verify_update_region_state::staged_verify_update_current_modifier_bbox;
use super::super::volume_cache_state::{StagedExtentBox, StagedVolumeExtents};

fn verify_extent(volume_id: u64, min: [f32; 3], max: [f32; 3]) -> StagedVolumeExtents {
    StagedVolumeExtents::new(volume_id, StagedExtentBox::new(min, max))
}

#[test]
fn verify_update_current_modifier_bbox_returns_exact_matching_bbox() {
    let extents = [
        verify_extent(10, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
        verify_extent(20, [2.0, 3.0, 4.0], [5.0, 6.0, 7.0]),
    ];

    let bbox = staged_verify_update_current_modifier_bbox(&extents, 20);

    assert_eq!(bbox, StagedExtentBox::new([2.0, 3.0, 4.0], [5.0, 6.0, 7.0]));
}

#[test]
#[should_panic]
fn verify_update_current_modifier_bbox_panics_when_current_extents_are_missing() {
    let extents = [verify_extent(10, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])];

    staged_verify_update_current_modifier_bbox(&extents, 20);
}

#[test]
#[should_panic]
fn verify_update_current_modifier_bbox_does_not_use_neighbor_extents() {
    let extents = [
        verify_extent(10, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
        verify_extent(30, [3.0, 3.0, 3.0], [4.0, 4.0, 4.0]),
    ];

    staged_verify_update_current_modifier_bbox(&extents, 20);
}
