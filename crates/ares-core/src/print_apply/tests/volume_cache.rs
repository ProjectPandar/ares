use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::{
    StagedCachedModelVolume, StagedVolumeCacheRegions,
    staged_print_objects_regions_invalidate_keep_some_volumes,
};

fn volume(
    id: u64,
    volume_type: StagedModelVolumeType,
    transform_key: u64,
) -> StagedCachedModelVolume {
    StagedCachedModelVolume::new(id, volume_type, transform_key)
}

fn model_part(id: u64, transform_key: u64) -> StagedCachedModelVolume {
    volume(id, StagedModelVolumeType::ModelPart, transform_key)
}

#[test]
fn keep_some_volumes_clears_all_regions_before_matching() {
    let mut regions = StagedVolumeCacheRegions::new(vec![10, 20], vec![7]);

    staged_print_objects_regions_invalidate_keep_some_volumes(
        &mut regions,
        &[model_part(7, 1)],
        &[model_part(7, 1)],
    );

    assert!(regions.all_regions().is_empty());
    assert_eq!(regions.cached_volume_ids(), &[7]);
}

#[test]
fn keep_some_volumes_sorts_old_and_new_by_id_before_matching() {
    let mut regions = StagedVolumeCacheRegions::new(Vec::new(), vec![1, 2, 3]);

    staged_print_objects_regions_invalidate_keep_some_volumes(
        &mut regions,
        &[model_part(3, 30), model_part(1, 10), model_part(2, 20)],
        &[model_part(2, 20), model_part(3, 30), model_part(1, 10)],
    );

    assert_eq!(regions.cached_volume_ids(), &[1, 2, 3]);
}

#[test]
fn keep_some_volumes_ignores_non_solid_or_modifier_new_volumes() {
    let mut regions = StagedVolumeCacheRegions::new(Vec::new(), vec![1, 2, 3]);

    staged_print_objects_regions_invalidate_keep_some_volumes(
        &mut regions,
        &[
            volume(1, StagedModelVolumeType::SupportBlocker, 10),
            volume(2, StagedModelVolumeType::SupportEnforcer, 20),
            model_part(3, 30),
        ],
        &[
            volume(1, StagedModelVolumeType::SupportBlocker, 10),
            volume(2, StagedModelVolumeType::SupportEnforcer, 20),
            model_part(3, 30),
        ],
    );

    assert_eq!(regions.cached_volume_ids(), &[3]);
}

#[test]
fn keep_some_volumes_skips_transform_changed_matches() {
    let mut regions = StagedVolumeCacheRegions::new(Vec::new(), vec![1, 2]);

    staged_print_objects_regions_invalidate_keep_some_volumes(
        &mut regions,
        &[model_part(1, 10), model_part(2, 20)],
        &[model_part(1, 11), model_part(2, 20)],
    );

    assert_eq!(regions.cached_volume_ids(), &[2]);
}

#[test]
#[should_panic]
fn keep_some_volumes_panics_when_reusable_volume_is_missing_from_cache() {
    let mut regions = StagedVolumeCacheRegions::new(Vec::new(), vec![1]);

    staged_print_objects_regions_invalidate_keep_some_volumes(
        &mut regions,
        &[model_part(2, 20)],
        &[model_part(2, 20)],
    );
}

#[test]
fn keep_some_volumes_compacts_kept_ids_and_truncates_stale_tail() {
    let mut regions = StagedVolumeCacheRegions::new(Vec::new(), vec![1, 2, 3, 4, 5]);

    staged_print_objects_regions_invalidate_keep_some_volumes(
        &mut regions,
        &[model_part(2, 20), model_part(4, 40)],
        &[model_part(2, 20), model_part(4, 40)],
    );

    assert_eq!(regions.cached_volume_ids(), &[2, 4]);
}

#[test]
fn keep_some_volumes_does_not_retain_skipped_cached_ids_before_later_matches() {
    let mut regions = StagedVolumeCacheRegions::new(Vec::new(), vec![1, 2, 3, 4]);

    staged_print_objects_regions_invalidate_keep_some_volumes(
        &mut regions,
        &[model_part(2, 99), model_part(4, 40)],
        &[model_part(2, 20), model_part(4, 40)],
    );

    assert_eq!(regions.cached_volume_ids(), &[4]);
}

use super::super::volume_cache_state::{
    StagedExtentBox, StagedParentBboxIntersectionGate, StagedVolumeExtents, StagedVolumeRegion,
    staged_find_modifier_volume_extents, staged_find_volume_extents,
    staged_verify_update_parent_bbox_intersection_gate,
};

fn extent(volume_id: u64, marker: f32) -> StagedVolumeExtents {
    StagedVolumeExtents::new(
        volume_id,
        StagedExtentBox::new([marker, 0.0, 0.0], [marker, 1.0, 1.0]),
    )
}

fn wide_extent(volume_id: u64, min: [f32; 3], max: [f32; 3]) -> StagedVolumeExtents {
    StagedVolumeExtents::new(volume_id, StagedExtentBox::new(min, max))
}

fn region(volume_id: u64, is_model_part: bool, parent: isize) -> StagedVolumeRegion {
    StagedVolumeRegion::new(volume_id, is_model_part, parent)
}

#[test]
fn find_volume_extents_returns_exact_match_bbox() {
    let extents = [extent(10, 10.0), extent(20, 20.0), extent(30, 30.0)];

    let bbox = staged_find_volume_extents(&extents, 20).expect("id 20 should exist");

    assert_eq!(bbox.min(), [20.0, 0.0, 0.0]);
    assert_eq!(bbox.max(), [20.0, 1.0, 1.0]);
}

#[test]
fn find_volume_extents_returns_none_for_empty_extents() {
    assert!(staged_find_volume_extents(&[], 20).is_none());
}

#[test]
fn find_volume_extents_returns_none_below_first_mismatch() {
    let extents = [extent(10, 10.0), extent(20, 20.0)];

    assert!(staged_find_volume_extents(&extents, 5).is_none());
}

#[test]
fn find_volume_extents_returns_none_between_ids() {
    let extents = [extent(10, 10.0), extent(20, 20.0)];

    assert!(staged_find_volume_extents(&extents, 15).is_none());
}

#[test]
fn find_volume_extents_returns_none_above_last_id() {
    let extents = [extent(10, 10.0), extent(20, 20.0)];

    assert!(staged_find_volume_extents(&extents, 30).is_none());
}

#[test]
fn find_volume_extents_returns_first_duplicate_id() {
    let extents = [extent(10, 1.0), extent(10, 2.0), extent(20, 3.0)];

    let bbox = staged_find_volume_extents(&extents, 10).expect("id 10 should exist");

    assert_eq!(bbox.min(), [1.0, 0.0, 0.0]);
}

#[test]
fn find_modifier_volume_extents_returns_model_part_bbox_directly() {
    let regions = [region(10, true, -1)];
    let extents = [wide_extent(10, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0])];

    let bbox = staged_find_modifier_volume_extents(&regions, &extents, 0);

    assert_eq!(bbox.min(), [1.0, 2.0, 3.0]);
    assert_eq!(bbox.max(), [4.0, 5.0, 6.0]);
}

#[test]
fn find_modifier_volume_extents_extends_direct_model_part_parent() {
    let regions = [region(10, true, -1), region(20, false, 0)];
    let extents = [
        wide_extent(10, [-5.0, 2.0, 0.0], [0.0, 8.0, 3.0]),
        wide_extent(20, [1.0, -2.0, 1.0], [4.0, 3.0, 9.0]),
    ];

    let bbox = staged_find_modifier_volume_extents(&regions, &extents, 1);

    assert_eq!(bbox.min(), [-5.0, -2.0, 0.0]);
    assert_eq!(bbox.max(), [4.0, 8.0, 9.0]);
}

#[test]
fn find_modifier_volume_extents_extends_multi_level_parent_chain() {
    let regions = [
        region(10, true, -1),
        region(20, false, 0),
        region(30, false, 1),
    ];
    let extents = [
        wide_extent(10, [-5.0, 0.0, 0.0], [0.0, 1.0, 1.0]),
        wide_extent(20, [1.0, -5.0, 1.0], [2.0, 2.0, 2.0]),
        wide_extent(30, [3.0, 3.0, -5.0], [9.0, 9.0, 9.0]),
    ];

    let bbox = staged_find_modifier_volume_extents(&regions, &extents, 2);

    assert_eq!(bbox.min(), [-5.0, -5.0, -5.0]);
    assert_eq!(bbox.max(), [9.0, 9.0, 9.0]);
}

#[test]
#[should_panic]
fn find_modifier_volume_extents_panics_when_current_extents_are_missing() {
    let regions = [region(10, true, -1)];

    staged_find_modifier_volume_extents(&regions, &[], 0);
}

#[test]
#[should_panic]
fn find_modifier_volume_extents_panics_when_parent_extents_are_missing() {
    let regions = [region(10, true, -1), region(20, false, 0)];
    let extents = [wide_extent(20, [1.0, 1.0, 1.0], [2.0, 2.0, 2.0])];

    staged_find_modifier_volume_extents(&regions, &extents, 1);
}

#[test]
#[should_panic]
fn find_modifier_volume_extents_panics_when_modifier_parent_is_negative() {
    let regions = [region(10, false, -1)];
    let extents = [wide_extent(10, [1.0, 1.0, 1.0], [2.0, 2.0, 2.0])];

    staged_find_modifier_volume_extents(&regions, &extents, 0);
}

fn parent_bbox_gate(
    parent_region_id: usize,
    parent_bbox: StagedExtentBox,
    current_modifier_bbox: StagedExtentBox,
    intersects: bool,
) -> StagedParentBboxIntersectionGate {
    StagedParentBboxIntersectionGate::new(
        parent_region_id,
        parent_bbox,
        current_modifier_bbox,
        intersects,
    )
}

#[test]
fn verify_update_parent_bbox_gate_records_intersecting_bboxes() {
    let regions = [region(10, true, -1)];
    let extents = [wide_extent(10, [0.0, 0.0, 0.0], [2.0, 2.0, 2.0])];
    let current_bbox = StagedExtentBox::new([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);

    let gate =
        staged_verify_update_parent_bbox_intersection_gate(&regions, &extents, current_bbox, 0);

    assert_eq!(
        gate,
        parent_bbox_gate(
            0,
            StagedExtentBox::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]),
            current_bbox,
            true,
        )
    );
}

#[test]
fn verify_update_parent_bbox_gate_records_disjoint_bboxes() {
    let regions = [region(10, true, -1)];
    let extents = [wide_extent(10, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])];
    let current_bbox = StagedExtentBox::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);

    let gate =
        staged_verify_update_parent_bbox_intersection_gate(&regions, &extents, current_bbox, 0);

    assert_eq!(
        gate,
        parent_bbox_gate(
            0,
            StagedExtentBox::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            current_bbox,
            false,
        )
    );
}

#[test]
fn verify_update_parent_bbox_gate_treats_touching_boundary_as_intersection() {
    let regions = [region(10, true, -1)];
    let extents = [wide_extent(10, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])];
    let current_bbox = StagedExtentBox::new([1.0, -1.0, -1.0], [2.0, 0.5, 0.5]);

    let gate =
        staged_verify_update_parent_bbox_intersection_gate(&regions, &extents, current_bbox, 0);

    assert_eq!(
        gate,
        parent_bbox_gate(
            0,
            StagedExtentBox::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            current_bbox,
            true,
        )
    );
}

#[test]
fn verify_update_parent_bbox_gate_uses_model_part_bbox_directly() {
    let regions = [region(10, true, -1)];
    let extents = [wide_extent(10, [5.0, 6.0, 7.0], [8.0, 9.0, 10.0])];
    let current_bbox = StagedExtentBox::new([7.0, 8.0, 9.0], [11.0, 12.0, 13.0]);

    let gate =
        staged_verify_update_parent_bbox_intersection_gate(&regions, &extents, current_bbox, 0);

    assert_eq!(
        gate,
        parent_bbox_gate(
            0,
            StagedExtentBox::new([5.0, 6.0, 7.0], [8.0, 9.0, 10.0]),
            current_bbox,
            true,
        )
    );
}

#[test]
fn verify_update_parent_bbox_gate_extends_modifier_parent_chain_before_intersection() {
    let regions = [region(10, true, -1), region(20, false, 0)];
    let extents = [
        wide_extent(10, [-5.0, 0.0, 0.0], [0.0, 1.0, 1.0]),
        wide_extent(20, [2.0, 2.0, 2.0], [3.0, 3.0, 3.0]),
    ];
    let current_bbox = StagedExtentBox::new([-4.0, 0.5, 0.5], [-3.0, 0.75, 0.75]);

    let gate =
        staged_verify_update_parent_bbox_intersection_gate(&regions, &extents, current_bbox, 1);

    assert_eq!(
        gate,
        parent_bbox_gate(
            1,
            StagedExtentBox::new([-5.0, 0.0, 0.0], [3.0, 3.0, 3.0]),
            current_bbox,
            true,
        )
    );
}

#[test]
#[should_panic]
fn verify_update_parent_bbox_gate_panics_when_parent_extents_are_missing() {
    let regions = [region(10, true, -1)];
    let current_bbox = StagedExtentBox::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    staged_verify_update_parent_bbox_intersection_gate(&regions, &[], current_bbox, 0);
}
