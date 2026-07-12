use super::super::print_region_state::{
    StagedPrintRegionRefCount, staged_print_region_ref_cnt, staged_print_region_ref_inc,
    staged_print_region_ref_reset,
};

#[test]
fn print_region_ref_count_defaults_to_zero() {
    let region = StagedPrintRegionRefCount::default();

    assert_eq!(staged_print_region_ref_cnt(&region), 0);
}

#[test]
fn print_region_ref_inc_accumulates_count() {
    let mut region = StagedPrintRegionRefCount::default();

    staged_print_region_ref_inc(&mut region);
    staged_print_region_ref_inc(&mut region);

    assert_eq!(staged_print_region_ref_cnt(&region), 2);
}

#[test]
fn print_region_ref_reset_clears_incremented_count() {
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);
    staged_print_region_ref_inc(&mut region);

    staged_print_region_ref_reset(&mut region);

    assert_eq!(staged_print_region_ref_cnt(&region), 0);
}

#[test]
fn print_region_ref_cnt_reads_without_mutating() {
    let mut region = StagedPrintRegionRefCount::default();
    staged_print_region_ref_inc(&mut region);

    assert_eq!(staged_print_region_ref_cnt(&region), 1);
    assert_eq!(staged_print_region_ref_cnt(&region), 1);
}

#[test]
fn print_region_ref_reset_is_idempotent_at_zero() {
    let mut region = StagedPrintRegionRefCount::default();

    staged_print_region_ref_reset(&mut region);
    staged_print_region_ref_reset(&mut region);

    assert_eq!(staged_print_region_ref_cnt(&region), 0);
}
