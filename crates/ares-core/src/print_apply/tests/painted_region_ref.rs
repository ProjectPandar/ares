use super::super::painted_region_state::staged_painted_region_ref_inc;
use super::super::print_region_state::StagedPrintRegionRefCount;
use super::super::verify_update_config_state::{
    StagedConfigValue, StagedExistingRegionConfigApply, StagedExistingRegionRefIncrement,
    StagedExistingRegionUpdateAction,
};

fn value(key: &str, fingerprint: u64) -> StagedConfigValue {
    StagedConfigValue::new(key, fingerprint)
}

fn staged_apply(values: &[(&str, u64)]) -> StagedExistingRegionConfigApply {
    StagedExistingRegionConfigApply::new(
        values
            .iter()
            .map(|(key, fingerprint)| value(key, *fingerprint))
            .collect(),
        false,
        true,
    )
}

fn ref_increment(count_after: i32) -> StagedExistingRegionRefIncrement {
    StagedExistingRegionRefIncrement::new(count_after)
}

#[test]
fn painted_region_ref_inc_increments_unchanged_zero_ref_region() {
    let mut region = StagedPrintRegionRefCount::default();

    let increment = staged_painted_region_ref_inc(
        StagedExistingRegionUpdateAction::Unchanged,
        None,
        &mut region,
    );

    assert_eq!(increment, Some(ref_increment(1)));
}

#[test]
fn painted_region_ref_inc_updates_in_place_when_apply_exists() {
    let mut region = StagedPrintRegionRefCount::default();
    let apply = staged_apply(&[("wall_filament", 7)]);

    let increment = staged_painted_region_ref_inc(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        Some(&apply),
        &mut region,
    );

    assert_eq!(increment, Some(ref_increment(1)));
}

#[test]
fn painted_region_ref_inc_skips_update_in_place_without_apply() {
    let mut region = StagedPrintRegionRefCount::default();

    let increment = staged_painted_region_ref_inc(
        StagedExistingRegionUpdateAction::UpdateInPlace,
        None,
        &mut region,
    );

    assert_eq!(increment, None);
    assert_eq!(
        staged_painted_region_ref_inc(
            StagedExistingRegionUpdateAction::Unchanged,
            None,
            &mut region,
        ),
        Some(ref_increment(1))
    );
}

#[test]
fn painted_region_ref_inc_skips_requires_reslice() {
    let mut region = StagedPrintRegionRefCount::default();
    let apply = staged_apply(&[("wall_filament", 7)]);

    let increment = staged_painted_region_ref_inc(
        StagedExistingRegionUpdateAction::RequiresReslice,
        Some(&apply),
        &mut region,
    );

    assert_eq!(increment, None);
    assert_eq!(
        staged_painted_region_ref_inc(
            StagedExistingRegionUpdateAction::Unchanged,
            None,
            &mut region,
        ),
        Some(ref_increment(1))
    );
}

#[test]
fn painted_region_ref_inc_accumulates_successful_increments() {
    let mut region = StagedPrintRegionRefCount::default();
    let apply = staged_apply(&[("wall_filament", 7)]);

    assert_eq!(
        staged_painted_region_ref_inc(
            StagedExistingRegionUpdateAction::Unchanged,
            None,
            &mut region,
        ),
        Some(ref_increment(1))
    );
    assert_eq!(
        staged_painted_region_ref_inc(
            StagedExistingRegionUpdateAction::UpdateInPlace,
            Some(&apply),
            &mut region,
        ),
        Some(ref_increment(2))
    );
}
