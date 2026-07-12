use super::super::print_region_state::{
    StagedPrintRegionRefCount, staged_print_region_ref_cnt, staged_print_region_ref_inc,
};
use super::super::verify_update_region_state::{
    StagedVerifyModelVolume, staged_verify_update_print_object_regions_init,
};

fn volume(id: u64) -> StagedVerifyModelVolume {
    StagedVerifyModelVolume::new(id)
}

fn ids(volumes: &[StagedVerifyModelVolume]) -> Vec<u64> {
    volumes.iter().map(StagedVerifyModelVolume::id).collect()
}

#[test]
fn verify_update_region_init_sorts_unsorted_model_volumes_by_id() {
    let mut volumes = [volume(30), volume(10), volume(20)];
    let mut regions = [];

    staged_verify_update_print_object_regions_init(&mut volumes, &mut regions);

    assert_eq!(ids(&volumes), [10, 20, 30]);
}

#[test]
fn verify_update_region_init_keeps_already_sorted_model_volume_ids() {
    let mut volumes = [volume(10), volume(20), volume(30)];
    let mut regions = [];

    staged_verify_update_print_object_regions_init(&mut volumes, &mut regions);

    assert_eq!(ids(&volumes), [10, 20, 30]);
}

#[test]
fn verify_update_region_init_groups_duplicate_model_volume_ids() {
    let mut volumes = [volume(20), volume(10), volume(20)];
    let mut regions = [];

    staged_verify_update_print_object_regions_init(&mut volumes, &mut regions);

    assert_eq!(ids(&volumes), [10, 20, 20]);
}

#[test]
fn verify_update_region_init_resets_all_existing_print_region_ref_counts() {
    let mut volumes = [];
    let mut regions = [
        StagedPrintRegionRefCount::default(),
        StagedPrintRegionRefCount::default(),
    ];
    staged_print_region_ref_inc(&mut regions[0]);
    staged_print_region_ref_inc(&mut regions[0]);
    staged_print_region_ref_inc(&mut regions[1]);

    staged_verify_update_print_object_regions_init(&mut volumes, &mut regions);

    assert_eq!(staged_print_region_ref_cnt(&regions[0]), 0);
    assert_eq!(staged_print_region_ref_cnt(&regions[1]), 0);
}

#[test]
fn verify_update_region_init_accepts_empty_inputs() {
    let mut volumes = [];
    let mut regions = [];

    staged_verify_update_print_object_regions_init(&mut volumes, &mut regions);

    assert!(volumes.is_empty());
    assert!(regions.is_empty());
}

use super::super::model_volume_state::StagedModelVolumeType;
use super::super::verify_update_region_state::{
    StagedVerifyRegionMatch, StagedVerifyVolumeRegion, staged_verify_update_volume_region_matches,
};

fn verify_region(volume_id: u64, volume_type: StagedModelVolumeType) -> StagedVerifyVolumeRegion {
    StagedVerifyVolumeRegion::new(volume_id, volume_type)
}

fn match_record(
    region_id: usize,
    volume_id: u64,
    first_modifier_visit: bool,
) -> StagedVerifyRegionMatch {
    StagedVerifyRegionMatch::new(region_id, volume_id, first_modifier_visit)
}

#[test]
fn verify_update_volume_region_matches_skips_ineligible_volume_types() {
    let volumes = [
        volume(10),
        volume(20),
        volume(30),
        volume(40),
        volume(50),
        volume(60),
    ];
    let regions = [
        verify_region(10, StagedModelVolumeType::Invalid),
        verify_region(20, StagedModelVolumeType::ModelPart),
        verify_region(30, StagedModelVolumeType::NegativeVolume),
        verify_region(40, StagedModelVolumeType::ParameterModifier),
        verify_region(50, StagedModelVolumeType::SupportBlocker),
        verify_region(60, StagedModelVolumeType::SupportEnforcer),
    ];

    let matches = staged_verify_update_volume_region_matches(&volumes, &regions);

    assert_eq!(
        matches,
        vec![match_record(1, 20, false), match_record(3, 40, true)]
    );
}

#[test]
fn verify_update_volume_region_matches_exact_sorted_model_volume_ids() {
    let volumes = [volume(10), volume(20), volume(30)];
    let regions = [
        verify_region(30, StagedModelVolumeType::ModelPart),
        verify_region(10, StagedModelVolumeType::ParameterModifier),
        verify_region(20, StagedModelVolumeType::ModelPart),
    ];

    let matches = staged_verify_update_volume_region_matches(&volumes, &regions);

    assert_eq!(
        matches,
        vec![
            match_record(0, 30, false),
            match_record(1, 10, true),
            match_record(2, 20, false),
        ]
    );
}

#[test]
#[should_panic]
fn verify_update_volume_region_matches_panics_for_missing_model_volume_id() {
    let volumes = [volume(10), volume(30)];
    let regions = [verify_region(20, StagedModelVolumeType::ModelPart)];

    staged_verify_update_volume_region_matches(&volumes, &regions);
}

#[test]
fn verify_update_volume_region_matches_deduplicates_consecutive_modifier_visits() {
    let volumes = [volume(10), volume(20)];
    let regions = [
        verify_region(10, StagedModelVolumeType::ParameterModifier),
        verify_region(10, StagedModelVolumeType::ParameterModifier),
        verify_region(20, StagedModelVolumeType::ParameterModifier),
        verify_region(20, StagedModelVolumeType::ParameterModifier),
    ];

    let matches = staged_verify_update_volume_region_matches(&volumes, &regions);

    assert_eq!(
        matches,
        vec![
            match_record(0, 10, true),
            match_record(1, 10, false),
            match_record(2, 20, true),
            match_record(3, 20, false),
        ]
    );
}

#[test]
fn verify_update_volume_region_matches_model_parts_do_not_update_last_modifier() {
    let volumes = [volume(10), volume(20)];
    let regions = [
        verify_region(10, StagedModelVolumeType::ParameterModifier),
        verify_region(20, StagedModelVolumeType::ModelPart),
        verify_region(10, StagedModelVolumeType::ParameterModifier),
    ];

    let matches = staged_verify_update_volume_region_matches(&volumes, &regions);

    assert_eq!(
        matches,
        vec![
            match_record(0, 10, true),
            match_record(1, 20, false),
            match_record(2, 10, false),
        ]
    );
}

#[test]
fn verify_update_volume_region_matches_preserves_source_region_order() {
    let volumes = [volume(10), volume(20), volume(30)];
    let regions = [
        verify_region(30, StagedModelVolumeType::ParameterModifier),
        verify_region(10, StagedModelVolumeType::ModelPart),
        verify_region(20, StagedModelVolumeType::ParameterModifier),
    ];

    let matches = staged_verify_update_volume_region_matches(&volumes, &regions);

    assert_eq!(
        matches,
        vec![
            match_record(0, 30, true),
            match_record(1, 10, false),
            match_record(2, 20, true),
        ]
    );
}

use super::super::verify_update_region_state::{
    StagedVerifyParentScan, staged_verify_update_modifier_parent_scan,
};

fn verify_region_with_parent(
    volume_id: u64,
    volume_type: StagedModelVolumeType,
    parent: isize,
) -> StagedVerifyVolumeRegion {
    StagedVerifyVolumeRegion::with_parent(volume_id, volume_type, parent)
}

fn parent_scan(
    final_next_region_id: usize,
    scanned_parent_ids: Vec<usize>,
    existing_override_parent_ids: Vec<usize>,
) -> StagedVerifyParentScan {
    StagedVerifyParentScan::new(
        final_next_region_id,
        scanned_parent_ids,
        existing_override_parent_ids,
    )
}

#[test]
fn verify_update_modifier_parent_scan_records_descending_eligible_parent_ids() {
    let regions = [
        verify_region(10, StagedModelVolumeType::ModelPart),
        verify_region(20, StagedModelVolumeType::ParameterModifier),
        verify_region(30, StagedModelVolumeType::ModelPart),
        verify_region(40, StagedModelVolumeType::ParameterModifier),
    ];

    let scan = staged_verify_update_modifier_parent_scan(&regions, 3);

    assert_eq!(scan, parent_scan(3, vec![2, 1, 0], Vec::new()));
}

#[test]
fn verify_update_modifier_parent_scan_skips_ineligible_parent_candidates() {
    let regions = [
        verify_region(10, StagedModelVolumeType::ModelPart),
        verify_region(20, StagedModelVolumeType::SupportBlocker),
        verify_region(30, StagedModelVolumeType::Invalid),
        verify_region(40, StagedModelVolumeType::ParameterModifier),
    ];

    let scan = staged_verify_update_modifier_parent_scan(&regions, 3);

    assert_eq!(scan, parent_scan(3, vec![0], Vec::new()));
}

#[test]
#[should_panic]
fn verify_update_modifier_parent_scan_panics_when_parent_has_same_volume_id() {
    let regions = [
        verify_region(10, StagedModelVolumeType::ModelPart),
        verify_region(40, StagedModelVolumeType::ModelPart),
        verify_region(40, StagedModelVolumeType::ParameterModifier),
    ];

    staged_verify_update_modifier_parent_scan(&regions, 2);
}

#[test]
#[should_panic]
fn verify_update_modifier_parent_scan_panics_on_generated_region_ordering_violation() {
    let regions = [
        verify_region(10, StagedModelVolumeType::ModelPart),
        verify_region_with_parent(40, StagedModelVolumeType::ParameterModifier, 2),
        verify_region(20, StagedModelVolumeType::ModelPart),
        verify_region(40, StagedModelVolumeType::ParameterModifier),
    ];

    staged_verify_update_modifier_parent_scan(&regions, 1);
}

#[test]
fn verify_update_modifier_parent_scan_advances_for_existing_override() {
    let regions = [
        verify_region(10, StagedModelVolumeType::ModelPart),
        verify_region(20, StagedModelVolumeType::ModelPart),
        verify_region_with_parent(40, StagedModelVolumeType::ParameterModifier, 1),
        verify_region(40, StagedModelVolumeType::ParameterModifier),
    ];

    let scan = staged_verify_update_modifier_parent_scan(&regions, 2);

    assert_eq!(scan, parent_scan(3, vec![1, 0], vec![1]));
}

#[test]
fn verify_update_modifier_parent_scan_keeps_next_region_id_without_existing_override() {
    let regions = [
        verify_region(10, StagedModelVolumeType::ModelPart),
        verify_region(20, StagedModelVolumeType::ModelPart),
        verify_region(40, StagedModelVolumeType::ParameterModifier),
    ];

    let scan = staged_verify_update_modifier_parent_scan(&regions, 2);

    assert_eq!(scan, parent_scan(2, vec![1, 0], Vec::new()));
}

#[test]
fn verify_update_modifier_parent_scan_advances_for_multiple_sequential_existing_overrides() {
    let regions = [
        verify_region(10, StagedModelVolumeType::ModelPart),
        verify_region(20, StagedModelVolumeType::ModelPart),
        verify_region(30, StagedModelVolumeType::ModelPart),
        verify_region_with_parent(40, StagedModelVolumeType::ParameterModifier, 2),
        verify_region_with_parent(40, StagedModelVolumeType::ParameterModifier, 1),
        verify_region(40, StagedModelVolumeType::ParameterModifier),
    ];

    let scan = staged_verify_update_modifier_parent_scan(&regions, 3);

    assert_eq!(scan, parent_scan(5, vec![2, 1, 0], vec![2, 1]));
}
