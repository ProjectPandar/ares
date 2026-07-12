use super::super::region_merge_state::{
    StagedMergeVerificationResult, StagedRegionMergeRegion, staged_region_merge_verification,
};

fn region(
    region_id: usize,
    config_fingerprint: u64,
    config_hash: u64,
    ref_count: i32,
) -> StagedRegionMergeRegion {
    StagedRegionMergeRegion::new(region_id, config_fingerprint, config_hash, ref_count)
}

fn valid() -> StagedMergeVerificationResult {
    StagedMergeVerificationResult::Valid
}

fn requires_reslice(
    first_region_id: usize,
    second_region_id: usize,
) -> StagedMergeVerificationResult {
    StagedMergeVerificationResult::RequiresReslice {
        first_region_id,
        second_region_id,
    }
}

#[test]
fn region_merge_verification_accepts_empty_all_regions() {
    let result = staged_region_merge_verification(&[]);

    assert_eq!(result, valid());
}

#[test]
fn region_merge_verification_accepts_unique_referenced_regions() {
    let regions = [region(1, 10, 100, 1), region(2, 20, 200, 2)];

    let result = staged_region_merge_verification(&regions);

    assert_eq!(result, valid());
}

#[test]
fn region_merge_verification_requires_reslice_for_equal_config_in_same_hash_group() {
    let regions = [region(1, 10, 100, 1), region(2, 10, 100, 1)];

    let result = staged_region_merge_verification(&regions);

    assert_eq!(result, requires_reslice(1, 2));
}

#[test]
fn region_merge_verification_accepts_hash_collision_with_unequal_configs() {
    let regions = [region(1, 10, 100, 1), region(2, 20, 100, 1)];

    let result = staged_region_merge_verification(&regions);

    assert_eq!(result, valid());
}

#[test]
fn region_merge_verification_does_not_compare_equal_configs_with_different_hashes() {
    let regions = [region(1, 10, 100, 1), region(2, 10, 200, 1)];

    let result = staged_region_merge_verification(&regions);

    assert_eq!(result, valid());
}

#[test]
fn region_merge_verification_sorts_by_hash_before_comparing_groups() {
    let regions = [
        region(1, 10, 300, 1),
        region(2, 20, 100, 1),
        region(3, 20, 100, 1),
        region(4, 10, 200, 1),
    ];

    let result = staged_region_merge_verification(&regions);

    assert_eq!(result, requires_reslice(2, 3));
}

#[test]
#[should_panic(expected = "print region ref count must be positive")]
fn region_merge_verification_panics_for_zero_ref_region() {
    let regions = [region(1, 10, 100, 0)];

    let _ = staged_region_merge_verification(&regions);
}

#[test]
#[should_panic(expected = "print region ref count must be positive")]
fn region_merge_verification_panics_for_negative_ref_region() {
    let regions = [region(1, 10, 100, -1)];

    let _ = staged_region_merge_verification(&regions);
}
